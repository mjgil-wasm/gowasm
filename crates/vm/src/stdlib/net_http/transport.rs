use std::collections::BTreeMap;

mod request_body;
mod response_body;

use self::request_body::{request_body_upload_request, RequestBodyError};
pub(crate) use self::response_body::response_body_read;
pub(crate) use self::response_body::response_body_read_into;
use self::response_body::{fetch_response_results, http_header_value, response_body_close};
use super::super::{
    context_impl, net_url_impl, unsupported_multi_result_stdlib, StdlibFunction, StdlibMethod,
    NET_HTTP_CLIENT_DO, NET_HTTP_CLIENT_GET, NET_HTTP_CLIENT_HEAD, NET_HTTP_CLIENT_POST,
    NET_HTTP_CLIENT_POST_FORM, NET_HTTP_RESPONSE_BODY_CLOSE, NET_HTTP_RESPONSE_BODY_READ,
};
use super::core::canonicalize_header_key_text;
use crate::{
    CapabilityRequest, FetchHeader, FetchRequest, Program, Value, ValueData, Vm, VmError,
    TYPE_HTTP_CLIENT, TYPE_HTTP_REQUEST, TYPE_HTTP_RESPONSE_PTR,
};

const POST_FORM_CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
pub(super) const RESPONSE_BODY_ID_FIELD: &str = "__http_response_body_id";

pub(crate) const NET_HTTP_TRANSPORT_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "*http.Client",
        method: "Do",
        function: NET_HTTP_CLIENT_DO,
    },
    StdlibMethod {
        receiver_type: "*http.Client",
        method: "Get",
        function: NET_HTTP_CLIENT_GET,
    },
    StdlibMethod {
        receiver_type: "*http.Client",
        method: "Head",
        function: NET_HTTP_CLIENT_HEAD,
    },
    StdlibMethod {
        receiver_type: "*http.Client",
        method: "Post",
        function: NET_HTTP_CLIENT_POST,
    },
    StdlibMethod {
        receiver_type: "*http.Client",
        method: "PostForm",
        function: NET_HTTP_CLIENT_POST_FORM,
    },
    StdlibMethod {
        receiver_type: "http.__responseBody",
        method: "Read",
        function: NET_HTTP_RESPONSE_BODY_READ,
    },
    StdlibMethod {
        receiver_type: "http.__responseBody",
        method: "Close",
        function: NET_HTTP_RESPONSE_BODY_CLOSE,
    },
];

pub(crate) const NET_HTTP_TRANSPORT_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: NET_HTTP_CLIENT_DO,
        symbol: "Do",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_CLIENT_GET,
        symbol: "Get",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_CLIENT_HEAD,
        symbol: "Head",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_CLIENT_POST,
        symbol: "Post",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_CLIENT_POST_FORM,
        symbol: "PostForm",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_RESPONSE_BODY_READ,
        symbol: "Read",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_RESPONSE_BODY_CLOSE,
        symbol: "Close",
        returns_value: true,
        handler: response_body_close,
    },
];

pub(crate) fn get(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    dispatch_url_method_request(
        vm,
        program,
        "http.Get",
        "GET",
        &args[0],
        &Value::nil(),
        None,
    )
}

pub(crate) fn head(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    dispatch_url_method_request(
        vm,
        program,
        "http.Head",
        "HEAD",
        &args[0],
        &Value::nil(),
        None,
    )
}

fn dispatch_built_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    request: super::request::RequestBuild,
) -> Result<Vec<Value>, VmError> {
    dispatch_built_request_with_body(vm, program, builtin, request, None)
}

fn dispatch_built_request_with_body(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    request: super::request::RequestBuild,
    prefilled_body: Option<Vec<u8>>,
) -> Result<Vec<Value>, VmError> {
    match request {
        super::request::RequestBuild::Ready(request) => {
            dispatch_request_with_body(vm, program, builtin, &request, prefilled_body)
        }
        super::request::RequestBuild::ReturnedError(err) => {
            Ok(vec![Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR), err])
        }
    }
}

pub(crate) fn post(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Vec<Value>, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let request = post_request(vm, program, "http.Post", &args[0], &args[1], &args[2])?;
    dispatch_built_request(vm, program, "http.Post", request)
}

pub(crate) fn post_form(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    const BUILTIN: &str = "http.PostForm";

    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let (request, body) = post_form_request(vm, program, BUILTIN, &args[0], &args[1])?;
    dispatch_built_request_with_body(vm, program, BUILTIN, request, Some(body))
}

fn post_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    url: &Value,
    content_type: &Value,
    body: &Value,
) -> Result<super::request::RequestBuild, VmError> {
    let ValueData::String(_) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string, string, io.Reader arguments".into(),
        });
    };
    let ValueData::String(content_type) = &content_type.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string, string, io.Reader arguments".into(),
        });
    };

    let headers = if content_type.is_empty() {
        Vec::new()
    } else {
        vec![FetchHeader {
            name: canonicalize_header_key_text("Content-Type"),
            values: vec![content_type.clone()],
        }]
    };

    dispatch_url_method_request_build(
        vm,
        program,
        builtin,
        "POST",
        url,
        body,
        Some(http_header_value(headers)),
    )
}

fn post_form_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    url: &Value,
    values: &Value,
) -> Result<(super::request::RequestBuild, Vec<u8>), VmError> {
    let ValueData::String(_) = &url.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string, url.Values arguments".into(),
        });
    };

    let body = net_url_impl::url_values_encoded_text(
        vm,
        program,
        builtin,
        "string, url.Values arguments",
        values,
    )?
    .into_bytes();

    Ok((
        dispatch_url_method_request_build(
            vm,
            program,
            builtin,
            "POST",
            url,
            &Value::nil(),
            Some(http_header_value(vec![FetchHeader {
                name: canonicalize_header_key_text("Content-Type"),
                values: vec![POST_FORM_CONTENT_TYPE.into()],
            }])),
        )?,
        body,
    ))
}

fn dispatch_url_method_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    method: &str,
    url: &Value,
    body: &Value,
    header: Option<Value>,
) -> Result<Vec<Value>, VmError> {
    let request =
        dispatch_url_method_request_build(vm, program, builtin, method, url, body, header)?;
    dispatch_built_request(vm, program, builtin, request)
}

fn dispatch_url_method_request_build(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    method: &str,
    url: &Value,
    body: &Value,
    header: Option<Value>,
) -> Result<super::request::RequestBuild, VmError> {
    super::request::build_request_value(
        vm,
        program,
        builtin,
        &Value::string(method),
        url,
        body,
        None,
        header,
    )
}

pub(crate) fn client_do(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    const BUILTIN: &str = "(*http.Client).Do";

    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let _ = client_receiver_value(vm, program, BUILTIN, &args[0])?;
    let request = request_receiver_value(vm, program, BUILTIN, &args[1])?;
    dispatch_request(vm, program, BUILTIN, &request)
}

pub(crate) fn client_get(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    const BUILTIN: &str = "(*http.Client).Get";

    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let _ = client_receiver_value(vm, program, BUILTIN, &args[0])?;
    dispatch_url_method_request(vm, program, BUILTIN, "GET", &args[1], &Value::nil(), None)
}

pub(crate) fn client_head(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    const BUILTIN: &str = "(*http.Client).Head";

    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let _ = client_receiver_value(vm, program, BUILTIN, &args[0])?;
    dispatch_url_method_request(vm, program, BUILTIN, "HEAD", &args[1], &Value::nil(), None)
}

pub(crate) fn client_post(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    const BUILTIN: &str = "(*http.Client).Post";

    if args.len() != 4 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 4,
            actual: args.len(),
        });
    }

    let _ = client_receiver_value(vm, program, BUILTIN, &args[0])?;
    let request = post_request(vm, program, BUILTIN, &args[1], &args[2], &args[3])?;
    dispatch_built_request(vm, program, BUILTIN, request)
}

pub(crate) fn client_post_form(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    const BUILTIN: &str = "(*http.Client).PostForm";

    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }

    let _ = client_receiver_value(vm, program, BUILTIN, &args[0])?;
    let (request, body) = post_form_request(vm, program, BUILTIN, &args[1], &args[2])?;
    dispatch_built_request_with_body(vm, program, BUILTIN, request, Some(body))
}

fn dispatch_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    request: &Value,
) -> Result<Vec<Value>, VmError> {
    dispatch_request_with_body(vm, program, builtin, request, None)
}

fn dispatch_request_with_body(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    request: &Value,
    prefilled_body: Option<Vec<u8>>,
) -> Result<Vec<Value>, VmError> {
    if request.typ != TYPE_HTTP_REQUEST {
        return Err(invalid_request_argument(vm, program, builtin));
    }
    let ValueData::Struct(fields) = &request.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };

    let method = match super::request::normalized_method(request_string_field(
        vm, program, builtin, fields, "Method",
    )?) {
        Ok(method) => method,
        Err(message) => {
            return Ok(vec![
                Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR),
                Value::error(format!("net/http: {message}")),
            ]);
        }
    };
    let Some(url) = request_url_text(
        vm,
        program,
        builtin,
        request_field(vm, program, builtin, fields, "URL")?,
    )?
    else {
        return Ok(vec![
            Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR),
            Value::error("net/http: nil Request.URL"),
        ]);
    };
    let headers = request_header_values(
        vm,
        program,
        builtin,
        request_field(vm, program, builtin, fields, "Header")?,
    )?;
    let body_value = request_field(vm, program, builtin, fields, "Body")?.clone();
    let request_context = super::request::request_context_value(fields);

    if let Some(results) = fetch_response_results(vm) {
        vm.set_http_request_upload_state(None);
        return Ok(results);
    }

    if let Some(error) = super::request::request_context_error(vm, program, builtin, fields)? {
        vm.set_pending_http_request_context(None);
        return Ok(vec![Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR), error]);
    }

    let context_deadline_unix_millis =
        request_context_deadline_unix_millis(vm, program, builtin, request_context.as_ref())?;

    if !vm.capability_requests_enabled() {
        vm.set_pending_http_request_context(None);
        return Ok(missing_fetch_capability(builtin));
    }

    vm.set_pending_http_request_context(request_context);

    let capability = match prefilled_body {
        Some(body) => CapabilityRequest::Fetch {
            request: FetchRequest {
                method,
                url,
                headers,
                body,
                context_deadline_unix_millis,
            },
        },
        None if matches!(&body_value.data, ValueData::Nil) => CapabilityRequest::Fetch {
            request: FetchRequest {
                method,
                url,
                headers,
                body: Vec::new(),
                context_deadline_unix_millis,
            },
        },
        None => match request_body_upload_request(
            vm,
            program,
            builtin,
            &method,
            &url,
            &headers,
            &body_value,
            context_deadline_unix_millis,
        ) {
            Ok(capability) => capability,
            Err(RequestBodyError::Returned(err)) => {
                vm.set_pending_http_request_context(None);
                return Ok(vec![Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR), err]);
            }
            Err(RequestBodyError::Fatal(err)) => return Err(err),
        },
    };

    Err(VmError::CapabilityRequest { kind: capability })
}

fn missing_fetch_capability(builtin: &str) -> Vec<Value> {
    vec![
        Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR),
        Value::error(format!(
            "net/http: {builtin} requires a host-provided fetch capability"
        )),
    ]
}

fn request_context_deadline_unix_millis(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    context: Option<&Value>,
) -> Result<Option<i64>, VmError> {
    let Some(context) = context else {
        return Ok(None);
    };
    let deadline_unix_nanos =
        context_impl::context_value_deadline_unix_nanos(vm, program, builtin, context)?;
    Ok(deadline_unix_nanos.map(unix_nanos_to_deadline_millis))
}

fn unix_nanos_to_deadline_millis(unix_nanos: i64) -> i64 {
    if unix_nanos <= 0 {
        0
    } else {
        ((unix_nanos - 1) / 1_000_000) + 1
    }
}

fn client_receiver_value(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    receiver: &Value,
) -> Result<Value, VmError> {
    let client = vm.deref_pointer(program, receiver)?;
    if client.typ != TYPE_HTTP_CLIENT {
        return Err(invalid_client_argument(vm, program, builtin));
    }
    let ValueData::Struct(_) = &client.data else {
        return Err(invalid_client_argument(vm, program, builtin));
    };
    Ok(client)
}

fn request_receiver_value(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    receiver: &Value,
) -> Result<Value, VmError> {
    let request = vm.deref_pointer(program, receiver)?;
    if request.typ != TYPE_HTTP_REQUEST {
        return Err(invalid_request_argument(vm, program, builtin));
    }
    let ValueData::Struct(_) = &request.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };
    Ok(request)
}

fn request_field<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    fields: &'a [(String, Value)],
    name: &str,
) -> Result<&'a Value, VmError> {
    fields
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
        .ok_or_else(|| invalid_request_argument(vm, program, builtin))
}

fn request_string_field<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    fields: &'a [(String, Value)],
    name: &str,
) -> Result<&'a str, VmError> {
    let value = request_field(vm, program, builtin, fields, name)?;
    let ValueData::String(text) = &value.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };
    Ok(text)
}

fn request_url_text(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Option<String>, VmError> {
    let ValueData::Pointer(pointer) = &value.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };
    if pointer.is_nil() {
        return Ok(None);
    }

    let rendered = vm.invoke_method(program, value.clone(), "String", Vec::new())?;
    let ValueData::String(text) = rendered.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };
    Ok(Some(text))
}

fn request_header_values(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Vec<FetchHeader>, VmError> {
    let ValueData::Map(map) = &value.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };
    let Some(entries) = &map.entries else {
        return Ok(Vec::new());
    };
    let entries = entries.borrow();

    let mut canonicalized = BTreeMap::<String, Vec<String>>::new();
    for (name, values) in entries.iter() {
        let ValueData::String(name) = &name.data else {
            return Err(invalid_request_argument(vm, program, builtin));
        };
        let ValueData::Slice(values) = &values.data else {
            return Err(invalid_request_argument(vm, program, builtin));
        };
        let entry = canonicalized
            .entry(canonicalize_header_key_text(name))
            .or_default();
        let visible = values.values_snapshot();
        for value in &visible {
            let ValueData::String(value) = &value.data else {
                return Err(invalid_request_argument(vm, program, builtin));
            };
            entry.push(value.clone());
        }
    }

    Ok(canonicalized
        .into_iter()
        .map(|(name, values)| FetchHeader { name, values })
        .collect())
}

fn invalid_client_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a valid *http.Client receiver".into(),
    }
}

fn invalid_request_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a valid *http.Request receiver".into(),
    }
}
