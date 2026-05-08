use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn mutating_slice_helpers_preserve_named_generic_types_and_alias_visibility() {
    let source = r#"
package main
import "cmp"
import "fmt"
import "slices"
import "sort"

type Bag[T any] []T

type Item struct {
    Group int
    Label string
}

func makeBag[T any](values []T) Bag[T] {
    return values
}

func main() {
    values := makeBag([]Item{
        Item{Group: 2, Label: "z"},
        Item{Group: 1, Label: "b"},
        Item{Group: 1, Label: "a"},
        Item{Group: 2, Label: "y"},
    })
    alias := values[:]
    slices.SortStableFunc(values, func(a Item, b Item) int {
        if byGroup := cmp.Compare(a.Group, b.Group); byGroup != 0 {
            return byGroup
        }
        return cmp.Compare(a.Label, b.Label)
    })
    fmt.Println(values, alias)

    sort.Slice(values, func(i int, j int) bool {
        return values[i].Label > values[j].Label
    })
    fmt.Println(values, alias)

    labels := makeBag([]string{"go", "go", "vm", "vm", "wasm"})
    labelAlias := labels[:]
    labels = slices.Compact(labels)
    fmt.Printf("%T %v %v %d %d\n", labels, labels, labelAlias[:len(labels)], len(labels), cap(labels))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[{1 a} {1 b} {2 y} {2 z}] [{1 a} {1 b} {2 y} {2 z}]\n[{2 z} {2 y} {1 b} {1 a}] [{2 z} {2 y} {1 b} {1 a}]\nmain.Bag[string] [go vm wasm] [go vm wasm] 3 5\n"
    );
}

#[test]
fn generic_named_map_helpers_cover_keys_values_nil_and_callbacks() {
    let source = r#"
package main
import "cmp"
import "fmt"
import "maps"
import "slices"

type Dict[T any] map[string]T

func makeDict[T any](values map[string]T) Dict[T] {
    return values
}

func main() {
    var empty Dict[int]
    var other Dict[int]
    fmt.Println(empty == nil, maps.Equal(empty, maps.Clone(empty)))
    fmt.Println(len(maps.Keys(empty)), len(maps.Values(empty)))

    values := makeDict(map[string]int{"b": 2, "a": 1, "c": 3})
    keys := maps.Keys(values)
    slices.SortFunc(keys, func(a string, b string) int {
        return cmp.Compare(a, b)
    })
    nums := maps.Values(values)
    slices.SortFunc(nums, func(a int, b int) int {
        return cmp.Compare(a, b)
    })
    fmt.Println(keys)
    fmt.Println(nums)

    clone := maps.Clone(values)
    maps.Copy(clone, makeDict(map[string]int{"c": 30, "d": 4}))
    maps.DeleteFunc(clone, func(k string, v int) bool {
        return v > 20
    })
    fmt.Println(values["c"], values["d"])
    fmt.Println(clone["a"], clone["b"], clone["c"], clone["d"])
    fmt.Println(maps.Equal(empty, other))
    fmt.Println(maps.EqualFunc(values, makeDict(map[string]int{"a": 10, "b": 20, "c": 30}), func(left int, right int) bool {
        return right == left * 10
    }))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\n0 0\n[a b c]\n[1 2 3]\n3 0\n1 2 0 4\ntrue\ntrue\n"
    );
}
