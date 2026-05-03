use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_errors_join_with_nil_elision() {
    let source = r#"
package main
import "errors"
import "fmt"

func main() {
    first := errors.New("first")
    second := errors.New("second")
    fmt.Println(errors.Join() == nil)
    fmt.Println(errors.Join(nil) == nil)
    joined := errors.Join(first, nil, second)
    fmt.Println(joined != nil)
    fmt.Println(joined)
    fmt.Println(joined.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue\ntrue\nfirst\nsecond\nfirst\nsecond\n"
    );
}

#[test]
fn compiles_and_runs_errorf_with_wrap() {
    let source = r#"
package main
import "errors"
import "fmt"

func main() {
    base := errors.New("not found")
    wrapped := fmt.Errorf("load failed: %w", base)
    fmt.Println(wrapped)
    fmt.Println(errors.Is(wrapped, base))
    fmt.Println(errors.Unwrap(wrapped))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "load failed: not found\ntrue\nnot found\n");
}

#[test]
fn compiles_and_runs_errors_is_chain() {
    let source = r#"
package main
import "errors"
import "fmt"

func main() {
    root := errors.New("root cause")
    mid := fmt.Errorf("middle: %w", root)
    top := fmt.Errorf("top: %w", mid)
    fmt.Println(errors.Is(top, root))
    fmt.Println(errors.Is(top, mid))
    fmt.Println(errors.Is(top, errors.New("other")))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\nfalse\n");
}

#[test]
fn compiles_and_runs_errors_is_with_os_path_errors() {
    let source = r#"
package main
import "errors"
import "fmt"
import "os"

func main() {
    _, missingErr := os.ReadFile("missing")
    wrappedMissing := fmt.Errorf("wrapped: %w", missingErr)

    fsys := os.DirFS(".")
    file, _ := fsys.Open("assets/a.txt")
    _ = file.Close()
    closedErr := file.Close()

    buf := make([]byte, 1)
    _, readClosedErr := file.Read(buf)

    fmt.Println(errors.Is(missingErr, os.ErrNotExist))
    fmt.Println(errors.Is(wrappedMissing, os.ErrNotExist))
    fmt.Println(errors.Is(closedErr, os.ErrClosed))
    fmt.Println(errors.Is(readClosedErr, os.ErrClosed))
    fmt.Println(errors.Is(missingErr, os.ErrClosed))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\ntrue\ntrue\nfalse\n");
}

#[test]
fn compiles_and_runs_errorf_without_wrap() {
    let source = r#"
package main
import "errors"
import "fmt"

func main() {
    err := fmt.Errorf("code %d: %s", 404, "not found")
    fmt.Println(err)
    fmt.Println(errors.Unwrap(err) == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "code 404: not found\ntrue\n");
}

#[test]
fn compiles_and_runs_errors_join_multi_wrap_and_as() {
    let source = r#"
package main
import "errors"
import "fmt"

type Problem struct {
    Code int
}

func (p Problem) Error() string {
    return fmt.Sprintf("problem:%d", p.Code)
}

func main() {
    first := errors.New("first")
    second := errors.New("second")
    joined := errors.Join(first, nil, second)
    multi := fmt.Errorf("multi: %w + %w", first, second)
    wrapped := fmt.Errorf("outer: %w", Problem{Code: 7})

    var problem Problem
    var top error

    fmt.Println(joined)
    fmt.Println(errors.Is(joined, first), errors.Is(joined, second), errors.Unwrap(joined) == nil)
    fmt.Println(multi)
    fmt.Println(errors.Is(multi, first), errors.Is(multi, second), errors.Unwrap(multi) == nil)
    fmt.Println(errors.As(wrapped, &problem), problem.Code)
    fmt.Println(errors.As(wrapped, &top), top)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "first\nsecond\ntrue true true\nmulti: first + second\ntrue true true\ntrue 7\ntrue outer: problem:7\n"
    );
}

#[test]
fn errors_as_rejects_non_pointer_targets() {
    let source = r#"
package main
import "errors"

func main() {
    err := errors.New("boom")
    _ = errors.As(err, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("errors: target must be a non-nil pointer"));
}
