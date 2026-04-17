use super::{
    resolve_stdlib_function, resolve_stdlib_method, stdlib_function_param_types,
    stdlib_function_result_count, stdlib_function_result_types, stdlib_function_returns_value,
    stdlib_function_variadic_param_type,
};

#[test]
fn resolves_strings_replacer_constructor_and_method_from_the_registry() {
    let constructor = resolve_stdlib_function("strings", "NewReplacer")
        .expect("strings.NewReplacer should exist");
    assert!(stdlib_function_returns_value(constructor));
    assert_eq!(stdlib_function_result_count(constructor), 1);
    assert_eq!(stdlib_function_param_types(constructor), Some(&[][..]));
    assert_eq!(
        stdlib_function_variadic_param_type(constructor),
        Some("string")
    );
    assert_eq!(
        stdlib_function_result_types(constructor),
        Some(&["*strings.Replacer"][..])
    );

    let replace = resolve_stdlib_method("*strings.Replacer", "Replace")
        .expect("(*strings.Replacer).Replace should exist");
    assert!(stdlib_function_returns_value(replace));
    assert_eq!(stdlib_function_result_count(replace), 1);
    assert_eq!(
        stdlib_function_param_types(replace),
        Some(&["*strings.Replacer", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(replace), Some(&["string"][..]));
}
