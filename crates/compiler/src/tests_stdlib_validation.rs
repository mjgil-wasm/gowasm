use super::compile_source;

#[test]
fn rejects_stdlib_calls_with_the_wrong_arity() {
    let source = r#"
package main
import "strings"

func main() {
    strings.Contains("go")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("`strings.Contains` expects 2 argument(s), found 1"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_scalar_argument_type() {
    let source = r#"
package main
import "strconv"

func main() {
    strconv.Itoa("go")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `strconv.Itoa` has type `string`, expected `int`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_collection_argument_type() {
    let source = r#"
package main
import "strings"

func main() {
    strings.Join("go", ",")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `strings.Join` has type `string`, expected `[]string`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_equal_fold_argument_type() {
    let source = r#"
package main
import "strings"

func main() {
    strings.EqualFold("go", 5)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 2 to `strings.EqualFold` has type `int`, expected `string`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_split_n_argument_type() {
    let source = r#"
package main
import "strings"

func main() {
    strings.SplitN("go", ",", false)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 3 to `strings.SplitN` has type `bool`, expected `int`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_split_after_n_argument_type() {
    let source = r#"
package main
import "strings"

func main() {
    strings.SplitAfterN("go", ",", false)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 3 to `strings.SplitAfterN` has type `bool`, expected `int`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_path_argument_type() {
    let source = r#"
package main
import "path"

func main() {
    path.Clean(5)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `path.Clean` has type `int`, expected `string`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_error_join_argument_type() {
    let source = r#"
package main
import "errors"

func main() {
    errors.Join("go")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `errors.Join` has type `string`, expected `error`"));
}

#[test]
fn rejects_multi_result_path_calls_in_single_value_positions() {
    let source = r#"
package main
import "path"

func main() {
    value := path.Split("a/b")
    _ = value
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("`path.Split` cannot be used in value position"));
}

#[test]
fn rejects_non_constant_stdlib_values_in_const_initializers() {
    let source = r#"
package main
import "net/http"

const defaultClient = http.DefaultClient

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("const initializers currently require supported constant expressions"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_variadic_argument_type() {
    let source = r#"
package main
import "path"

func main() {
    path.Join("a", 5, "b")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 2 to `path.Join` has type `int`, expected `string`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_path_match_argument_type() {
    let source = r#"
package main
import "path"

func main() {
    path.Match("a*", 5)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 2 to `path.Match` has type `int`, expected `string`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_sort_argument_type() {
    let source = r#"
package main
import "sort"

func main() {
    sort.SearchInts([]string{"a"}, 1)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `sort.SearchInts` has type `[]string`, expected `[]int`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_unicode_argument_type() {
    let source = r#"
package main
import "unicode"

func main() {
    unicode.IsDigit("7")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to `unicode.IsDigit` has type `string`, expected `int`"));
}

#[test]
fn rejects_stdlib_calls_with_the_wrong_unicode_to_argument_type() {
    let source = r#"
package main
import "unicode"

func main() {
    unicode.To(0, "A")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 2 to `unicode.To` has type `string`, expected `int`"));
}

#[test]
fn rejects_variadic_stdlib_function_values() {
    let source = r#"
package main
import "path"

func main() {
    join := path.Join
    _ = join
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("package selector `path.Join` cannot be used in value position"));
}

#[test]
fn rejects_variadic_stdlib_function_values_in_typed_bindings() {
    let source = r#"
package main
import "path"

var join func(string, string) string = path.Join

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("package selector `path.Join` cannot be used in value position"));
}

#[test]
fn rejects_variadic_stdlib_function_values_through_typed_params() {
    let source = r#"
package main
import "path"

func install(run func(string, string) string) {}

func main() {
    install(path.Join)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("package selector `path.Join` cannot be used in value position"));
}

#[test]
fn rejects_variadic_stdlib_function_values_through_typed_returns() {
    let source = r#"
package main
import "path"

func chooser() func(string, string) string {
    return path.Join
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("package selector `path.Join` cannot be used in value position"));
}

#[test]
fn rejects_mismatched_typed_stdlib_function_value_bindings() {
    let source = r#"
package main
import "path"

var clean func(int) string = path.Clean

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) string` is not assignable to `func(int) string` in the current subset",
    ));
}

#[test]
fn rejects_mismatched_typed_stdlib_function_value_params() {
    let source = r#"
package main
import "path"

func apply(run func(int) string) {}

func main() {
    apply(path.Clean)
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) string` is not assignable to `func(int) string` in the current subset",
    ));
}

#[test]
fn rejects_mismatched_typed_stdlib_function_value_returns() {
    let source = r#"
package main
import "strconv"

func parser() func(string) int {
    return strconv.Atoi
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) (int, error)` is not assignable to `func(string) int` in the current subset",
    ));
}

#[test]
fn rejects_mismatched_reassigned_stdlib_function_values() {
    let source = r#"
package main
import "path"

func main() {
    var clean func(int) string
    clean = path.Clean
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) string` is not assignable to `func(int) string` in the current subset",
    ));
}

#[test]
fn rejects_mismatched_package_reassigned_stdlib_function_values() {
    let source = r#"
package main
import "path"

var clean func(int) string

func init() {
    clean = path.Clean
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) string` is not assignable to `func(int) string` in the current subset",
    ));
}

#[test]
fn rejects_mismatched_package_stdlib_function_values_through_helpers() {
    let source = r#"
package main
import "path"

var clean func(int) string

func install(run func(int) string) {
    clean = run
}

func init() {
    install(path.Clean)
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error.to_string().contains(
        "function value of type `func(string) string` is not assignable to `func(int) string` in the current subset",
    ));
}
