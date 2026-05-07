use crate::{compile_workspace, module_cache_source_path, SourceInput};
use gowasm_vm::Vm;

fn run_workspace(sources: &[SourceInput<'_>], entry_path: &str) -> String {
    let program = compile_workspace(sources, entry_path).expect("workspace should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("workspace should run");
    vm.stdout().to_string()
}

#[test]
fn compiles_and_runs_package_var_initialization_in_dependency_order_across_files() {
    let output = run_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "fmt"

func main() {
    fmt.Println(first, second, third)
}
"#,
            },
            SourceInput {
                path: "z_values.go",
                source: r#"
package main

var third = second + 1
"#,
            },
            SourceInput {
                path: "m_values.go",
                source: r#"
package main

var second = first + 1
"#,
            },
            SourceInput {
                path: "a_values.go",
                source: r#"
package main

var first = 40
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "40 41 42\n");
}

#[test]
fn compiles_and_runs_package_var_initialization_from_called_function_dependencies() {
    let program = crate::compile_source(
        r#"
package main
import "fmt"

var second = readFirst()
var first = 41

func readFirst() int {
    return first + 1
}

func main() {
    fmt.Println(first, second)
}
"#,
    )
    .expect("program should compile");

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "41 42\n");
}

#[test]
fn rejects_package_var_initialization_cycles() {
    let error = crate::compile_source(
        r#"
package main

var first = second + 1
var second = first + 1

func main() {}
"#,
    )
    .expect_err("program should not compile");

    assert!(error
        .to_string()
        .contains("package initialization cycle involving first, second"));
}

#[test]
fn compiles_and_runs_local_import_dag_init_order() {
    let output = run_workspace(
        &[
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
    "example.com/app/left"
    "example.com/app/right"
)

func init() {
    fmt.Println("main")
}

func main() {
    fmt.Println(left.Message(), right.Message())
}
"#,
            },
            SourceInput {
                path: "left/left.go",
                source: r#"
package left

import (
    "fmt"
    "example.com/app/shared"
)

func init() {
    fmt.Println("left")
}

func Message() string {
    return "left-" + shared.Message()
}
"#,
            },
            SourceInput {
                path: "right/right.go",
                source: r#"
package right

import (
    "fmt"
    "example.com/app/shared"
)

func init() {
    fmt.Println("right")
}

func Message() string {
    return "right-" + shared.Message()
}
"#,
            },
            SourceInput {
                path: "shared/shared.go",
                source: r#"
package shared

import "fmt"

func init() {
    fmt.Println("shared")
}

func Message() string {
    return "ready"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(
        output,
        "shared\nleft\nright\nmain\nleft-ready right-ready\n"
    );
}

#[test]
fn compiles_and_runs_remote_module_init_before_local_package_init() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");
    let output = run_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/lib"

func main() {
    lib.Run()
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

import (
    "fmt"
    "example.com/remote/greeter"
)

func init() {
    fmt.Println("local")
}

func Run() {
    fmt.Println(greeter.Message())
}
"#,
            },
            SourceInput {
                path: module_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: module_go_file_path.as_str(),
                source: r#"
package greeter

import "fmt"

func init() {
    fmt.Println("remote")
}

func Message() string {
    return "done"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "remote\nlocal\ndone\n");
}
