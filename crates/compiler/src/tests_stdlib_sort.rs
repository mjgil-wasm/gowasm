use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_sort_queries() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    ints := []int{1, 2, 4, 4, 9}
    mixedInts := []int{3, 1}
    words := []string{"ant", "bee", "cat"}
    mixedWords := []string{"bee", "ant"}
    fmt.Println(
        sort.IntsAreSorted(ints),
        sort.IntsAreSorted(mixedInts),
        sort.SearchInts(ints, 4),
        sort.SearchInts(ints, 5),
        sort.StringsAreSorted(words),
        sort.StringsAreSorted(mixedWords),
        sort.SearchStrings(words, "bee"),
        sort.SearchStrings(words, "cow"),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false 2 4 true false 1 3\n");
}

#[test]
fn compiles_and_runs_sort_nil_slice_queries() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    var ints []int
    var words []string
    fmt.Println(
        sort.IntsAreSorted(ints),
        sort.StringsAreSorted(words),
        sort.SearchInts(ints, 7),
        sort.SearchStrings(words, "go"),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true 0 0\n");
}

#[test]
fn compiles_and_runs_sort_ints() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    nums := []int{5, 3, 1, 4, 2}
    sort.Ints(nums)
    fmt.Println(nums)
    fmt.Println(sort.IntsAreSorted(nums))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3 4 5]\ntrue\n");
}

#[test]
fn compiles_and_runs_sort_strings() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    words := []string{"cherry", "apple", "banana"}
    sort.Strings(words)
    fmt.Println(words)
    fmt.Println(sort.StringsAreSorted(words))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[apple banana cherry]\ntrue\n");
}

#[test]
fn compiles_and_runs_sort_float64s() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    vals := []float64{3.14, 1.41, 2.72}
    sort.Float64s(vals)
    fmt.Println(vals)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1.41 2.72 3.14]\n");
}

#[test]
fn compiles_and_runs_sort_ints_global() {
    let source = r#"
package main
import "fmt"
import "sort"

var nums = []int{9, 7, 8}

func main() {
    sort.Ints(nums)
    fmt.Println(nums)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[7 8 9]\n");
}

#[test]
fn compiles_and_runs_sort_already_sorted() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    nums := []int{1, 2, 3}
    sort.Ints(nums)
    fmt.Println(nums)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[1 2 3]\n");
}

#[test]
fn compiles_and_runs_sort_single_element() {
    let source = r#"
package main
import "fmt"
import "sort"

func main() {
    nums := []int{42}
    sort.Ints(nums)
    fmt.Println(nums)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[42]\n");
}
