use super::{
    resolve_stdlib_function, stdlib_function_param_types, stdlib_function_result_count,
    stdlib_function_result_types, stdlib_function_returns_value,
};

#[test]
fn resolves_encoding_json_marshal_from_the_registry() {
    let marshal =
        resolve_stdlib_function("encoding/json", "Marshal").expect("encoding/json.Marshal exists");
    assert!(!stdlib_function_returns_value(marshal));
    assert_eq!(stdlib_function_result_count(marshal), 2);
    assert_eq!(
        stdlib_function_param_types(marshal),
        Some(&["interface{}"][..])
    );
    assert_eq!(
        stdlib_function_result_types(marshal),
        Some(&["[]byte", "error"][..])
    );

    let marshal_indent = resolve_stdlib_function("encoding/json", "MarshalIndent")
        .expect("encoding/json.MarshalIndent exists");
    assert!(!stdlib_function_returns_value(marshal_indent));
    assert_eq!(stdlib_function_result_count(marshal_indent), 2);
    assert_eq!(
        stdlib_function_param_types(marshal_indent),
        Some(&["interface{}", "string", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(marshal_indent),
        Some(&["[]byte", "error"][..])
    );

    let valid =
        resolve_stdlib_function("encoding/json", "Valid").expect("encoding/json.Valid exists");
    assert!(stdlib_function_returns_value(valid));
    assert_eq!(stdlib_function_result_count(valid), 1);
    assert_eq!(stdlib_function_param_types(valid), Some(&["[]byte"][..]));
    assert_eq!(stdlib_function_result_types(valid), Some(&["bool"][..]));

    let unmarshal = resolve_stdlib_function("encoding/json", "Unmarshal")
        .expect("encoding/json.Unmarshal exists");
    assert!(stdlib_function_returns_value(unmarshal));
    assert_eq!(stdlib_function_result_count(unmarshal), 1);
    assert_eq!(
        stdlib_function_param_types(unmarshal),
        Some(&["[]byte", "interface{}"][..])
    );
    assert_eq!(
        stdlib_function_result_types(unmarshal),
        Some(&["error"][..])
    );
}
