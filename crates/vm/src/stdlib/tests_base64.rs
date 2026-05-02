use super::{
    resolve_stdlib_function, resolve_stdlib_method, resolve_stdlib_value,
    stdlib_function_param_types, stdlib_function_result_count, stdlib_function_result_types,
    stdlib_function_returns_value, StdlibValueInit,
};

#[test]
fn resolves_base64_wrappers_from_the_registry() {
    let encode = resolve_stdlib_function("encoding/base64", "StdEncodingEncodeToString")
        .expect("StdEncodingEncodeToString should exist");
    assert!(stdlib_function_returns_value(encode));
    assert_eq!(stdlib_function_result_count(encode), 1);
    assert_eq!(stdlib_function_result_types(encode), Some(&["string"][..]));
    assert_eq!(stdlib_function_param_types(encode), Some(&["[]byte"][..]));

    let decode = resolve_stdlib_function("encoding/base64", "StdEncodingDecodeString")
        .expect("StdEncodingDecodeString should exist");
    assert!(!stdlib_function_returns_value(decode));
    assert_eq!(stdlib_function_result_count(decode), 2);
    assert_eq!(
        stdlib_function_result_types(decode),
        Some(&["[]byte", "error"][..])
    );
    assert_eq!(stdlib_function_param_types(decode), Some(&["string"][..]));

    let raw_url_encode = resolve_stdlib_function("encoding/base64", "RawURLEncodingEncodeToString")
        .expect("RawURLEncodingEncodeToString should exist");
    assert!(stdlib_function_returns_value(raw_url_encode));
    assert_eq!(
        stdlib_function_param_types(raw_url_encode),
        Some(&["[]byte"][..])
    );

    let raw_url_decode = resolve_stdlib_function("encoding/base64", "RawURLEncodingDecodeString")
        .expect("RawURLEncodingDecodeString should exist");
    assert!(!stdlib_function_returns_value(raw_url_decode));
    assert_eq!(
        stdlib_function_result_types(raw_url_decode),
        Some(&["[]byte", "error"][..])
    );
}

#[test]
fn resolves_base64_encoding_values_and_methods() {
    let std_encoding =
        resolve_stdlib_value("encoding/base64", "StdEncoding").expect("StdEncoding should exist");
    assert_eq!(std_encoding.typ, "*base64.Encoding");
    assert_eq!(
        std_encoding.value,
        StdlibValueInit::NewPointerWithIntField {
            type_name: "base64.Encoding",
            field: "__encodingKind",
            value: 1,
        }
    );

    let raw_url_encoding = resolve_stdlib_value("encoding/base64", "RawURLEncoding")
        .expect("RawURLEncoding should exist");
    assert_eq!(raw_url_encoding.typ, "*base64.Encoding");
    assert_eq!(
        raw_url_encoding.value,
        StdlibValueInit::NewPointerWithIntField {
            type_name: "base64.Encoding",
            field: "__encodingKind",
            value: 4,
        }
    );

    let encode = resolve_stdlib_method("*base64.Encoding", "EncodeToString")
        .expect("(*base64.Encoding).EncodeToString should exist");
    assert!(stdlib_function_returns_value(encode));
    assert_eq!(
        stdlib_function_param_types(encode),
        Some(&["*base64.Encoding", "[]byte"][..])
    );
    assert_eq!(stdlib_function_result_types(encode), Some(&["string"][..]));

    let decode = resolve_stdlib_method("*base64.Encoding", "DecodeString")
        .expect("(*base64.Encoding).DecodeString should exist");
    assert!(!stdlib_function_returns_value(decode));
    assert_eq!(
        stdlib_function_param_types(decode),
        Some(&["*base64.Encoding", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(decode),
        Some(&["[]byte", "error"][..])
    );
}
