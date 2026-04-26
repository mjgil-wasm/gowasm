use super::{
    resolve_stdlib_function, resolve_stdlib_method, stdlib_function_param_types,
    stdlib_function_result_count, stdlib_function_result_types, stdlib_function_returns_value,
};

#[test]
fn resolves_regexp_compile_and_compiled_methods_from_the_registry() {
    let compile =
        resolve_stdlib_function("regexp", "Compile").expect("regexp.Compile should exist");
    assert!(!stdlib_function_returns_value(compile));
    assert_eq!(stdlib_function_result_count(compile), 2);
    assert_eq!(stdlib_function_param_types(compile), Some(&["string"][..]));
    assert_eq!(
        stdlib_function_result_types(compile),
        Some(&["*regexp.Regexp", "error"][..])
    );

    let must_compile =
        resolve_stdlib_function("regexp", "MustCompile").expect("regexp.MustCompile should exist");
    assert!(stdlib_function_returns_value(must_compile));
    assert_eq!(stdlib_function_result_count(must_compile), 1);
    assert_eq!(
        stdlib_function_param_types(must_compile),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(must_compile),
        Some(&["*regexp.Regexp"][..])
    );

    let match_string = resolve_stdlib_method("*regexp.Regexp", "MatchString")
        .expect("(*regexp.Regexp).MatchString should exist");
    assert!(stdlib_function_returns_value(match_string));
    assert_eq!(
        stdlib_function_param_types(match_string),
        Some(&["*regexp.Regexp", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(match_string),
        Some(&["bool"][..])
    );

    let find_string = resolve_stdlib_method("*regexp.Regexp", "FindString")
        .expect("(*regexp.Regexp).FindString should exist");
    assert!(stdlib_function_returns_value(find_string));
    assert_eq!(
        stdlib_function_param_types(find_string),
        Some(&["*regexp.Regexp", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(find_string),
        Some(&["string"][..])
    );

    let replace_all_string = resolve_stdlib_method("*regexp.Regexp", "ReplaceAllString")
        .expect("(*regexp.Regexp).ReplaceAllString should exist");
    assert!(stdlib_function_returns_value(replace_all_string));
    assert_eq!(
        stdlib_function_param_types(replace_all_string),
        Some(&["*regexp.Regexp", "string", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(replace_all_string),
        Some(&["string"][..])
    );

    let find_string_submatch = resolve_stdlib_method("*regexp.Regexp", "FindStringSubmatch")
        .expect("(*regexp.Regexp).FindStringSubmatch should exist");
    assert!(stdlib_function_returns_value(find_string_submatch));
    assert_eq!(
        stdlib_function_param_types(find_string_submatch),
        Some(&["*regexp.Regexp", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(find_string_submatch),
        Some(&["[]string"][..])
    );

    let split =
        resolve_stdlib_method("*regexp.Regexp", "Split").expect("(*regexp.Regexp).Split exists");
    assert!(stdlib_function_returns_value(split));
    assert_eq!(
        stdlib_function_param_types(split),
        Some(&["*regexp.Regexp", "string", "int"][..])
    );
    assert_eq!(stdlib_function_result_types(split), Some(&["[]string"][..]));
}
