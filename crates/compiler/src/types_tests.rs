use std::collections::HashMap;

use gowasm_parser::parse_source_file;
use gowasm_vm::Vm;

use super::{
    build_substitutions, check_type_constraint, collect_type_tables, infer_type_args,
    parse_function_type, parse_type_key, split_generic_type_name, substitute_type_params,
    validate_type_args, CompileError, GenericFunctionDef, GenericMethodDef, InstanceKey,
    InstantiationCache, TypeConstraint, TypeKey, TypeParamDef,
};
use crate::compile_source;

#[test]
fn parses_nested_function_signatures() {
    let signature =
        "__gowasm_func__(__gowasm_func__(string)->(string))->(__gowasm_func__(string)->(string))";
    let parsed = parse_function_type(signature).expect("signature should parse");
    assert_eq!(
        parsed.0,
        vec!["__gowasm_func__(string)->(string)".to_string()]
    );
    assert_eq!(
        parsed.1,
        vec!["__gowasm_func__(string)->(string)".to_string()]
    );
}

#[test]
fn parses_nested_type_keys_for_canonical_type_shapes() {
    let parsed = parse_type_key(
        "func(map[string]*Pair[int], <-chan []Value[T]) (chan<- map[string]func(*Node[T]) []Result[U], [3]*pkg.Item)",
    )
    .expect("type key should parse");
    assert_eq!(
        parsed.render(),
        "__gowasm_func__(map[string]*Pair[int],<-chan []Value[T])->(chan<- map[string]__gowasm_func__(*Node[T])->([]Result[U]),[3]*pkg.Item)"
    );

    match parsed {
        TypeKey::Function { params, results } => {
            assert_eq!(params.len(), 2);
            assert_eq!(results.len(), 2);
            assert!(matches!(&params[0], TypeKey::Map { .. }));
            assert!(matches!(&params[1], TypeKey::Channel { .. }));
            assert!(matches!(&results[0], TypeKey::Channel { .. }));
            assert!(matches!(&results[1], TypeKey::Array { len: 3, .. }));
        }
        other => panic!("expected function type key, got {other:?}"),
    }
}

#[test]
fn parses_function_signatures_with_generic_maps_and_nested_functions() {
    let signature = "__gowasm_func__(map[string]Pair[int,string],__gowasm_func__(Pair[int,string])->(map[string]Pair[int,string]))->(__gowasm_func__(map[string]Pair[int,string])->(Pair[int,string]))";
    let parsed = parse_function_type(signature).expect("signature should parse");
    assert_eq!(
        parsed.0,
        vec![
            "map[string]Pair[int,string]".to_string(),
            "__gowasm_func__(Pair[int,string])->(map[string]Pair[int,string])".to_string(),
        ]
    );
    assert_eq!(
        parsed.1,
        vec!["__gowasm_func__(map[string]Pair[int,string])->(Pair[int,string])".to_string()]
    );
}

#[test]
fn splits_generic_type_args_when_one_arg_is_a_function_type() {
    let (base, args) = split_generic_type_name(
        "Handler[__gowasm_func__(Pair[int,string])->(map[string]Pair[int,string]),Pair[int,string]]",
    )
    .expect("generic type should parse");
    assert_eq!(base, "Handler");
    assert_eq!(
        args,
        vec![
            "__gowasm_func__(Pair[int,string])->(map[string]Pair[int,string])".to_string(),
            "Pair[int,string]".to_string(),
        ]
    );
}

#[test]
fn substitute_replaces_direct_type_param() {
    let subs = build_substitutions(
        &[TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        &["int".to_string()],
    );
    assert_eq!(substitute_type_params("T", &subs), "int");
}

#[test]
fn substitute_replaces_in_slice() {
    let subs = build_substitutions(
        &[TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        &["string".to_string()],
    );
    assert_eq!(substitute_type_params("[]T", &subs), "[]string");
}

#[test]
fn substitute_replaces_in_pointer() {
    let subs = build_substitutions(
        &[TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        &["int".to_string()],
    );
    assert_eq!(substitute_type_params("*T", &subs), "*int");
}

#[test]
fn substitute_replaces_in_map() {
    let subs = build_substitutions(
        &[
            TypeParamDef {
                name: "K".to_string(),
                constraint: TypeConstraint::Any,
            },
            TypeParamDef {
                name: "V".to_string(),
                constraint: TypeConstraint::Any,
            },
        ],
        &["string".to_string(), "int".to_string()],
    );
    assert_eq!(substitute_type_params("map[K]V", &subs), "map[string]int");
}

#[test]
fn substitute_replaces_nested_canonical_type_shapes() {
    let subs = build_substitutions(
        &[
            TypeParamDef {
                name: "K".to_string(),
                constraint: TypeConstraint::Any,
            },
            TypeParamDef {
                name: "V".to_string(),
                constraint: TypeConstraint::Any,
            },
            TypeParamDef {
                name: "T".to_string(),
                constraint: TypeConstraint::Any,
            },
        ],
        &[
            "string".to_string(),
            "int".to_string(),
            "pkg.Value".to_string(),
        ],
    );
    assert_eq!(
        substitute_type_params(
            "__gowasm_func__(map[K]*Box[V],<-chan []V)->(chan<- Pair[K,V],*T)",
            &subs,
        ),
        "__gowasm_func__(map[string]*Box[int],<-chan []int)->(chan<- Pair[string,int],*pkg.Value)"
    );
}

#[test]
fn substitute_leaves_concrete_types_alone() {
    let subs = HashMap::new();
    assert_eq!(substitute_type_params("int", &subs), "int");
    assert_eq!(substitute_type_params("[]string", &subs), "[]string");
}

#[test]
fn instance_key_mangled_name() {
    let key = InstanceKey {
        base_name: "Map".to_string(),
        type_args: vec!["string".to_string(), "int".to_string()],
    };
    assert_eq!(key.mangled_name(), "Map[string,int]");
}

#[test]
fn any_constraint_accepts_all() {
    assert!(check_type_constraint(&TypeConstraint::Any, "int", &HashMap::new()).is_ok());
    assert!(check_type_constraint(&TypeConstraint::Any, "string", &HashMap::new()).is_ok());
}

#[test]
fn comparable_rejects_interface() {
    let mut ifaces = HashMap::new();
    ifaces.insert(
        "MyIface".to_string(),
        super::InterfaceTypeDef {
            type_id: gowasm_vm::TypeId(999),
            methods: Vec::new(),
        },
    );
    assert!(check_type_constraint(&TypeConstraint::Comparable, "MyIface", &ifaces).is_err());
    assert!(check_type_constraint(&TypeConstraint::Comparable, "int", &ifaces).is_ok());
}

#[test]
fn validate_type_args_rejects_wrong_arity() {
    let result = validate_type_args(
        &[TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        &[],
        &HashMap::new(),
    );
    match result {
        Err(CompileError::Unsupported { detail }) => {
            assert!(detail.contains("expected 1 type argument(s), found 0"));
        }
        other => panic!("expected wrong-arity diagnostic, got {other:?}"),
    }
}

#[test]
fn validate_type_args_checks_constraints() {
    let mut ifaces = HashMap::new();
    ifaces.insert(
        "MyIface".to_string(),
        super::InterfaceTypeDef {
            type_id: gowasm_vm::TypeId(999),
            methods: Vec::new(),
        },
    );
    let result = validate_type_args(
        &[TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Comparable,
        }],
        &["MyIface".to_string()],
        &ifaces,
    );
    match result {
        Err(CompileError::Unsupported { detail }) => {
            assert!(detail.contains("type argument `MyIface` does not satisfy `T`"));
            assert!(detail.contains("comparable"));
        }
        other => panic!("expected constraint diagnostic, got {other:?}"),
    }
}

#[test]
fn instantiation_cache_tracks_function_and_type_instances_by_instance_key() {
    let mut cache = InstantiationCache::default();
    let key = InstanceKey {
        base_name: "Map".to_string(),
        type_args: vec!["string".to_string(), "int".to_string()],
    };
    cache
        .function_instances
        .entry(key.clone())
        .or_insert_with(|| key.mangled_name());
    cache
        .type_instances
        .entry(key.clone())
        .or_insert(gowasm_vm::TypeId(777));
    cache
        .function_instances
        .entry(key.clone())
        .or_insert_with(|| "ignored".to_string());
    cache
        .type_instances
        .entry(key.clone())
        .or_insert(gowasm_vm::TypeId(888));

    assert_eq!(
        cache.function_instances.get(&key).map(String::as_str),
        Some("Map[string,int]")
    );
    assert_eq!(
        cache.type_instances.get(&key),
        Some(&gowasm_vm::TypeId(777))
    );
}

#[test]
fn generic_function_collected_from_source() {
    let source = r#"
        package main
        func Identity[T any](x T) T { return x }
        func main() {}
    "#;
    let result = compile_source(source);
    assert!(
        result.is_ok(),
        "generic function should not break compilation: {:?}",
        result.err()
    );
}

#[test]
fn generic_function_is_not_emitted_until_instantiated() {
    let source = r#"
        package main
        func Identity[T any](x T) T { return x }
        func main() {}
    "#;
    let program = compile_source(source).expect("generic template should be skipped");
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "main"));
    assert!(!program
        .functions
        .iter()
        .any(|function| function.name == "Identity"));
}

#[test]
fn explicit_generic_calls_compile_to_concrete_functions() {
    let source = r#"
        package main
        import "fmt"

        func Identity[T any](x T) T { return x }
        func main() {
            fmt.Println(Identity[int](1), Identity[int](2))
        }
    "#;
    let program = compile_source(source).expect("explicit generic calls should compile");
    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.name == "Identity[int]")
            .count(),
        1
    );
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2\n");
}

#[test]
fn generic_struct_literals_compile_to_concrete_runtime_types() {
    let source = r#"
        package main
        import "fmt"

        type Box[T any] struct {
            value T
        }

        func main() {
            var box Box[int]
            box.value = 7
            fmt.Println(box.value)
        }
    "#;
    let program = compile_source(source).expect("generic struct literal should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn generic_methods_compile_on_concrete_named_types() {
    let source = r#"
        package main
        import "fmt"

        type Box[T any] struct {
            value T
        }

        func (b Box[T]) Get() T {
            return b.value
        }

        func main() {
            var box Box[int]
            box.value = 1
            fmt.Println(box.Get())
        }
    "#;
    let program = compile_source(source).expect("generic methods should compile");
    assert_eq!(
        program
            .functions
            .iter()
            .filter(|function| function.name == "Box[int].Get")
            .count(),
        1
    );
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn generic_zero_value_locals_use_concrete_runtime_types() {
    let source = r#"
        package main
        import "fmt"

        type Counter[T any] struct {
            value T
        }

        func (c Counter[T]) Value() T {
            return c.value
        }

        func main() {
            var counter Counter[int]
            fmt.Println(counter.Value())
        }
    "#;
    let program = compile_source(source).expect("generic zero-value locals should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0\n");
}

#[test]
fn generic_interfaces_accept_instantiated_generic_methods() {
    let source = r#"
        package main
        import "fmt"

        type Reader[T any] interface {
            Value() T
        }

        type Box[T any] struct {
            value T
        }

        func (b Box[T]) Value() T {
            return b.value
        }

        func main() {
            var box Box[int]
            box.value = 9
            var reader Reader[int] = box
            fmt.Println(reader.Value())
        }
    "#;
    let program = compile_source(source).expect("generic interface satisfaction should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9\n");
}

#[test]
fn infer_type_args_from_direct_parameter() {
    let generic_function = GenericFunctionDef {
        type_params: vec![TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        param_types: vec!["T".to_string()],
        result_types: vec!["T".to_string()],
    };
    let inferred = infer_type_args(&generic_function, &["int".to_string()], &HashMap::new())
        .expect("type args should infer");
    assert_eq!(inferred, vec!["int".to_string()]);
}

#[test]
fn infer_type_args_from_nested_slice_pointer_and_map_shapes() {
    let generic_function = GenericFunctionDef {
        type_params: vec![
            TypeParamDef {
                name: "K".to_string(),
                constraint: TypeConstraint::Any,
            },
            TypeParamDef {
                name: "V".to_string(),
                constraint: TypeConstraint::Any,
            },
        ],
        param_types: vec!["map[K][]*V".to_string()],
        result_types: Vec::new(),
    };
    let inferred = infer_type_args(
        &generic_function,
        &["map[string][]*int".to_string()],
        &HashMap::new(),
    )
    .expect("nested shapes should infer");
    assert_eq!(inferred, vec!["string".to_string(), "int".to_string()]);
}

#[test]
fn infer_type_args_from_channel_shape() {
    let generic_function = GenericFunctionDef {
        type_params: vec![TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        param_types: vec!["<-chan T".to_string()],
        result_types: Vec::new(),
    };
    let inferred = infer_type_args(
        &generic_function,
        &["chan int".to_string()],
        &HashMap::new(),
    )
    .expect("channel element type should infer");
    assert_eq!(inferred, vec!["int".to_string()]);
}

#[test]
fn infer_type_args_rejects_conflicting_bindings() {
    let generic_function = GenericFunctionDef {
        type_params: vec![TypeParamDef {
            name: "T".to_string(),
            constraint: TypeConstraint::Any,
        }],
        param_types: vec!["T".to_string(), "T".to_string()],
        result_types: Vec::new(),
    };
    let inferred = infer_type_args(
        &generic_function,
        &["int".to_string(), "string".to_string()],
        &HashMap::new(),
    );
    assert!(inferred.is_none());
}

#[test]
fn inferred_generic_calls_compile_to_concrete_functions() {
    let source = r#"
        package main
        import "fmt"

        func Identity[T any](x T) T { return x }
        func main() {
            fmt.Println(Identity(1), Identity("go"))
        }
    "#;
    let program = compile_source(source).expect("inferred generic calls should compile");
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "Identity[int]"));
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "Identity[string]"));
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 go\n");
}

#[test]
fn generic_multi_result_calls_compile_to_concrete_functions() {
    let source = r#"
        package main
        import "fmt"

        func Pair[T any](x T) (T, T) { return x, x }

        func main() {
            first, second := Pair(7)
            fmt.Println(first, second)
        }
    "#;
    let program = compile_source(source).expect("generic multi-result call should compile");
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "Pair[int]"));
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 7\n");
}

#[test]
fn generic_function_body_type_substitution_reaches_locals() {
    let source = r#"
        package main
        import "fmt"

        func Zero[T any]() T {
            var zero T
            return zero
        }

        func main() {
            fmt.Println(Zero[int](), Zero[string]())
        }
    "#;
    let program = compile_source(source).expect("generic zero-value function should compile");
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "Zero[int]"));
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "Zero[string]"));
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 \n");
}

#[test]
fn generic_type_collected_from_source() {
    let source = r#"
        package main
        type Box[T any] struct { value T }
        func main() {}
    "#;
    let result = compile_source(source);
    assert!(
        result.is_ok(),
        "generic type should not break compilation: {:?}",
        result.err()
    );
}

#[test]
fn generic_type_methods_are_recorded_in_type_tables() {
    let source = r#"
        package main
        type Box[T any] struct { value T }
        func (b Box[T]) Get() T { return b.value }
        func (b *Box[T]) Set(v T) { b.value = v }
    "#;
    let file = parse_source_file(source).expect("source should parse");
    let tables = collect_type_tables(std::iter::once(("main.go", &file)))
        .expect("generic type methods should type-check");
    let generic_type = tables.generic_types.get("Box").expect("generic type");
    assert_eq!(
        generic_type.methods,
        vec![
            GenericMethodDef {
                name: "Get".to_string(),
                params: Vec::new(),
                result_types: vec!["T".to_string()],
                pointer_receiver: false,
            },
            GenericMethodDef {
                name: "Set".to_string(),
                params: vec![gowasm_parser::Parameter {
                    name: "v".to_string(),
                    typ: "T".to_string(),
                    variadic: false,
                }],
                result_types: Vec::new(),
                pointer_receiver: true,
            },
        ]
    );
}

#[test]
fn generic_type_method_templates_are_not_emitted_until_instantiated() {
    let source = r#"
        package main
        type Box[T any] struct { value T }
        func (b Box[T]) Get() T { return b.value }
        func main() {}
    "#;
    let program = compile_source(source).expect("generic type method template should be skipped");
    assert!(program
        .functions
        .iter()
        .any(|function| function.name == "main"));
    assert!(!program
        .functions
        .iter()
        .any(|function| function.name.ends_with(".Get")));
}
