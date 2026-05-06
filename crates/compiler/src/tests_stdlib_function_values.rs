use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_single_result_stdlib_function_values() {
    let source = r#"
package main
import "fmt"
import "path"
import "unicode"

var clean = path.Clean

func main() {
    isDigit := unicode.IsDigit
    fmt.Println(isDigit(55), isDigit(65), clean("/a/../b"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false /b\n");
}

#[test]
fn compiles_and_runs_multi_result_stdlib_function_values() {
    let source = r#"
package main
import "fmt"
import "strconv"

var atoi = strconv.Atoi

func main() {
    value, err := atoi("42")
    bad, badErr := atoi("go")
    fmt.Println(value, err == nil, bad, badErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true 0 true\n");
}

#[test]
fn compiles_and_runs_typed_stdlib_function_value_bindings() {
    let source = r#"
package main
import "fmt"
import "path"
import "unicode"

var clean func(string) string = path.Clean

func main() {
    var isDigit func(int) bool = unicode.IsDigit
    fmt.Println(isDigit(57), isDigit(65), clean("a/../b"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false b\n");
}

#[test]
fn compiles_and_runs_local_multi_result_stdlib_function_values() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    atoi := strconv.Atoi
    good, goodErr := atoi("42")
    bad, badErr := atoi("go")
    fmt.Println(good, goodErr == nil, bad, badErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true 0 true\n");
}

#[test]
fn compiles_and_runs_returned_stdlib_function_values() {
    let source = r#"
package main
import "fmt"
import "path"
import "strconv"

func cleaner() func(string) string {
    return path.Clean
}

func parser() func(string) (int, error) {
    return strconv.Atoi
}

func main() {
    clean := cleaner()
    atoi := parser()
    value, err := atoi("42")
    fmt.Println(clean("a/../b"), value, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b 42 true\n");
}

#[test]
fn compiles_and_runs_stdlib_function_values_through_typed_params() {
    let source = r#"
package main
import "fmt"
import "path"
import "strconv"

func apply(run func(string) string, value string) string {
    return run(value)
}

func parse(run func(string) (int, error), value string) (int, error) {
    return run(value)
}

func main() {
    clean := path.Clean
    atoi := strconv.Atoi
    value, err := parse(atoi, "42")
    fmt.Println(apply(clean, "a/../b"), value, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b 42 true\n");
}

#[test]
fn compiles_and_runs_stdlib_function_values_through_existing_bindings() {
    let source = r#"
package main
import "fmt"
import "path"
import "strconv"

func main() {
    var clean func(string) string
    clean = path.Clean
    var atoi func(string) (int, error)
    atoi = strconv.Atoi
    value, err := atoi("42")
    fmt.Println(clean("a/../b"), value, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b 42 true\n");
}

#[test]
fn compiles_and_runs_stdlib_function_value_nil_transitions() {
    let source = r#"
package main
import "fmt"
import "path"
import "unicode"

var clean func(string) string

func main() {
    var isDigit func(int) bool
    fmt.Println(clean == nil, isDigit == nil)
    clean = path.Clean
    isDigit = unicode.IsDigit
    fmt.Println(clean == nil, clean != nil, isDigit == nil, isDigit != nil)
    fmt.Println(clean("a/../b"), isDigit(57), isDigit(65))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\nfalse true false true\nb true false\n"
    );
}

#[test]
fn compiles_and_runs_rebound_stdlib_function_values_through_typed_wrappers() {
    let source = r#"
package main
import "fmt"
import "path"
import "strconv"

func bounceClean(run func(string) string) func(string) string {
    return run
}

func bounceParse(run func(string) (int, error)) func(string) (int, error) {
    return run
}

func main() {
    var clean func(string) string
    clean = path.Clean
    clean = bounceClean(clean)
    var atoi func(string) (int, error)
    atoi = strconv.Atoi
    atoi = bounceParse(atoi)
    value, err := atoi("42")
    fmt.Println(clean("a/../b"), value, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b 42 true\n");
}

#[test]
fn compiles_and_runs_package_stdlib_function_value_rebinding_in_init() {
    let source = r#"
package main
import "fmt"
import "path"
import "strconv"

var clean func(string) string = path.Clean
var atoi func(string) (int, error) = strconv.Atoi

func bounceClean(run func(string) string) func(string) string {
    return run
}

func bounceParse(run func(string) (int, error)) func(string) (int, error) {
    return run
}

func init() {
    clean = bounceClean(clean)
    atoi = bounceParse(atoi)
}

func main() {
    value, err := atoi("42")
    fmt.Println(clean("a/../b"), value, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b 42 true\n");
}

#[test]
fn compiles_and_runs_package_stdlib_function_values_through_typed_helpers() {
    let source = r#"
package main
import "fmt"
import "path"
import "strconv"

var clean func(string) string
var atoi func(string) (int, error)

func installClean(run func(string) string) {
    clean = run
}

func installParse(run func(string) (int, error)) {
    atoi = run
}

func currentClean() func(string) string {
    return clean
}

func currentParse() func(string) (int, error) {
    return atoi
}

func init() {
    installClean(path.Clean)
    installParse(strconv.Atoi)
}

func main() {
    value, err := currentParse()("42")
    fmt.Println(currentClean()("a/../b"), value, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b 42 true\n");
}

#[test]
fn compiles_and_runs_package_stdlib_multi_result_error_flows() {
    let source = r#"
package main
import "fmt"
import "strconv"

var atoi func(string) (int, error)

func install(run func(string) (int, error)) {
    atoi = run
}

func current() func(string) (int, error) {
    return atoi
}

func init() {
    install(strconv.Atoi)
}

func main() {
    good, goodErr := current()("42")
    bad, badErr := current()("go")
    fmt.Println(good, goodErr == nil, bad, badErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "42 true 0 true\n");
}

#[test]
fn compiles_and_runs_stdlib_function_values_with_alias_backed_map_results() {
    let source = r#"
package main
import "fmt"
import "net/url"

var parse func(string) (url.Values, error) = url.ParseQuery

func main() {
    values, err := parse("a=1&a=two+words")
    fmt.Println(values.Get("a"), err == nil, values.Encode())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 true a=1&a=two+words\n");
}
