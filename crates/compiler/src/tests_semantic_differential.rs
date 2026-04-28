use gowasm_host_types::ErrorCategory;
use gowasm_vm::Vm;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::{compile_workspace, take_last_compile_error_context, SourceInput};

#[derive(Debug, Deserialize)]
struct DifferentialCorpusIndex {
    schema_version: u32,
    cases: Vec<DifferentialCase>,
}

#[derive(Debug, Deserialize)]
struct DifferentialCase {
    id: String,
    name: String,
    entry_path: String,
    workspace_files: Vec<String>,
    expected_native_go_stdout: String,
    #[serde(default)]
    expected_native_go_stderr: String,
    #[serde(default)]
    expected_native_go_exit_code: i32,
    #[serde(default)]
    expected_compiler_error_substring: Option<String>,
    #[serde(default)]
    expected_gowasm_outcome_category: Option<GowasmOutcomeCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GowasmOutcomeCategory {
    Success,
    CompileError,
    RuntimePanic,
    RuntimeTrap,
    RuntimeBudgetExhaustion,
    RuntimeDeadlock,
    RuntimeExit,
}

struct WorkspaceFileFixture {
    path: String,
    contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiagnosticLocation {
    path: String,
    line: usize,
    column: usize,
}

impl DifferentialCase {
    fn expected_gowasm_outcome_category(&self) -> GowasmOutcomeCategory {
        self.expected_gowasm_outcome_category.unwrap_or_else(|| {
            if self.expected_compiler_error_substring.is_some() {
                GowasmOutcomeCategory::CompileError
            } else {
                GowasmOutcomeCategory::Success
            }
        })
    }
}

#[test]
fn semantic_differential_corpus_matches_checked_in_native_go_outputs() {
    let index = load_corpus_index();
    assert_eq!(
        index.schema_version, 1,
        "unexpected semantic differential schema"
    );
    assert!(
        !index.cases.is_empty(),
        "semantic differential corpus should contain representative cases"
    );

    let case_ids = design_target_semantic_case_ids();
    let mut matched = 0usize;

    for case in index
        .cases
        .into_iter()
        .filter(|case| case_ids.contains(&case.id.as_str()))
    {
        matched += 1;
        run_case(case);
    }

    assert_eq!(
        matched,
        case_ids.len(),
        "semantic differential corpus should cover the full design-target category set"
    );
}

#[test]
fn assignment_list_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "assignment-list",
        &[
            "assignment_list_direct_values",
            "assignment_list_short_decl_too_few",
            "assignment_list_assign_too_few",
        ],
    );
}

#[test]
fn const_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "const",
        &[
            "const_make_bounds",
            "const_generic_args",
            "const_byte_overflow",
        ],
    );
}

#[test]
fn generic_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "generic",
        &[
            "generic_closure_values",
            "generic_inline_constraints",
            "generic_inline_constraint_rejection",
            "generic_recursive_inline_constraints",
        ],
    );
}

#[test]
fn interface_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "interface",
        &[
            "promoted_methods",
            "interface_nil_comparisons",
            "interface_method_set_satisfaction",
            "interface_method_set_failed_assignment",
            "interface_method_set_assertion_signature_mismatch",
            "type_assert_pointer_typed_nil",
            "type_switch_nil_cases",
            "type_switch_generic_interface",
        ],
    );
}

#[test]
fn method_value_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete("method-value", &["method_values_bound_interface_and_defer"]);
}

#[test]
fn package_init_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "package-init",
        &[
            "package_init_file_and_var_order",
            "package_init_function_dependency",
            "package_init_var_cycle",
        ],
    );
}

#[test]
fn map_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "map",
        &[
            "map_nil_lookup_delete_and_struct_keys",
            "map_index_address_rejection",
        ],
    );
}

#[test]
fn pointer_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "pointer",
        &[
            "addressability",
            "pointer_selectors",
            "addressable_pointer_receivers",
            "promoted_pointer_methods_interface",
            "address_of_temporary_index",
            "map_index_address_rejection",
        ],
    );
}

#[test]
fn slice_array_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "slice-array",
        &[
            "slice_bounds_capacity",
            "slice_copy_overlap",
            "slice_nil_empty",
            "slice_multidimensional_aliasing",
            "array_value_assignment",
        ],
    );
}

#[test]
fn string_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "string",
        &[
            "string_utf8_range_and_slice",
            "string_rune_conversions",
            "string_invalid_utf8_helpers",
            "string_formatting_bytes_and_quotes",
        ],
    );
}

#[test]
fn nil_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "nil",
        &["nil_cross_type_matrix", "typed_nil_interface_formatting"],
    );
}

#[test]
fn channel_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "channel",
        &[
            "channel_nil_and_directional_flow",
            "channel_blocking_handoff_and_close_wakeups",
            "channel_buffered_close_and_range",
            "channel_directional_rejection",
        ],
    );
}

#[test]
fn select_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "select",
        &[
            "select_dense_ready_rotation",
            "select_default_dense_ready_rotation",
            "select_all_nil_default",
            "select_close_wakeup_multi_case",
        ],
    );
}

#[test]
fn range_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "range",
        &[
            "short_decl_range_scope",
            "control_flow_range_supported_types",
        ],
    );
}

#[test]
fn switch_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "switch",
        &[
            "control_flow_fallthrough_limits",
            "control_flow_switch_scoping",
            "control_flow_type_switch_init",
            "type_switch_nil_cases",
            "type_switch_generic_interface",
        ],
    );
}

#[test]
fn defer_recover_semantic_differential_cases_match_native_go_outputs() {
    assert_case_ids_complete(
        "defer-recover",
        &[
            "named_result_defer",
            "closure_capture_mutation_and_defer",
            "unwind_panic_replacement",
            "unwind_recover_eligibility",
            "unwind_goroutine_local_recover",
        ],
    );
}

fn assert_case_ids_complete(label: &str, case_ids: &[&str]) {
    let index = load_corpus_index();
    let mut matched = 0usize;

    for case in index
        .cases
        .into_iter()
        .filter(|case| case_ids.contains(&case.id.as_str()))
    {
        matched += 1;
        run_case(case);
    }

    assert_eq!(
        matched,
        case_ids.len(),
        "{label} semantic differential coverage should stay complete"
    );
}

fn design_target_semantic_case_ids() -> &'static [&'static str] {
    &[
        "assignment_list_direct_values",
        "assignment_list_short_decl_too_few",
        "assignment_list_assign_too_few",
        "const_make_bounds",
        "const_generic_args",
        "const_byte_overflow",
        "generic_closure_values",
        "generic_inline_constraints",
        "generic_inline_constraint_rejection",
        "generic_recursive_inline_constraints",
        "promoted_methods",
        "interface_nil_comparisons",
        "interface_method_set_satisfaction",
        "interface_method_set_failed_assignment",
        "interface_method_set_assertion_signature_mismatch",
        "type_assert_pointer_typed_nil",
        "type_switch_nil_cases",
        "type_switch_generic_interface",
        "method_values_bound_interface_and_defer",
        "package_init_file_and_var_order",
        "package_init_function_dependency",
        "package_init_var_cycle",
        "map_nil_lookup_delete_and_struct_keys",
        "map_index_address_rejection",
        "addressability",
        "pointer_selectors",
        "addressable_pointer_receivers",
        "promoted_pointer_methods_interface",
        "address_of_temporary_index",
        "slice_bounds_capacity",
        "slice_copy_overlap",
        "slice_nil_empty",
        "slice_multidimensional_aliasing",
        "array_value_assignment",
        "string_utf8_range_and_slice",
        "string_rune_conversions",
        "string_invalid_utf8_helpers",
        "string_formatting_bytes_and_quotes",
        "nil_cross_type_matrix",
        "typed_nil_interface_formatting",
        "channel_nil_and_directional_flow",
        "channel_blocking_handoff_and_close_wakeups",
        "channel_buffered_close_and_range",
        "channel_directional_rejection",
        "select_dense_ready_rotation",
        "select_default_dense_ready_rotation",
        "select_all_nil_default",
        "select_close_wakeup_multi_case",
        "short_decl_range_scope",
        "control_flow_range_supported_types",
        "control_flow_fallthrough_limits",
        "control_flow_switch_scoping",
        "control_flow_type_switch_init",
        "named_result_defer",
        "closure_capture_mutation_and_defer",
        "unwind_panic_replacement",
        "unwind_recover_eligibility",
        "unwind_goroutine_local_recover",
    ]
}

fn load_corpus_index() -> DifferentialCorpusIndex {
    serde_json::from_str(
        &fs::read_to_string(corpus_root().join("index.json"))
            .expect("semantic differential corpus index should be readable"),
    )
    .expect("semantic differential corpus index should deserialize")
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/semantic-differential")
}

fn run_case(case: DifferentialCase) {
    let workspace_files = load_workspace_files(&case.id, &case.workspace_files);
    let source_inputs = workspace_files
        .iter()
        .filter(|file| file.path.ends_with(".go"))
        .map(|file| SourceInput {
            path: file.path.as_str(),
            source: file.contents.as_str(),
        })
        .collect::<Vec<_>>();

    let expected_outcome = case.expected_gowasm_outcome_category();
    match expected_outcome {
        GowasmOutcomeCategory::Success => assert_eq!(
            case.expected_native_go_exit_code, 0,
            "semantic differential case `{}` should record a zero native-Go exit code for success cases",
            case.name
        ),
        GowasmOutcomeCategory::CompileError => assert_ne!(
            case.expected_native_go_exit_code, 0,
            "semantic differential case `{}` should record a non-zero native-Go exit code for compile-error cases",
            case.name
        ),
        _ => {}
    }
    let compile_result = compile_workspace(&source_inputs, &case.entry_path);

    match compile_result {
        Err(error) => {
            assert_eq!(
                expected_outcome,
                GowasmOutcomeCategory::CompileError,
                "semantic differential case `{}` should compile and run, got compile error `{error}`",
                case.name
            );

            let expected_error = case
                .expected_compiler_error_substring
                .as_deref()
                .unwrap_or_else(|| {
                    panic!(
                        "semantic differential case `{}` should declare an expected compile error substring",
                        case.name
                    )
                });
            assert!(
                error.to_string().contains(expected_error),
                "semantic differential case `{}` should fail with `{expected_error}`, got `{error}`",
                case.name
            );

            if let (Some(expected_location), Some(context)) = (
                parse_first_native_go_diagnostic_location(&case.expected_native_go_stderr),
                take_last_compile_error_context(),
            ) {
                let actual_location = actual_diagnostic_location(
                    &workspace_files,
                    &context.file_path,
                    context.span_start,
                );
                assert_eq!(
                    normalize_path(&actual_location.path),
                    normalize_path(&expected_location.path),
                    "semantic differential case `{}` should report the same diagnostic path as native Go",
                    case.name
                );
            }
        }
        Ok(program) => {
            let mut vm = Vm::new();
            match vm.run_program(&program) {
                Ok(()) => {
                    assert_eq!(
                        expected_outcome,
                        GowasmOutcomeCategory::Success,
                        "semantic differential case `{}` should end with category `{:?}`, got success",
                        case.name,
                        expected_outcome
                    );
                    assert_eq!(
                        vm.stdout(),
                        case.expected_native_go_stdout,
                        "semantic differential case `{}` diverged from the checked-in native Go stdout",
                        case.name
                    );
                }
                Err(error) => {
                    let actual_outcome = gowasm_outcome_category_for_vm_error(&error);
                    assert_eq!(
                        actual_outcome,
                        expected_outcome,
                        "semantic differential case `{}` ended with unexpected VM outcome `{actual_outcome:?}`: {error}",
                        case.name
                    );
                    assert_eq!(
                        vm.stdout(),
                        case.expected_native_go_stdout,
                        "semantic differential case `{}` diverged from the checked-in native Go stdout before failing",
                        case.name
                    );
                }
            }
        }
    }
}

fn gowasm_outcome_category_for_vm_error(error: &gowasm_vm::VmError) -> GowasmOutcomeCategory {
    match error.category() {
        ErrorCategory::RuntimePanic => GowasmOutcomeCategory::RuntimePanic,
        ErrorCategory::RuntimeTrap => GowasmOutcomeCategory::RuntimeTrap,
        ErrorCategory::RuntimeBudgetExhaustion => GowasmOutcomeCategory::RuntimeBudgetExhaustion,
        ErrorCategory::RuntimeDeadlock => GowasmOutcomeCategory::RuntimeDeadlock,
        ErrorCategory::RuntimeExit => GowasmOutcomeCategory::RuntimeExit,
        other => panic!("unexpected VM error category for semantic differential test: {other:?}"),
    }
}

fn parse_first_native_go_diagnostic_location(stderr: &str) -> Option<DiagnosticLocation> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("note:") {
            continue;
        }
        let mut parts = trimmed.splitn(4, ':');
        let path = parts.next()?.trim();
        let line = parts.next()?.trim().parse().ok()?;
        let column = parts.next()?.trim().parse().ok()?;
        return Some(DiagnosticLocation {
            path: path.into(),
            line,
            column,
        });
    }
    None
}

fn actual_diagnostic_location(
    workspace_files: &[WorkspaceFileFixture],
    file_path: &str,
    span_start: usize,
) -> DiagnosticLocation {
    let contents = workspace_files
        .iter()
        .find(|file| normalize_path(&file.path) == normalize_path(file_path))
        .unwrap_or_else(|| {
            panic!(
                "semantic differential diagnostic file `{file_path}` should exist in the workspace"
            )
        })
        .contents
        .as_str();
    let (line, column) = line_and_column_for_offset(contents, span_start);
    DiagnosticLocation {
        path: file_path.into(),
        line,
        column,
    }
}

fn line_and_column_for_offset(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    let prefix = source.get(..offset).unwrap_or(source);
    for ch in prefix.chars() {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

fn normalize_path(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

fn load_workspace_files(case_id: &str, workspace_files: &[String]) -> Vec<WorkspaceFileFixture> {
    let workspace_root = corpus_root().join(case_id).join("workspace");
    let mut files = Vec::with_capacity(workspace_files.len());
    for path in workspace_files {
        let full_path = workspace_root.join(path);
        files.push(WorkspaceFileFixture {
            path: path.clone(),
            contents: fs::read_to_string(&full_path).unwrap_or_else(|_| {
                panic!(
                    "semantic differential workspace file `{}` should be readable",
                    full_path.display()
                )
            }),
        });
    }
    files
}
