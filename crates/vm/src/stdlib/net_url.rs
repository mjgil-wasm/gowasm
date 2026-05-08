use super::{
    unsupported_multi_result_stdlib, StdlibFunction, StdlibMethod, NET_URL_JOIN_PATH,
    NET_URL_PARSE, NET_URL_PARSE_QUERY, NET_URL_PARSE_REQUEST_URI, NET_URL_PATH_ESCAPE,
    NET_URL_PATH_UNESCAPE, NET_URL_QUERY_ESCAPE, NET_URL_QUERY_UNESCAPE,
    NET_URL_URL_ESCAPED_FRAGMENT, NET_URL_URL_ESCAPED_PATH, NET_URL_URL_HOSTNAME,
    NET_URL_URL_IS_ABS, NET_URL_URL_JOIN_PATH, NET_URL_URL_MARSHAL_BINARY, NET_URL_URL_PARSE,
    NET_URL_URL_PORT, NET_URL_URL_QUERY, NET_URL_URL_REDACTED, NET_URL_URL_REQUEST_URI,
    NET_URL_URL_RESOLVE_REFERENCE, NET_URL_URL_STRING, NET_URL_URL_UNMARSHAL_BINARY, NET_URL_USER,
    NET_URL_USERINFO_PASSWORD, NET_URL_USERINFO_STRING, NET_URL_USERINFO_USERNAME,
    NET_URL_USER_PASSWORD, NET_URL_VALUES_ADD, NET_URL_VALUES_DEL, NET_URL_VALUES_ENCODE,
    NET_URL_VALUES_GET, NET_URL_VALUES_HAS, NET_URL_VALUES_SET,
};
use crate::{Program, Value, ValueData, Vm, VmError, TYPE_URL, TYPE_URL_PTR};

#[path = "net_url/url_methods.rs"]
mod url_methods;
#[path = "net_url/url_reference.rs"]
mod url_reference;
#[path = "net_url/url_shape.rs"]
mod url_shape;
#[path = "net_url/userinfo.rs"]
mod userinfo;
#[path = "net_url/values.rs"]
mod values;

pub(crate) use self::userinfo::url_userinfo_password;
pub(super) use self::values::url_values_encoded_text;
use self::{
    url_methods::{
        url_url_escaped_fragment, url_url_escaped_path, url_url_hostname, url_url_is_abs,
        url_url_port, url_url_query, url_url_redacted, url_url_request_uri, url_url_string,
        url_url_unmarshal_binary,
    },
    url_reference::url_url_resolve_reference,
    url_shape::{
        parse_request_uri_fields, parse_url_fields, path_escape_component, path_unescape_component,
        query_escape_component, query_unescape_component, ParsedUrlFields,
    },
    userinfo::{url_user, url_user_password, url_userinfo_string, url_userinfo_username},
    values::{
        parse_query_values, url_values_add, url_values_del, url_values_get, url_values_has,
        url_values_set,
    },
};

pub(super) const NET_URL_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: NET_URL_PARSE,
        symbol: "Parse",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_QUERY_ESCAPE,
        symbol: "QueryEscape",
        returns_value: true,
        handler: url_query_escape,
    },
    StdlibFunction {
        id: NET_URL_PARSE_QUERY,
        symbol: "ParseQuery",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_QUERY_UNESCAPE,
        symbol: "QueryUnescape",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_PATH_ESCAPE,
        symbol: "PathEscape",
        returns_value: true,
        handler: url_path_escape,
    },
    StdlibFunction {
        id: NET_URL_PATH_UNESCAPE,
        symbol: "PathUnescape",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_PARSE_REQUEST_URI,
        symbol: "ParseRequestURI",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_JOIN_PATH,
        symbol: "JoinPath",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_USER,
        symbol: "User",
        returns_value: true,
        handler: url_user,
    },
    StdlibFunction {
        id: NET_URL_USER_PASSWORD,
        symbol: "UserPassword",
        returns_value: true,
        handler: url_user_password,
    },
];

pub(super) const NET_URL_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "url.URL",
        method: "String",
        function: NET_URL_URL_STRING,
    },
    StdlibMethod {
        receiver_type: "*url.Userinfo",
        method: "String",
        function: NET_URL_USERINFO_STRING,
    },
    StdlibMethod {
        receiver_type: "*url.Userinfo",
        method: "Username",
        function: NET_URL_USERINFO_USERNAME,
    },
    StdlibMethod {
        receiver_type: "*url.Userinfo",
        method: "Password",
        function: NET_URL_USERINFO_PASSWORD,
    },
    StdlibMethod {
        receiver_type: "url.Values",
        method: "Get",
        function: NET_URL_VALUES_GET,
    },
    StdlibMethod {
        receiver_type: "url.Values",
        method: "Set",
        function: NET_URL_VALUES_SET,
    },
    StdlibMethod {
        receiver_type: "url.Values",
        method: "Add",
        function: NET_URL_VALUES_ADD,
    },
    StdlibMethod {
        receiver_type: "url.Values",
        method: "Encode",
        function: NET_URL_VALUES_ENCODE,
    },
    StdlibMethod {
        receiver_type: "url.Values",
        method: "Del",
        function: NET_URL_VALUES_DEL,
    },
    StdlibMethod {
        receiver_type: "url.Values",
        method: "Has",
        function: NET_URL_VALUES_HAS,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "EscapedPath",
        function: NET_URL_URL_ESCAPED_PATH,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "EscapedFragment",
        function: NET_URL_URL_ESCAPED_FRAGMENT,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "Query",
        function: NET_URL_URL_QUERY,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "IsAbs",
        function: NET_URL_URL_IS_ABS,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "Hostname",
        function: NET_URL_URL_HOSTNAME,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "Port",
        function: NET_URL_URL_PORT,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "RequestURI",
        function: NET_URL_URL_REQUEST_URI,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "JoinPath",
        function: NET_URL_URL_JOIN_PATH,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "Parse",
        function: NET_URL_URL_PARSE,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "ResolveReference",
        function: NET_URL_URL_RESOLVE_REFERENCE,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "Redacted",
        function: NET_URL_URL_REDACTED,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "MarshalBinary",
        function: NET_URL_URL_MARSHAL_BINARY,
    },
    StdlibMethod {
        receiver_type: "*url.URL",
        method: "UnmarshalBinary",
        function: NET_URL_URL_UNMARSHAL_BINARY,
    },
];

pub(super) const NET_URL_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: NET_URL_URL_STRING,
        symbol: "String",
        returns_value: true,
        handler: url_url_string,
    },
    StdlibFunction {
        id: NET_URL_USERINFO_STRING,
        symbol: "String",
        returns_value: true,
        handler: url_userinfo_string,
    },
    StdlibFunction {
        id: NET_URL_USERINFO_USERNAME,
        symbol: "Username",
        returns_value: true,
        handler: url_userinfo_username,
    },
    StdlibFunction {
        id: NET_URL_USERINFO_PASSWORD,
        symbol: "Password",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_VALUES_GET,
        symbol: "Get",
        returns_value: true,
        handler: url_values_get,
    },
    StdlibFunction {
        id: NET_URL_VALUES_SET,
        symbol: "Set",
        returns_value: false,
        handler: url_values_set,
    },
    StdlibFunction {
        id: NET_URL_VALUES_ADD,
        symbol: "Add",
        returns_value: false,
        handler: url_values_add,
    },
    StdlibFunction {
        id: NET_URL_VALUES_ENCODE,
        symbol: "Encode",
        returns_value: true,
        handler: url_values_encode,
    },
    StdlibFunction {
        id: NET_URL_VALUES_DEL,
        symbol: "Del",
        returns_value: false,
        handler: url_values_del,
    },
    StdlibFunction {
        id: NET_URL_VALUES_HAS,
        symbol: "Has",
        returns_value: true,
        handler: url_values_has,
    },
    StdlibFunction {
        id: NET_URL_URL_ESCAPED_PATH,
        symbol: "EscapedPath",
        returns_value: true,
        handler: url_url_escaped_path,
    },
    StdlibFunction {
        id: NET_URL_URL_ESCAPED_FRAGMENT,
        symbol: "EscapedFragment",
        returns_value: true,
        handler: url_url_escaped_fragment,
    },
    StdlibFunction {
        id: NET_URL_URL_QUERY,
        symbol: "Query",
        returns_value: true,
        handler: url_url_query,
    },
    StdlibFunction {
        id: NET_URL_URL_IS_ABS,
        symbol: "IsAbs",
        returns_value: true,
        handler: url_url_is_abs,
    },
    StdlibFunction {
        id: NET_URL_URL_HOSTNAME,
        symbol: "Hostname",
        returns_value: true,
        handler: url_url_hostname,
    },
    StdlibFunction {
        id: NET_URL_URL_PORT,
        symbol: "Port",
        returns_value: true,
        handler: url_url_port,
    },
    StdlibFunction {
        id: NET_URL_URL_REQUEST_URI,
        symbol: "RequestURI",
        returns_value: true,
        handler: url_url_request_uri,
    },
    StdlibFunction {
        id: NET_URL_URL_JOIN_PATH,
        symbol: "JoinPath",
        returns_value: true,
        handler: url_reference::url_url_join_path,
    },
    StdlibFunction {
        id: NET_URL_URL_PARSE,
        symbol: "Parse",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_URL_RESOLVE_REFERENCE,
        symbol: "ResolveReference",
        returns_value: true,
        handler: url_url_resolve_reference,
    },
    StdlibFunction {
        id: NET_URL_URL_REDACTED,
        symbol: "Redacted",
        returns_value: true,
        handler: url_url_redacted,
    },
    StdlibFunction {
        id: NET_URL_URL_MARSHAL_BINARY,
        symbol: "MarshalBinary",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_URL_URL_UNMARSHAL_BINARY,
        symbol: "UnmarshalBinary",
        returns_value: true,
        handler: url_url_unmarshal_binary,
    },
];

pub(super) fn url_parse(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    url_parse_with(vm, program, args, "url.Parse", parse_url_fields)
}

pub(super) fn url_parse_request_uri(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    url_parse_with(
        vm,
        program,
        args,
        "url.ParseRequestURI",
        parse_request_uri_fields,
    )
}

pub(super) fn url_join_path(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    url_reference::url_join_path(vm, program, args)
}

pub(super) fn url_url_marshal_binary(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    url_methods::url_url_marshal_binary(vm, program, args)
}

pub(super) fn url_url_parse(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    url_reference::url_url_parse(vm, program, args)
}

fn url_parse_with(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    builtin: &str,
    parser: fn(&str) -> Result<ParsedUrlFields, String>,
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument".into(),
        });
    };

    match parser(text) {
        Ok(parsed) => {
            let user = userinfo::userinfo_field_value(vm, parsed.user.as_ref());
            let url = Value::struct_value(
                TYPE_URL,
                vec![
                    ("Scheme".into(), Value::string(parsed.scheme)),
                    ("Opaque".into(), Value::string(parsed.opaque)),
                    ("User".into(), user),
                    ("Host".into(), Value::string(parsed.host)),
                    ("Path".into(), Value::string(parsed.path)),
                    ("RawPath".into(), Value::string(parsed.raw_path)),
                    ("ForceQuery".into(), Value::bool(parsed.force_query)),
                    ("RawQuery".into(), Value::string(parsed.raw_query)),
                    ("Fragment".into(), Value::string(parsed.fragment)),
                    ("RawFragment".into(), Value::string(parsed.raw_fragment)),
                ],
            );
            Ok(vec![vm.box_heap_value(url, TYPE_URL_PTR), Value::nil()])
        }
        Err(detail) => Ok(vec![
            Value::nil_pointer(TYPE_URL_PTR),
            Value::error(format!("parse {text:?}: {detail}")),
        ]),
    }
}

pub(super) fn url_parse_query(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.ParseQuery".into(),
            expected: "a string argument".into(),
        });
    };

    let (values, error) = parse_query_values(text);
    Ok(vec![values, error.map_or_else(Value::nil, Value::error)])
}

pub(super) fn url_query_unescape(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.QueryUnescape".into(),
            expected: "a string argument".into(),
        });
    };

    match query_unescape_component(text) {
        Ok(decoded) => Ok(vec![Value::string(decoded), Value::nil()]),
        Err(error) => Ok(vec![Value::string(String::new()), Value::error(error)]),
    }
}

pub(super) fn url_path_unescape(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.PathUnescape".into(),
            expected: "a string argument".into(),
        });
    };

    match path_unescape_component(text) {
        Ok(decoded) => Ok(vec![Value::string(decoded), Value::nil()]),
        Err(error) => Ok(vec![Value::string(String::new()), Value::error(error)]),
    }
}

fn url_query_escape(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.QueryEscape".into(),
            expected: "a string argument".into(),
        });
    };

    Ok(Value::string(query_escape_component(text)))
}

fn url_path_escape(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "url.PathEscape".into(),
            expected: "a string argument".into(),
        });
    };

    Ok(Value::string(path_escape_component(text)))
}

fn url_values_encode(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    Ok(Value::string(url_values_encoded_text(
        vm,
        program,
        "url.Values.Encode",
        "a url.Values receiver",
        &args[0],
    )?))
}
