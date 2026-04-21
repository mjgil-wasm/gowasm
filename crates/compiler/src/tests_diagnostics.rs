use super::{compile_source, compile_workspace, SourceInput};
use gowasm_vm::program_debug_info;

fn span_text(source: &str, start: usize, end: usize) -> &str {
    &source[start..end]
}

#[test]
fn preserves_instruction_spans_for_top_level_functions_and_closures() {
    let source = r#"
package main

func main() {
    value := 1
    bump := func() {
        value = value + 1
    }
    bump()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let debug_info =
        program_debug_info(&program).expect("compiled program should register debug info");

    let main_index = program
        .functions
        .iter()
        .position(|function| function.name == "main")
        .expect("main function should exist");
    let closure_index = program
        .functions
        .iter()
        .position(|function| function.name.starts_with("__gowasm_closure$"))
        .expect("generated closure function should exist");

    let main_debug = &debug_info.functions[main_index];
    assert_eq!(
        main_debug.instruction_spans.len(),
        program.functions[main_index].code.len()
    );
    assert!(main_debug.instruction_spans.iter().flatten().any(|span| {
        span.path == "main.go" && span_text(source, span.start, span.end).contains("value := 1")
    }));
    assert!(main_debug.instruction_spans.iter().flatten().any(|span| {
        span.path == "main.go" && span_text(source, span.start, span.end).contains("bump()")
    }));

    let closure_debug = &debug_info.functions[closure_index];
    assert_eq!(
        closure_debug.instruction_spans.len(),
        program.functions[closure_index].code.len()
    );
    assert!(closure_debug
        .instruction_spans
        .iter()
        .flatten()
        .any(|span| {
            span.path == "main.go"
                && span_text(source, span.start, span.end).contains("value = value + 1")
        }));
}

#[test]
fn preserves_instruction_spans_for_generated_package_init_code() {
    let main_source = r#"
package main

func main() {}
"#;
    let globals_source = r#"
package main

var label = "hello"
"#;

    let program = compile_workspace(
        &[
            SourceInput {
                path: "main.go",
                source: main_source,
            },
            SourceInput {
                path: "globals.go",
                source: globals_source,
            },
        ],
        "main.go",
    )
    .expect("workspace should compile");
    let debug_info =
        program_debug_info(&program).expect("compiled program should register debug info");

    let init_index = program
        .functions
        .iter()
        .position(|function| function.name == "__gowasm_init")
        .expect("package init function should exist");
    let init_debug = &debug_info.functions[init_index];
    assert_eq!(
        init_debug.instruction_spans.len(),
        program.functions[init_index].code.len()
    );
    assert!(init_debug.instruction_spans.iter().flatten().any(|span| {
        span.path == "globals.go"
            && span_text(globals_source, span.start, span.end).contains("var label = \"hello\"")
    }));
}

#[test]
fn preserves_instruction_spans_for_generic_function_instances() {
    let source = r#"
package main
import "fmt"

func id[T any](value T) T {
    copied := value
    return copied
}

func main() {
    fmt.Println(id[int](1))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let debug_info =
        program_debug_info(&program).expect("compiled program should register debug info");

    let generic_index = program
        .functions
        .iter()
        .position(|function| function.name == "id[int]")
        .expect("instantiated generic function should exist");
    let generic_debug = &debug_info.functions[generic_index];
    assert_eq!(
        generic_debug.instruction_spans.len(),
        program.functions[generic_index].code.len()
    );
    assert!(generic_debug
        .instruction_spans
        .iter()
        .flatten()
        .any(|span| {
            span.path == "main.go"
                && span_text(source, span.start, span.end).contains("copied := value")
        }));
    assert!(generic_debug
        .instruction_spans
        .iter()
        .flatten()
        .any(|span| {
            span.path == "main.go"
                && span_text(source, span.start, span.end).contains("return copied")
        }));
}
