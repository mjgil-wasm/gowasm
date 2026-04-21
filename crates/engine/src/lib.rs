use std::collections::HashMap;

use gowasm_compiler::SourceInput;
use gowasm_host_types::{
    CapabilityResult, Diagnostic, EngineInfo, EngineRequest, EngineResponse, ErrorCategory,
    ModuleGraphRoot, TestResultDetails, TestRunnerKind, WorkspaceFile, ENGINE_PROTOCOL_VERSION,
};
use gowasm_vm::{CapabilityRequest as VmCapabilityRequest, Program, RunOutcome, Vm, VmError};

mod capability_bridge;
mod compile_cache;
mod compile_diagnostics;
mod diagnostic_source;
mod formatting;
mod linting;
mod module_loading;
mod runtime_diagnostics;
mod snippet_runner;
mod test_result_details;
mod test_runner;

use capability_bridge::{apply_capability_result, map_vm_capability_request};
use compile_cache::CompileCache;
use compile_diagnostics::compile_error_diagnostic;
use formatting::format_workspace_files;
use linting::lint_workspace_files;
use module_loading::{resume_module_graph_load, start_module_graph_load, PausedModuleLoad};
use runtime_diagnostics::vm_error_diagnostic;
use snippet_runner::{prepare_snippet_test, SnippetSourceMapper};
use test_result_details::{finalize_test_result_details, finalize_test_result_details_with_stdout};
use test_runner::prepare_package_test;

pub const ENGINE_NAME: &str = "gowasm-engine";
const DEFAULT_COOPERATIVE_YIELD_INTERVAL: u64 = 10_000;

struct PausedRun {
    response_kind: ExecutionResponseKind,
    entry_path: String,
    files: Vec<WorkspaceFile>,
    program: Program,
    vm: Vm,
    pending_request: VmCapabilityRequest,
}

#[derive(Clone)]
enum ExecutionResponseKind {
    Run,
    Test {
        runner: TestRunnerKind,
        details: TestResultDetails,
        source_mapper: Option<SnippetSourceMapper>,
    },
}

enum ResumeRequestOutcome {
    Completed {
        response_kind: ExecutionResponseKind,
        stdout: String,
    },
    Errored {
        response_kind: ExecutionResponseKind,
        entry_path: String,
        files: Vec<WorkspaceFile>,
        stdout: String,
        error: VmError,
    },
}

#[derive(Default)]
pub struct Engine {
    next_module_request_id: u64,
    next_run_id: u64,
    paused_module_loads: HashMap<u64, PausedModuleLoad>,
    paused_runs: HashMap<u64, PausedRun>,
    compile_cache: CompileCache,
    instruction_budget: Option<u64>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_instruction_budget(instruction_budget: u64) -> Self {
        Self {
            instruction_budget: Some(instruction_budget),
            ..Self::default()
        }
    }

    pub fn handle_request(&mut self, request: EngineRequest) -> EngineResponse {
        match request {
            EngineRequest::Boot => EngineResponse::Ready {
                info: EngineInfo {
                    protocol_version: ENGINE_PROTOCOL_VERSION,
                    engine_name: ENGINE_NAME.into(),
                },
            },
            EngineRequest::LoadModuleGraph { modules } => self.load_module_graph_request(modules),
            EngineRequest::Compile { files, entry_path } => {
                self.compile_request(&files, &entry_path)
            }
            EngineRequest::Format { files } => self.format_request(&files),
            EngineRequest::Lint { files } => self.lint_request(&files),
            EngineRequest::TestPackage {
                files,
                target_path,
                filter,
            } => self.test_package_request(&files, &target_path, filter.as_deref()),
            EngineRequest::TestSnippet { files, entry_path } => {
                self.test_snippet_request(&files, &entry_path)
            }
            EngineRequest::Run {
                files,
                entry_path,
                host_time_unix_nanos,
                host_time_unix_millis,
            } => self.run_request(
                &files,
                &entry_path,
                host_time_unix_nanos,
                host_time_unix_millis,
            ),
            EngineRequest::Resume { run_id, capability } => self.resume_request(run_id, capability),
            EngineRequest::ResumeModule { request_id, module } => {
                self.resume_module_graph_request(request_id, module)
            }
            EngineRequest::Cancel => {
                self.paused_module_loads.clear();
                self.paused_runs.clear();
                cancelled_response()
            }
        }
    }

    pub fn handle_request_json(&mut self, input: &str) -> String {
        let response = match serde_json::from_str::<EngineRequest>(input) {
            Ok(request) => self.handle_request(request),
            Err(error) => fatal_response(
                ErrorCategory::ProtocolError,
                format!("invalid engine request json: {error}"),
            ),
        };

        serde_json::to_string(&response).expect("engine responses should always serialize")
    }

    fn load_module_graph_request(&mut self, modules: Vec<ModuleGraphRoot>) -> EngineResponse {
        start_module_graph_load(
            &mut self.next_module_request_id,
            &mut self.paused_module_loads,
            modules,
        )
    }

    fn resume_module_graph_request(
        &mut self,
        request_id: u64,
        module: gowasm_host_types::ModuleResult,
    ) -> EngineResponse {
        resume_module_graph_load(&mut self.paused_module_loads, request_id, module)
    }

    fn run_request(
        &mut self,
        files: &[WorkspaceFile],
        entry_path: &str,
        host_time_unix_nanos: Option<i64>,
        host_time_unix_millis: Option<i64>,
    ) -> EngineResponse {
        self.execute_request_with_compile_files(
            files,
            files,
            entry_path,
            host_time_unix_nanos,
            host_time_unix_millis,
            ExecutionResponseKind::Run,
        )
    }

    fn execute_request_with_compile_files(
        &mut self,
        files: &[WorkspaceFile],
        compile_files: &[WorkspaceFile],
        entry_path: &str,
        host_time_unix_nanos: Option<i64>,
        host_time_unix_millis: Option<i64>,
        response_kind: ExecutionResponseKind,
    ) -> EngineResponse {
        let sources = match build_source_inputs(compile_files, entry_path) {
            Ok(sources) => sources,
            Err(response) => return response,
        };

        let program = match self.compile_cache.compile(files, &sources, entry_path) {
            Ok(program) => program,
            Err(error) => {
                self.compile_cache.invalidate(entry_path);
                return execution_compile_error(response_kind, compile_files, entry_path, &error);
            }
        };

        let mut vm = Vm::new();
        vm.workspace_files = files
            .iter()
            .map(|file| (file.path.clone(), file.contents.clone()))
            .collect();
        let host_time_unix_nanos =
            match resolve_host_time_unix_nanos(host_time_unix_nanos, host_time_unix_millis) {
                Ok(value) => value,
                Err(response) => return response,
            };
        if let Some(unix_nanos) = host_time_unix_nanos {
            vm.set_time_now_override_unix_nanos(unix_nanos);
        }
        vm.enable_capability_requests();
        vm.set_instruction_yield_interval(DEFAULT_COOPERATIVE_YIELD_INTERVAL);
        if let Some(instruction_budget) = self.instruction_budget {
            vm.set_instruction_budget(instruction_budget);
        }

        match vm.start_program(&program) {
            Ok(RunOutcome::Completed) => {
                execution_completed(response_kind, vm.stdout().to_string())
            }
            Ok(RunOutcome::CapabilityRequest(kind)) => {
                let run_id = self.allocate_run_id();
                let capability = map_vm_capability_request(kind.clone());
                self.paused_runs.insert(
                    run_id,
                    PausedRun {
                        response_kind,
                        entry_path: entry_path.into(),
                        files: files.to_vec(),
                        program,
                        vm,
                        pending_request: kind,
                    },
                );
                EngineResponse::CapabilityRequest { run_id, capability }
            }
            Err(error) => execution_vm_error_with_stdout(
                response_kind,
                files,
                entry_path,
                vm.stdout().to_string(),
                &error,
            ),
        }
    }

    fn resume_request(&mut self, run_id: u64, capability: CapabilityResult) -> EngineResponse {
        let outcome = {
            let Some(paused_run) = self.paused_runs.get_mut(&run_id) else {
                return fatal_response(
                    ErrorCategory::ProtocolError,
                    format!("run `{run_id}` is not waiting for a capability result"),
                );
            };
            let entry_path = paused_run.entry_path.clone();
            let response_kind = paused_run.response_kind.clone();

            if let Err(response) = apply_capability_result(
                &entry_path,
                &paused_run.program,
                &mut paused_run.vm,
                paused_run.pending_request.clone(),
                capability,
            ) {
                return response;
            }

            match paused_run.vm.resume_program(&paused_run.program) {
                Ok(RunOutcome::Completed) => ResumeRequestOutcome::Completed {
                    response_kind,
                    stdout: paused_run.vm.stdout().to_string(),
                },
                Ok(RunOutcome::CapabilityRequest(kind)) => {
                    paused_run.pending_request = kind.clone();
                    return EngineResponse::CapabilityRequest {
                        run_id,
                        capability: map_vm_capability_request(kind),
                    };
                }
                Err(error) => ResumeRequestOutcome::Errored {
                    response_kind,
                    entry_path: paused_run.entry_path.clone(),
                    files: paused_run.files.clone(),
                    stdout: paused_run.vm.stdout().to_string(),
                    error,
                },
            }
        };

        self.paused_runs.remove(&run_id);
        match outcome {
            ResumeRequestOutcome::Completed {
                response_kind,
                stdout,
            } => execution_completed(response_kind, stdout),
            ResumeRequestOutcome::Errored {
                response_kind,
                entry_path,
                files,
                stdout,
                error,
            } => execution_vm_error_with_stdout(response_kind, &files, &entry_path, stdout, &error),
        }
    }

    fn compile_request(&mut self, files: &[WorkspaceFile], entry_path: &str) -> EngineResponse {
        let sources = match build_source_inputs(files, entry_path) {
            Ok(sources) => sources,
            Err(response) => return response,
        };

        match self.compile_cache.compile(files, &sources, entry_path) {
            Ok(_) => EngineResponse::Diagnostics {
                diagnostics: Vec::new(),
            },
            Err(error) => {
                self.compile_cache.invalidate(entry_path);
                EngineResponse::Diagnostics {
                    diagnostics: vec![compile_error_diagnostic(files, entry_path, &error, None)],
                }
            }
        }
    }

    fn format_request(&mut self, files: &[WorkspaceFile]) -> EngineResponse {
        let (files, diagnostics) = format_workspace_files(files);
        EngineResponse::FormatResult { files, diagnostics }
    }

    fn lint_request(&mut self, files: &[WorkspaceFile]) -> EngineResponse {
        EngineResponse::LintResult {
            diagnostics: lint_workspace_files(files),
        }
    }

    fn test_package_request(
        &mut self,
        files: &[WorkspaceFile],
        target_path: &str,
        filter: Option<&str>,
    ) -> EngineResponse {
        let prepared = match prepare_package_test(files, target_path, filter) {
            Ok(prepared) => prepared,
            Err(diagnostics) => {
                return test_result(
                    TestRunnerKind::Package,
                    false,
                    String::new(),
                    diagnostics,
                    TestResultDetails {
                        subject_path: target_path.into(),
                        planned_tests: Vec::new(),
                        completed_tests: Vec::new(),
                        active_test: None,
                    },
                );
            }
        };
        self.execute_request_with_compile_files(
            &prepared.files,
            &prepared.files,
            &prepared.entry_path,
            None,
            None,
            ExecutionResponseKind::Test {
                runner: TestRunnerKind::Package,
                details: prepared.details,
                source_mapper: None,
            },
        )
    }

    fn test_snippet_request(
        &mut self,
        files: &[WorkspaceFile],
        entry_path: &str,
    ) -> EngineResponse {
        let prepared = match prepare_snippet_test(files, entry_path) {
            Ok(prepared) => prepared,
            Err(diagnostics) => {
                return test_result(
                    TestRunnerKind::Snippet,
                    false,
                    String::new(),
                    diagnostics,
                    TestResultDetails {
                        subject_path: entry_path.into(),
                        planned_tests: vec![entry_path.into()],
                        completed_tests: Vec::new(),
                        active_test: Some(entry_path.into()),
                    },
                );
            }
        };
        self.execute_request_with_compile_files(
            files,
            &prepared.compile_files,
            &prepared.entry_path,
            None,
            None,
            ExecutionResponseKind::Test {
                runner: TestRunnerKind::Snippet,
                details: prepared.details,
                source_mapper: prepared.source_mapper,
            },
        )
    }

    fn allocate_run_id(&mut self) -> u64 {
        let run_id = self.next_run_id;
        self.next_run_id += 1;
        run_id
    }
}

pub fn handle_request(request: EngineRequest) -> EngineResponse {
    Engine::new().handle_request(request)
}

pub fn handle_request_json(input: &str) -> String {
    Engine::new().handle_request_json(input)
}

#[allow(clippy::result_large_err)]
fn resolve_host_time_unix_nanos(
    host_time_unix_nanos: Option<i64>,
    host_time_unix_millis: Option<i64>,
) -> Result<Option<i64>, EngineResponse> {
    if let Some(unix_nanos) = host_time_unix_nanos {
        return Ok(Some(unix_nanos));
    }
    let Some(unix_millis) = host_time_unix_millis else {
        return Ok(None);
    };
    let unix_nanos = unix_millis.checked_mul(1_000_000).ok_or_else(|| {
        fatal_response(
            ErrorCategory::ProtocolError,
            format!("host_time_unix_millis `{unix_millis}` overflowed Unix nanoseconds"),
        )
    })?;
    Ok(Some(unix_nanos))
}

#[allow(clippy::result_large_err)]
fn build_source_inputs<'a>(
    files: &'a [WorkspaceFile],
    entry_path: &str,
) -> Result<Vec<SourceInput<'a>>, EngineResponse> {
    if !files.iter().any(|file| file.path == entry_path) {
        return Err(fatal_response(
            ErrorCategory::ProtocolError,
            format!("entry file `{entry_path}` was not found in the workspace payload"),
        ));
    }

    Ok(files
        .iter()
        .map(|file| SourceInput {
            path: file.path.as_str(),
            source: file.contents.as_str(),
        })
        .collect())
}

fn execution_completed(response_kind: ExecutionResponseKind, stdout: String) -> EngineResponse {
    match response_kind {
        ExecutionResponseKind::Run => EngineResponse::RunResult {
            stdout,
            diagnostics: Vec::new(),
        },
        ExecutionResponseKind::Test {
            runner,
            details,
            source_mapper: _,
        } => test_result(
            runner,
            true,
            stdout.clone(),
            Vec::new(),
            finalize_test_result_details_with_stdout(runner, details, &stdout, true),
        ),
    }
}

fn execution_compile_error(
    response_kind: ExecutionResponseKind,
    files: &[WorkspaceFile],
    entry_path: &str,
    error: &gowasm_compiler::CompileError,
) -> EngineResponse {
    let source_mapper = match &response_kind {
        ExecutionResponseKind::Test {
            source_mapper: Some(source_mapper),
            ..
        } => Some(source_mapper),
        _ => None,
    };
    let diagnostics = vec![compile_error_diagnostic(
        files,
        entry_path,
        error,
        source_mapper,
    )];
    match response_kind {
        ExecutionResponseKind::Run => EngineResponse::RunResult {
            stdout: String::new(),
            diagnostics,
        },
        ExecutionResponseKind::Test {
            runner,
            details,
            source_mapper: _,
        } => test_result(
            runner,
            false,
            String::new(),
            diagnostics,
            finalize_test_result_details(runner, details, false),
        ),
    }
}

fn execution_vm_error(
    response_kind: ExecutionResponseKind,
    files: &[WorkspaceFile],
    entry_path: &str,
    error: &VmError,
) -> EngineResponse {
    execution_vm_error_with_stdout(response_kind, files, entry_path, String::new(), error)
}

fn execution_vm_error_with_stdout(
    response_kind: ExecutionResponseKind,
    files: &[WorkspaceFile],
    entry_path: &str,
    stdout: String,
    error: &VmError,
) -> EngineResponse {
    let source_mapper = match &response_kind {
        ExecutionResponseKind::Test {
            source_mapper: Some(source_mapper),
            ..
        } => Some(source_mapper),
        _ => None,
    };
    let diagnostics = vec![vm_error_diagnostic(files, entry_path, error, source_mapper)];
    match response_kind {
        ExecutionResponseKind::Run => EngineResponse::RunResult {
            stdout,
            diagnostics,
        },
        ExecutionResponseKind::Test {
            runner,
            details,
            source_mapper: _,
        } => test_result(
            runner,
            false,
            stdout.clone(),
            diagnostics,
            finalize_test_result_details_with_stdout(runner, details, &stdout, false),
        ),
    }
}

fn vm_run_error(entry_path: &str, error: &VmError) -> EngineResponse {
    execution_vm_error(ExecutionResponseKind::Run, &[], entry_path, error)
}

fn test_result(
    runner: TestRunnerKind,
    passed: bool,
    stdout: String,
    diagnostics: Vec<Diagnostic>,
    details: TestResultDetails,
) -> EngineResponse {
    EngineResponse::TestResult {
        runner,
        passed,
        stdout,
        diagnostics,
        details,
    }
}

fn cancelled_response() -> EngineResponse {
    EngineResponse::Cancelled {
        category: ErrorCategory::RuntimeCancellation,
    }
}

fn fatal_response(category: ErrorCategory, message: String) -> EngineResponse {
    EngineResponse::Fatal { message, category }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app_samples_one;
#[cfg(test)]
mod tests_app_samples_reflect_json;
#[cfg(test)]
mod tests_app_samples_two;
#[cfg(test)]
mod tests_capabilities;
#[cfg(test)]
mod tests_capability_compat;
#[cfg(test)]
mod tests_compile_fail_diagnostics_golden;
#[cfg(test)]
mod tests_diagnostics;
#[cfg(test)]
mod tests_diagnostics_golden;
#[cfg(test)]
mod tests_fetch;
#[cfg(test)]
mod tests_fetch_more;
#[cfg(test)]
mod tests_fetch_response_fidelity;
#[cfg(test)]
mod tests_fetch_streaming;
#[cfg(test)]
mod tests_fetch_streaming_context;
#[cfg(test)]
mod tests_formatting;
#[cfg(test)]
mod tests_fuzz_harness;
#[cfg(test)]
mod tests_incremental_compile;
#[cfg(test)]
mod tests_incremental_compile_shapes;
#[cfg(test)]
mod tests_linting;
#[cfg(test)]
mod tests_module_protocol;
#[cfg(test)]
mod tests_module_resolution;
#[cfg(test)]
mod tests_negative_support_corpus;
#[cfg(test)]
mod tests_os_env;
#[cfg(test)]
mod tests_os_error_contracts;
#[cfg(test)]
mod tests_os_fs_mutation;
#[cfg(test)]
mod tests_os_identity;
#[cfg(test)]
mod tests_os_paths;
#[cfg(test)]
mod tests_parity_corpus;
#[cfg(test)]
mod tests_protocol;
#[cfg(test)]
mod tests_replay;
#[cfg(test)]
mod tests_test_runner;
#[cfg(test)]
mod tests_time_host_contract;
#[cfg(test)]
mod tests_workspace_fs;
#[cfg(test)]
mod tests_workspace_fs_interfaces;
#[cfg(test)]
mod tests_workspace_fs_walk;
