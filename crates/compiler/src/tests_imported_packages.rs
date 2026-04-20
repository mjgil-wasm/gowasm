use crate::{compile_workspace, module_cache_source_path, SourceInput};
use gowasm_vm::Vm;

fn run_workspace(sources: &[SourceInput<'_>], entry_path: &str) -> String {
    let program = compile_workspace(sources, entry_path).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    vm.stdout().to_string()
}

#[test]
fn compiles_and_runs_local_imported_package_function_calls() {
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
        "main.go",
    );

    assert_eq!(output, "hello\n");
}

#[test]
fn compiles_and_runs_local_imported_package_global_selectors() {
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
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Label)
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

var Label = "hello"
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "hello\n");
}

#[test]
fn compiles_and_runs_imported_package_function_values() {
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
        "main.go",
    );

    assert_eq!(output, "hello\n");
}

#[test]
fn compiles_and_runs_imported_package_multi_result_calls() {
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
    "example.com/app/lib"
)

func main() {
    number, label := lib.Values()
    fmt.Println(number, label)
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

func Values() (int, string) {
    return 7, "days"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "7 days\n");
}

#[test]
fn compiles_and_runs_imported_package_go_and_defer_calls() {
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
    "example.com/app/lib"
)

func main() {
    defer lib.Cleanup()
    ch := make(chan string)
    go lib.Send(ch)
    fmt.Println(<-ch)
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

import "fmt"

func Cleanup() {
    fmt.Println("cleanup")
}

func Send(ch chan string) {
    ch <- "hello"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "hello\ncleanup\n");
}

#[test]
fn runs_imported_package_init_before_entry_main() {
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

var label = "hello"

func init() {
    label = "ready"
}

func Message() string {
    return label
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "ready\n");
}

#[test]
fn compiles_and_runs_remote_module_imported_function_calls() {
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

import (
    "fmt"
    "example.com/remote/greeter"
)

func main() {
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

func Message() string {
    return "hi"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "hi\n");
}

#[test]
fn compiles_and_runs_imported_named_types_and_methods() {
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
    "example.com/app/lib"
)

type speaker interface {
    Speak() string
}

func main() {
    item := lib.Make("ada")
    var value speaker = item
    fmt.Println(item.Name)
    fmt.Println(item.Speak())
    fmt.Println(value.Speak())
    fmt.Println(lib.Wrap(lib.Item{Name: "lin"}))
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Item struct {
    Name string
}

func Make(name string) Item {
    return Item{Name: name}
}

func Wrap(item Item) string {
    return item.Speak()
}

func (item Item) Speak() string {
    return "hi:" + item.Name
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "ada\nhi:ada\nhi:ada\nhi:lin\n");
}

#[test]
fn compiles_and_runs_remote_module_imported_named_types_and_methods() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "cards/cards.go");
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
    "example.com/app/report"
)

func main() {
    card := report.First()
    fmt.Println(card.Label)
    fmt.Println(card.Message())
}
"#,
            },
            SourceInput {
                path: "report/report.go",
                source: r#"
package report

import "example.com/remote/cards"

func First() cards.Card {
    return cards.Card{Label: "Ada"}
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
package cards

type Card struct {
    Label string
}

func (card Card) Message() string {
    return "remote:" + card.Label
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "Ada\nremote:Ada\n");
}

#[test]
fn compiles_and_runs_direct_local_imported_generic_type_instantiations() {
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
    "example.com/app/lib"
)

type speaker interface {
    Speak() string
}

func main() {
    box := lib.Box[string]{Label: "Ada"}
    var value speaker = box
    fmt.Println(box.Label)
    fmt.Println(box.Speak())
    fmt.Println(value.Speak())
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Box[T any] struct {
    Label T
}

func (box Box[T]) Speak() string {
    return "local-template"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "Ada\nlocal-template\nlocal-template\n");
}

#[test]
fn compiles_and_runs_imported_instantiated_generic_types_and_methods() {
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
    "example.com/app/lib"
)

type speaker interface {
    Speak() string
}

func main() {
    box := lib.First()
    var value speaker = box
    fmt.Println(box.Label)
    fmt.Println(box.Speak())
    fmt.Println(value.Speak())
}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

type Box[T any] struct {
    Label T
}

func First() Box[string] {
    return Box[string]{Label: "Ada"}
}

func (box Box[T]) Speak() string {
    return "local-box"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "Ada\nlocal-box\nlocal-box\n");
}

#[test]
fn compiles_and_runs_imported_generic_function_calls() {
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
        "main.go",
    );

    assert_eq!(output, "lib:7\nlib:go\n");
}

#[test]
fn compiles_and_runs_remote_module_imported_instantiated_generic_types_and_methods() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "cards/cards.go");
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
    "example.com/app/report"
)

type speaker interface {
    Speak() string
}

func main() {
    box := report.First()
    var value speaker = box
    fmt.Println(box.Label)
    fmt.Println(box.Speak())
    fmt.Println(value.Speak())
}
"#,
            },
            SourceInput {
                path: "report/report.go",
                source: r#"
package report

import "example.com/remote/cards"

func First() cards.Box[string] {
    return cards.Box[string]{Label: "Ada"}
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
package cards

type Box[T any] struct {
    Label T
}

func (box Box[T]) Speak() string {
    return "remote-box"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "Ada\nremote-box\nremote-box\n");
}

#[test]
fn compiles_and_runs_direct_remote_module_imported_generic_type_instantiations() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "cards/cards.go");
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
    "example.com/remote/cards"
)

type speaker interface {
    Speak() string
}

func main() {
    box := cards.Box[string]{Label: "Ada"}
    var value speaker = box
    fmt.Println(box.Label)
    fmt.Println(box.Speak())
    fmt.Println(value.Speak())
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
package cards

type Box[T any] struct {
    Label T
}

func (box Box[T]) Speak() string {
    return "remote-template"
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "Ada\nremote-template\nremote-template\n");
}

#[test]
fn compiles_and_runs_remote_module_imported_generic_function_calls() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "convert/convert.go");
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
    "example.com/remote/convert"
)

func main() {
    fmt.Println(convert.Label(9))
    fmt.Println(convert.Label("hex"))
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
package convert

import "fmt"

func Label[T any](value T) string {
    return "remote:" + fmt.Sprint(value)
}
"#,
            },
        ],
        "main.go",
    );

    assert_eq!(output, "remote:9\nremote:hex\n");
}
