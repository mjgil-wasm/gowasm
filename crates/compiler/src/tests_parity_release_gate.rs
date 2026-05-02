use crate::{compile_workspace, module_cache_source_path, SourceInput};
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
fn parity_release_gate_cases_pass_through_direct_compiler_and_vm_execution() {
    let remote_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let remote_greeter_go_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");

    let cases = vec![
        ReleaseGateCase {
            name: "local package chain summary",
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
    "example.com/app/report"
)

func main() {
    fmt.Println(report.Build())
}
"#,
                },
                SourceInput {
                    path: "report/report.go",
                    source: r#"
package report

import (
    "fmt"
    "example.com/app/catalog"
)

func Build() string {
    items := catalog.Items()
    return fmt.Sprintf("%s %s %d %t", catalog.Title(), items[0], len(items), catalog.Ready())
}
"#,
                },
                SourceInput {
                    path: "catalog/catalog.go",
                    source: r#"
package catalog

import "example.com/app/seed"

func Title() string {
    return "local-imports"
}

func Items() []string {
    return []string{seed.First(), "beta"}
}

func Ready() bool {
    return seed.Ready()
}
"#,
                },
                SourceInput {
                    path: "seed/seed.go",
                    source: r#"
package seed

var ready bool

func init() {
    ready = true
}

func First() string {
    return "alpha"
}

func Ready() bool {
    return ready
}
"#,
                },
            ],
            expected_stdout: "local-imports alpha 2 true\n",
        },
        ReleaseGateCase {
            name: "mixed local and remote chain summary",
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
    "example.com/app/report"
)

func main() {
    fmt.Println(report.Build())
}
"#,
                },
                SourceInput {
                    path: "report/report.go",
                    source: r#"
package report

import (
    "fmt"
    "example.com/app/names"
)

func Build() string {
    values := names.All()
    return fmt.Sprintf("%s,%s,%d", values[0], values[1], len(values))
}
"#,
                },
                SourceInput {
                    path: "names/names.go",
                    source: r#"
package names

import "example.com/remote/greeter"

func All() []string {
    return []string{
        greeter.Message("Ada"),
        greeter.Message("Lin"),
    }
}
"#,
                },
                SourceInput {
                    path: remote_go_mod_path.as_str(),
                    source: "module example.com/remote\n\ngo 1.21\n",
                },
                SourceInput {
                    path: remote_greeter_go_path.as_str(),
                    source: r#"
package greeter

func Message(name string) string {
    return "hello:" + name
}
"#,
                },
            ],
            expected_stdout: "hello:Ada,hello:Lin,2\n",
        },
    ];

    for case in cases {
        let output = run_workspace(&case.sources, case.entry_path);
        assert_eq!(
            output, case.expected_stdout,
            "parity release gate case `{}` produced unexpected stdout",
            case.name
        );
    }
}
