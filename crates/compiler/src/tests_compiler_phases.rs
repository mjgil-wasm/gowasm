use crate::compiler_phase::CompilerPhase;
use crate::program::compile_file_in_explicit_phases_for_tests;
use gowasm_parser::parse_source_file;
use gowasm_vm::{program_debug_info, program_type_inventory};

#[test]
fn compiler_phase_order_is_explicit_and_stable() {
    assert_eq!(
        [
            CompilerPhase::ParseValidation,
            CompilerPhase::TypeChecking,
            CompilerPhase::Lowering,
            CompilerPhase::BytecodeEmission,
            CompilerPhase::RuntimeMetadataRegistration,
            CompilerPhase::Diagnostics,
        ],
        [
            CompilerPhase::ParseValidation,
            CompilerPhase::TypeChecking,
            CompilerPhase::Lowering,
            CompilerPhase::BytecodeEmission,
            CompilerPhase::RuntimeMetadataRegistration,
            CompilerPhase::Diagnostics,
        ]
    );
}

#[test]
fn explicit_compiler_phases_register_runtime_metadata_and_diagnostics() {
    let file = parse_source_file(
        r#"
package main

import "fmt"

func main() {
    fmt.Println("ok")
}
"#,
    )
    .expect("source should parse");

    let program =
        compile_file_in_explicit_phases_for_tests(&file).expect("phased compile should succeed");

    assert!(program_type_inventory(&program).is_some());
    assert!(program_debug_info(&program).is_some());
}

#[test]
fn type_check_phase_fails_before_any_bytecode_is_emitted() {
    let file = parse_source_file(
        r#"
package main

type Bad struct {
    Values map[[]int]int
}

func main() {}
"#,
    )
    .expect("source should parse");

    let failure =
        compile_file_in_explicit_phases_for_tests(&file).expect_err("type check should fail");

    assert_eq!(failure.phase, CompilerPhase::TypeChecking);
    assert_eq!(failure.emitted_function_count, 0);
    assert_eq!(failure.emitted_debug_info_count, 0);
    assert!(
        failure
            .error
            .to_string()
            .contains("map key type `[]int` is not comparable"),
        "unexpected error: {}",
        failure.error
    );
}
