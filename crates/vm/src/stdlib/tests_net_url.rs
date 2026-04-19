use super::{
    resolve_stdlib_function, resolve_stdlib_method, resolve_stdlib_runtime_method,
    stdlib_function_mutates_first_arg, stdlib_function_param_types, stdlib_function_result_count,
    stdlib_function_result_types, stdlib_function_returns_value,
    stdlib_function_variadic_param_type,
};
use crate::{TYPE_URL, TYPE_URL_PTR, TYPE_URL_VALUES};

#[test]
fn resolves_net_url_parse_function_from_the_registry() {
    let parse = resolve_stdlib_function("net/url", "Parse").expect("url.Parse should exist");
    assert!(!stdlib_function_returns_value(parse));
    assert_eq!(stdlib_function_result_count(parse), 2);
    assert_eq!(stdlib_function_param_types(parse), Some(&["string"][..]));
    assert_eq!(
        stdlib_function_result_types(parse),
        Some(&["*url.URL", "error"][..])
    );

    let parse_request_uri = resolve_stdlib_function("net/url", "ParseRequestURI")
        .expect("url.ParseRequestURI should exist");
    assert!(!stdlib_function_returns_value(parse_request_uri));
    assert_eq!(stdlib_function_result_count(parse_request_uri), 2);
    assert_eq!(
        stdlib_function_param_types(parse_request_uri),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(parse_request_uri),
        Some(&["*url.URL", "error"][..])
    );

    let join_path =
        resolve_stdlib_function("net/url", "JoinPath").expect("url.JoinPath should exist");
    assert!(!stdlib_function_returns_value(join_path));
    assert_eq!(stdlib_function_result_count(join_path), 2);
    assert_eq!(
        stdlib_function_param_types(join_path),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(join_path),
        Some(&["string", "error"][..])
    );
    assert_eq!(
        stdlib_function_variadic_param_type(join_path),
        Some("string")
    );

    let parse_query =
        resolve_stdlib_function("net/url", "ParseQuery").expect("url.ParseQuery should exist");
    assert!(!stdlib_function_returns_value(parse_query));
    assert_eq!(stdlib_function_result_count(parse_query), 2);
    assert_eq!(
        stdlib_function_param_types(parse_query),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(parse_query),
        Some(&["url.Values", "error"][..])
    );

    let query_escape =
        resolve_stdlib_function("net/url", "QueryEscape").expect("url.QueryEscape should exist");
    assert!(stdlib_function_returns_value(query_escape));
    assert_eq!(stdlib_function_result_count(query_escape), 1);
    assert_eq!(
        stdlib_function_param_types(query_escape),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(query_escape),
        Some(&["string"][..])
    );

    let query_unescape = resolve_stdlib_function("net/url", "QueryUnescape")
        .expect("url.QueryUnescape should exist");
    assert!(!stdlib_function_returns_value(query_unescape));
    assert_eq!(stdlib_function_result_count(query_unescape), 2);
    assert_eq!(
        stdlib_function_param_types(query_unescape),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(query_unescape),
        Some(&["string", "error"][..])
    );

    let path_escape =
        resolve_stdlib_function("net/url", "PathEscape").expect("url.PathEscape should exist");
    assert!(stdlib_function_returns_value(path_escape));
    assert_eq!(stdlib_function_result_count(path_escape), 1);
    assert_eq!(
        stdlib_function_param_types(path_escape),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(path_escape),
        Some(&["string"][..])
    );

    let path_unescape =
        resolve_stdlib_function("net/url", "PathUnescape").expect("url.PathUnescape should exist");
    assert!(!stdlib_function_returns_value(path_unescape));
    assert_eq!(stdlib_function_result_count(path_unescape), 2);
    assert_eq!(
        stdlib_function_param_types(path_unescape),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(path_unescape),
        Some(&["string", "error"][..])
    );
}

#[test]
fn resolves_net_url_string_method_from_the_registry() {
    let string = resolve_stdlib_method("url.URL", "String").expect("url.URL.String should exist");
    assert!(stdlib_function_returns_value(string));
    assert_eq!(stdlib_function_result_count(string), 1);
    assert_eq!(stdlib_function_param_types(string), Some(&["url.URL"][..]));
    assert_eq!(stdlib_function_result_types(string), Some(&["string"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL, "String"),
        Some(string)
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "String"),
        Some(string)
    );

    let query = resolve_stdlib_method("*url.URL", "Query").expect("(*url.URL).Query should exist");
    assert!(stdlib_function_returns_value(query));
    assert_eq!(stdlib_function_result_count(query), 1);
    assert_eq!(stdlib_function_param_types(query), Some(&["*url.URL"][..]));
    assert_eq!(
        stdlib_function_result_types(query),
        Some(&["url.Values"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "Query"),
        Some(query)
    );

    let is_abs = resolve_stdlib_method("*url.URL", "IsAbs").expect("(*url.URL).IsAbs should exist");
    assert!(stdlib_function_returns_value(is_abs));
    assert_eq!(stdlib_function_result_count(is_abs), 1);
    assert_eq!(stdlib_function_param_types(is_abs), Some(&["*url.URL"][..]));
    assert_eq!(stdlib_function_result_types(is_abs), Some(&["bool"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "IsAbs"),
        Some(is_abs)
    );

    let hostname =
        resolve_stdlib_method("*url.URL", "Hostname").expect("(*url.URL).Hostname should exist");
    assert!(stdlib_function_returns_value(hostname));
    assert_eq!(stdlib_function_result_count(hostname), 1);
    assert_eq!(
        stdlib_function_param_types(hostname),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(hostname),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "Hostname"),
        Some(hostname)
    );

    let port = resolve_stdlib_method("*url.URL", "Port").expect("(*url.URL).Port should exist");
    assert!(stdlib_function_returns_value(port));
    assert_eq!(stdlib_function_result_count(port), 1);
    assert_eq!(stdlib_function_param_types(port), Some(&["*url.URL"][..]));
    assert_eq!(stdlib_function_result_types(port), Some(&["string"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "Port"),
        Some(port)
    );

    let request_uri =
        resolve_stdlib_method("*url.URL", "RequestURI").expect("(*url.URL).RequestURI exists");
    assert!(stdlib_function_returns_value(request_uri));
    assert_eq!(stdlib_function_result_count(request_uri), 1);
    assert_eq!(
        stdlib_function_param_types(request_uri),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(request_uri),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "RequestURI"),
        Some(request_uri)
    );

    let join_path =
        resolve_stdlib_method("*url.URL", "JoinPath").expect("(*url.URL).JoinPath exists");
    assert!(stdlib_function_returns_value(join_path));
    assert_eq!(stdlib_function_result_count(join_path), 1);
    assert_eq!(
        stdlib_function_param_types(join_path),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(join_path),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_variadic_param_type(join_path),
        Some("string")
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "JoinPath"),
        Some(join_path)
    );

    let parse = resolve_stdlib_method("*url.URL", "Parse").expect("(*url.URL).Parse exists");
    assert!(!stdlib_function_returns_value(parse));
    assert_eq!(stdlib_function_result_count(parse), 2);
    assert_eq!(
        stdlib_function_param_types(parse),
        Some(&["*url.URL", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(parse),
        Some(&["*url.URL", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "Parse"),
        Some(parse)
    );

    let resolve_reference = resolve_stdlib_method("*url.URL", "ResolveReference")
        .expect("(*url.URL).ResolveReference should exist");
    assert!(stdlib_function_returns_value(resolve_reference));
    assert_eq!(stdlib_function_result_count(resolve_reference), 1);
    assert_eq!(
        stdlib_function_param_types(resolve_reference),
        Some(&["*url.URL", "*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(resolve_reference),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "ResolveReference"),
        Some(resolve_reference)
    );

    let redacted =
        resolve_stdlib_method("*url.URL", "Redacted").expect("(*url.URL).Redacted should exist");
    assert!(stdlib_function_returns_value(redacted));
    assert_eq!(stdlib_function_result_count(redacted), 1);
    assert_eq!(
        stdlib_function_param_types(redacted),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(redacted),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "Redacted"),
        Some(redacted)
    );

    let marshal_binary = resolve_stdlib_method("*url.URL", "MarshalBinary")
        .expect("(*url.URL).MarshalBinary should exist");
    assert!(!stdlib_function_returns_value(marshal_binary));
    assert_eq!(stdlib_function_result_count(marshal_binary), 2);
    assert_eq!(
        stdlib_function_param_types(marshal_binary),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(marshal_binary),
        Some(&["[]byte", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "MarshalBinary"),
        Some(marshal_binary)
    );

    let unmarshal_binary = resolve_stdlib_method("*url.URL", "UnmarshalBinary")
        .expect("(*url.URL).UnmarshalBinary should exist");
    assert!(stdlib_function_returns_value(unmarshal_binary));
    assert_eq!(stdlib_function_result_count(unmarshal_binary), 1);
    assert_eq!(
        stdlib_function_param_types(unmarshal_binary),
        Some(&["*url.URL", "[]byte"][..])
    );
    assert_eq!(
        stdlib_function_result_types(unmarshal_binary),
        Some(&["error"][..])
    );
    assert!(!stdlib_function_mutates_first_arg(unmarshal_binary));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "UnmarshalBinary"),
        Some(unmarshal_binary)
    );

    let escaped_path = resolve_stdlib_method("*url.URL", "EscapedPath")
        .expect("(*url.URL).EscapedPath should exist");
    assert!(stdlib_function_returns_value(escaped_path));
    assert_eq!(stdlib_function_result_count(escaped_path), 1);
    assert_eq!(
        stdlib_function_param_types(escaped_path),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(escaped_path),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "EscapedPath"),
        Some(escaped_path)
    );

    let escaped_fragment = resolve_stdlib_method("*url.URL", "EscapedFragment")
        .expect("(*url.URL).EscapedFragment should exist");
    assert!(stdlib_function_returns_value(escaped_fragment));
    assert_eq!(stdlib_function_result_count(escaped_fragment), 1);
    assert_eq!(
        stdlib_function_param_types(escaped_fragment),
        Some(&["*url.URL"][..])
    );
    assert_eq!(
        stdlib_function_result_types(escaped_fragment),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_PTR, "EscapedFragment"),
        Some(escaped_fragment)
    );
}

#[test]
fn resolves_net_url_values_encode_method_from_the_registry() {
    let get = resolve_stdlib_method("url.Values", "Get").expect("url.Values.Get should exist");
    assert!(stdlib_function_returns_value(get));
    assert_eq!(stdlib_function_result_count(get), 1);
    assert_eq!(
        stdlib_function_param_types(get),
        Some(&["url.Values", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(get), Some(&["string"][..]));
    assert!(!stdlib_function_mutates_first_arg(get));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_VALUES, "Get"),
        Some(get)
    );

    let set = resolve_stdlib_method("url.Values", "Set").expect("url.Values.Set should exist");
    assert!(!stdlib_function_returns_value(set));
    assert_eq!(stdlib_function_result_count(set), 0);
    assert_eq!(
        stdlib_function_param_types(set),
        Some(&["url.Values", "string", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(set), None);
    assert!(stdlib_function_mutates_first_arg(set));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_VALUES, "Set"),
        Some(set)
    );

    let add = resolve_stdlib_method("url.Values", "Add").expect("url.Values.Add should exist");
    assert!(!stdlib_function_returns_value(add));
    assert_eq!(stdlib_function_result_count(add), 0);
    assert_eq!(
        stdlib_function_param_types(add),
        Some(&["url.Values", "string", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(add), None);
    assert!(stdlib_function_mutates_first_arg(add));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_VALUES, "Add"),
        Some(add)
    );

    let del = resolve_stdlib_method("url.Values", "Del").expect("url.Values.Del should exist");
    assert!(!stdlib_function_returns_value(del));
    assert_eq!(stdlib_function_result_count(del), 0);
    assert_eq!(
        stdlib_function_param_types(del),
        Some(&["url.Values", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(del), None);
    assert!(stdlib_function_mutates_first_arg(del));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_VALUES, "Del"),
        Some(del)
    );

    let has = resolve_stdlib_method("url.Values", "Has").expect("url.Values.Has should exist");
    assert!(stdlib_function_returns_value(has));
    assert_eq!(stdlib_function_result_count(has), 1);
    assert_eq!(
        stdlib_function_param_types(has),
        Some(&["url.Values", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(has), Some(&["bool"][..]));
    assert!(!stdlib_function_mutates_first_arg(has));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_VALUES, "Has"),
        Some(has)
    );

    let encode =
        resolve_stdlib_method("url.Values", "Encode").expect("url.Values.Encode should exist");
    assert!(stdlib_function_returns_value(encode));
    assert_eq!(stdlib_function_result_count(encode), 1);
    assert_eq!(
        stdlib_function_param_types(encode),
        Some(&["url.Values"][..])
    );
    assert_eq!(stdlib_function_result_types(encode), Some(&["string"][..]));
    assert!(!stdlib_function_mutates_first_arg(encode));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_VALUES, "Encode"),
        Some(encode)
    );
}
