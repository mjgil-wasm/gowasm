use crate::{compile_workspace, SourceInput};
use gowasm_vm::Vm;

struct ReleaseGateCase<'a> {
    name: &'a str,
    entry_path: &'a str,
    sources: Vec<SourceInput<'a>>,
    expected_stdout: &'a str,
}

fn run_workspace(sources: &[SourceInput<'_>], entry_path: &str) -> String {
    let program = compile_workspace(sources, entry_path).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    vm.stdout().to_string()
}

#[test]
fn imported_package_release_gate_cases_pass_through_direct_compiler_and_vm_execution() {
    let cases = vec![
        ReleaseGateCase {
            name: "local imported package function call",
            entry_path: "main.go",
            sources: vec![
                SourceInput {
                    path: "go.mod",
                    source: "module example.com/app\n\ngo 1.21\n",
                },
                SourceInput {
                    path: "main.go",
                    source: r#"
package main

import (
    "fmt"
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message())
}
"#,
                },
                SourceInput {
                    path: "lib/lib.go",
                    source: r#"
package lib

func Message() string {
    return "hello"
}
"#,
                },
            ],
            expected_stdout: "hello\n",
        },
        ReleaseGateCase {
            name: "local imported generic function call",
            entry_path: "main.go",
            sources: vec![
                SourceInput {
                    path: "go.mod",
                    source: "module example.com/app\n\ngo 1.21\n",
                },
                SourceInput {
                    path: "main.go",
                    source: r#"
package main

import (
    "fmt"
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Describe[int](7))
    fmt.Println(lib.Describe("go"))
}
"#,
                },
                SourceInput {
                    path: "lib/lib.go",
                    source: r#"
package lib

import "fmt"

var prefix = "lib:"

func decorate(text string) string {
    return prefix + text
}

func Describe[T any](value T) string {
    return decorate(fmt.Sprint(value))
}
"#,
                },
            ],
            expected_stdout: "lib:7\nlib:go\n",
        },
        ReleaseGateCase {
            name: "imported package function values",
            entry_path: "main.go",
            sources: vec![
                SourceInput {
                    path: "go.mod",
                    source: "module example.com/app\n\ngo 1.21\n",
                },
                SourceInput {
                    path: "main.go",
                    source: r#"
package main

import (
    "fmt"
    "example.com/app/lib"
)

func main() {
    callback := lib.Message
    fmt.Println(callback())
}
"#,
                },
                SourceInput {
                    path: "lib/lib.go",
                    source: r#"
package lib

func Message() string {
    return "hello"
}
"#,
                },
            ],
            expected_stdout: "hello\n",
        },
    ];

    for case in cases {
        let output = run_workspace(&case.sources, case.entry_path);
        assert_eq!(
            output, case.expected_stdout,
            "imported-package release gate case `{}` produced unexpected stdout",
            case.name
        );
    }
}
