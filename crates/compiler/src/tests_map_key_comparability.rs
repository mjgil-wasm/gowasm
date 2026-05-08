use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn rejects_slice_map_key_types_in_make_calls() {
    let source = r#"
package main

func main() {
    _ = make(map[[]int]int)
}
"#;

    let error = compile_source(source).expect_err("slice map keys should fail");
    assert!(error
        .to_string()
        .contains("map key type `[]int` is not comparable"));
}

#[test]
fn rejects_map_map_key_types_in_declarations() {
    let source = r#"
package main

type Lookup map[map[string]int]int

func main() {}
"#;

    let error = compile_source(source).expect_err("map map keys should fail");
    assert!(error
        .to_string()
        .contains("map key type `map[string]int` is not comparable"));
}

#[test]
fn rejects_function_map_key_types_in_declarations() {
    let source = r#"
package main

type Lookup map[func() int]int

func main() {}
"#;

    let error = compile_source(source).expect_err("function map keys should fail");
    let message = error.to_string();
    assert!(message.contains("map key type `"));
    assert!(message.contains("not comparable"));
}

#[test]
fn rejects_struct_containing_slice_map_keys() {
    let source = r#"
package main

type BadKey struct {
    Values []int
}

type Lookup map[BadKey]int

func main() {}
"#;

    let error = compile_source(source).expect_err("structs containing slices should fail");
    assert!(error
        .to_string()
        .contains("map key type `BadKey` is not comparable"));
}

#[test]
fn rejects_imported_alias_map_keys_with_non_comparable_underlying_types() {
    let source = r#"
package main

import "net/url"

type Lookup map[url.Values]int

func main() {}
"#;

    let error = compile_source(source).expect_err("imported map aliases should fail");
    assert!(error
        .to_string()
        .contains("map key type `url.Values` is not comparable"));
}

#[test]
fn rejects_generic_map_key_type_params_without_comparable_constraints() {
    let source = r#"
package main

type Dict[K any] map[K]int

func main() {}
"#;

    let error = compile_source(source).expect_err("generic map keys should require comparable");
    assert!(error
        .to_string()
        .contains("map key type parameter `K` must satisfy `comparable`"));
}

#[test]
fn rejects_generic_map_instantiations_with_non_comparable_key_types() {
    let source = r#"
package main

type Dict[K comparable] map[K]int

func main() {
    _ = Dict[[]int]{}
}
"#;

    let error = compile_source(source).expect_err("non-comparable type args should fail");
    assert!(error
        .to_string()
        .contains("type argument `[]int` does not satisfy `K` (constraint `comparable`)"));
}

#[test]
fn interface_map_keys_panic_when_the_dynamic_key_is_not_comparable() {
    let source = r#"
package main

func main() {
    values := map[any]int{}
    var key any = []int{1, 2, 3}
    values[key] = 1
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("dynamic non-comparable interface keys should panic");
    assert!(error.to_string().contains("hash of unhashable type []int"));
}
