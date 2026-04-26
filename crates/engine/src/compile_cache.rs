use std::collections::{HashMap, HashSet};

use gowasm_compiler::{
    compile_workspace_with_graph, module_source_key_for_path,
    recompile_workspace_affected_packages, CompileError, CompiledWorkspace, SourceInput,
};
use gowasm_host_types::WorkspaceFile;
use gowasm_vm::{
    program_debug_info, program_type_inventory, register_program_debug_info,
    register_program_type_inventory, Program, ProgramDebugInfo, ProgramTypeInventory,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompileReuse {
    Recompiled,
    RecompiledAffectedPackages,
    ReusedExact,
    ReusedUnaffected,
}

enum CompilePlan {
    Reuse(CompileReuse),
    RecompileChangedPackages(Vec<String>),
    RecompileAll,
}

pub(super) struct CachedCompileSession {
    files: HashMap<String, String>,
    compiled_workspace: CompiledWorkspace,
    debug_info: Option<ProgramDebugInfo>,
    type_inventory: Option<ProgramTypeInventory>,
    pub(super) last_reuse: CompileReuse,
    pub(super) last_recompiled_import_paths: Vec<String>,
}

#[derive(Default)]
pub(super) struct CompileCache {
    pub(super) sessions: HashMap<String, CachedCompileSession>,
}

impl CompileCache {
    pub(super) fn compile(
        &mut self,
        files: &[WorkspaceFile],
        sources: &[SourceInput<'_>],
        entry_path: &str,
    ) -> Result<Program, CompileError> {
        if let Some(plan) = self
            .sessions
            .get(entry_path)
            .map(|session| session.compile_plan(files))
        {
            match plan {
                CompilePlan::Reuse(reuse) => {
                    let session = self
                        .sessions
                        .get_mut(entry_path)
                        .expect("compile session should still exist");
                    session.last_reuse = reuse;
                    session.last_recompiled_import_paths.clear();
                    return Ok(clone_program_with_debug_info(
                        &session.compiled_workspace.program,
                        session.debug_info.clone(),
                        session.type_inventory.clone(),
                    ));
                }
                CompilePlan::RecompileChangedPackages(changed_import_paths) => {
                    let session = self
                        .sessions
                        .get(entry_path)
                        .expect("compile session should still exist");
                    if let Some((compiled, recompiled_import_paths)) =
                        recompile_workspace_affected_packages(
                            &session.compiled_workspace,
                            sources,
                            entry_path,
                            &changed_import_paths,
                        )?
                    {
                        let mut session = CachedCompileSession::from_compiled(files, compiled);
                        session.last_reuse = CompileReuse::RecompiledAffectedPackages;
                        session.last_recompiled_import_paths = recompiled_import_paths;
                        self.sessions.insert(entry_path.into(), session);
                        let session = self
                            .sessions
                            .get(entry_path)
                            .expect("compile session should exist after insert");
                        return Ok(clone_program_with_debug_info(
                            &session.compiled_workspace.program,
                            session.debug_info.clone(),
                            session.type_inventory.clone(),
                        ));
                    }
                }
                CompilePlan::RecompileAll => {}
            }
        }

        let compiled = compile_workspace_with_graph(sources, entry_path)?;
        self.sessions.insert(
            entry_path.into(),
            CachedCompileSession::from_compiled(files, compiled),
        );
        let session = self
            .sessions
            .get(entry_path)
            .expect("compile session should exist after insert");
        Ok(clone_program_with_debug_info(
            &session.compiled_workspace.program,
            session.debug_info.clone(),
            session.type_inventory.clone(),
        ))
    }

    pub(super) fn invalidate(&mut self, entry_path: &str) {
        self.sessions.remove(entry_path);
    }
}

impl CachedCompileSession {
    fn from_compiled(files: &[WorkspaceFile], compiled: CompiledWorkspace) -> Self {
        let debug_info = program_debug_info(&compiled.program);
        let type_inventory = program_type_inventory(&compiled.program);
        let last_recompiled_import_paths = compiled.rebuild_graph.dependency_order().to_vec();
        Self {
            files: workspace_files_map(files),
            compiled_workspace: compiled,
            debug_info,
            type_inventory,
            last_reuse: CompileReuse::Recompiled,
            last_recompiled_import_paths,
        }
    }

    fn compile_plan(&self, files: &[WorkspaceFile]) -> CompilePlan {
        let current_files = workspace_files_map(files);
        let changed_paths = changed_paths(&self.files, &current_files);
        if changed_paths.is_empty() {
            return CompilePlan::Reuse(CompileReuse::ReusedExact);
        }
        if changed_paths.contains("go.mod") {
            return CompilePlan::RecompileAll;
        }

        let mut changed = self
            .compiled_workspace
            .rebuild_graph
            .changed_package_import_paths_for_source_paths(changed_paths.iter().map(String::as_str))
            .into_iter()
            .collect::<HashSet<_>>();
        changed.extend(
            self.compiled_workspace
                .rebuild_graph
                .changed_package_import_paths_for_modules(
                    changed_paths
                        .iter()
                        .filter_map(|path| module_source_key_for_path(path)),
                ),
        );

        if changed.is_empty() {
            return CompilePlan::Reuse(CompileReuse::ReusedUnaffected);
        }

        CompilePlan::RecompileChangedPackages(
            self.compiled_workspace
                .rebuild_graph
                .dependency_order()
                .iter()
                .filter(|import_path| changed.contains(import_path.as_str()))
                .cloned()
                .collect(),
        )
    }
}

fn clone_program_with_debug_info(
    program: &Program,
    debug_info: Option<ProgramDebugInfo>,
    type_inventory: Option<ProgramTypeInventory>,
) -> Program {
    let cloned = program.clone();
    if let Some(debug_info) = debug_info {
        register_program_debug_info(&cloned, debug_info);
    }
    if let Some(type_inventory) = type_inventory {
        register_program_type_inventory(&cloned, type_inventory);
    }
    cloned
}

fn workspace_files_map(files: &[WorkspaceFile]) -> HashMap<String, String> {
    files
        .iter()
        .map(|file| (file.path.clone(), file.contents.clone()))
        .collect()
}

fn changed_paths(
    previous_files: &HashMap<String, String>,
    current_files: &HashMap<String, String>,
) -> HashSet<String> {
    previous_files
        .keys()
        .chain(current_files.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .filter(|path| previous_files.get(path) != current_files.get(path))
        .collect()
}
