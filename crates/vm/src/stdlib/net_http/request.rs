use super::super::{
    context_impl, net_url_impl, StdlibFunction, StdlibMethod, NET_HTTP_REQUEST_CLONE,
    NET_HTTP_REQUEST_CONTEXT, NET_HTTP_REQUEST_WITH_CONTEXT,
};
use super::request_body::shared_request_body_value;
use crate::{
    ConcreteType, MapValue, Program, Value, ValueData, Vm, VmError, TYPE_HTTP_HEADER,
    TYPE_HTTP_REQUEST, TYPE_HTTP_REQUEST_PTR, TYPE_URL, TYPE_URL_PTR,
};

const REQUEST_CONTEXT_FIELD: &str = "__context";

pub(super) enum RequestBuild {
    Ready(Value),
    ReturnedError(Value),
}

pub(crate) const NET_HTTP_REQUEST_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "*http.Request",
        method: "Context",
        function: NET_HTTP_REQUEST_CONTEXT,
    },
    StdlibMethod {
        receiver_type: "*http.Request",
        method: "WithContext",
        function: NET_HTTP_REQUEST_WITH_CONTEXT,
    },
    StdlibMethod {
        receiver_type: "*http.Request",
        method: "Clone",
        function: NET_HTTP_REQUEST_CLONE,
    },
];

pub(crate) const NET_HTTP_REQUEST_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: NET_HTTP_REQUEST_CONTEXT,
        symbol: "Context",
        returns_value: true,
        handler: request_context,
    },
    StdlibFunction {
        id: NET_HTTP_REQUEST_WITH_CONTEXT,
        symbol: "WithContext",
        returns_value: true,
        handler: request_with_context,
    },
    StdlibFunction {
        id: NET_HTTP_REQUEST_CLONE,
        symbol: "Clone",
        returns_value: true,
        handler: request_clone,
    },
];

pub(crate) fn new_request(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    build_request(
        vm,
        program,
        "http.NewRequest",
        &args[0],
        &args[1],
        &args[2],
        None,
    )
}

pub(crate) fn new_request_with_context(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 4 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 4,
            actual: args.len(),
        });
    }
    if matches!(&args[0].data, ValueData::Nil) {
        return Ok(vec![
            Value::nil_pointer(TYPE_HTTP_REQUEST_PTR),
            Value::error("net/http: nil Context"),
        ]);
    }
    build_request(
        vm,
        program,
        "http.NewRequestWithContext",
        &args[1],
        &args[2],
        &args[3],
        Some(args[0].clone()),
    )
}

fn build_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    method_arg: &Value,
    url_arg: &Value,
    body: &Value,
    context: Option<Value>,
) -> Result<Vec<Value>, VmError> {
    Ok(
        match build_request_value(
            vm, program, builtin, method_arg, url_arg, body, context, None,
        )? {
            RequestBuild::Ready(request) => {
                vec![
                    vm.box_heap_value(request, TYPE_HTTP_REQUEST_PTR),
                    Value::nil(),
                ]
            }
            RequestBuild::ReturnedError(err) => {
                vec![Value::nil_pointer(TYPE_HTTP_REQUEST_PTR), err]
            }
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_request_value(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    method_arg: &Value,
    url_arg: &Value,
    body: &Value,
    context: Option<Value>,
    header: Option<Value>,
) -> Result<RequestBuild, VmError> {
    let ValueData::String(method_arg) = &method_arg.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string, string, io.Reader arguments".into(),
        });
    };
    let ValueData::String(url_arg) = &url_arg.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string, string, io.Reader arguments".into(),
        });
    };

    let method = match normalized_method(method_arg) {
        Ok(method) => method,
        Err(message) => {
            return Ok(RequestBuild::ReturnedError(Value::error(format!(
                "net/http: {message}"
            ))));
        }
    };

    let parsed_url = net_url_impl::url_parse(vm, program, &[Value::string(url_arg.clone())])?;
    if !matches!(
        parsed_url.get(1).map(|value| &value.data),
        Some(ValueData::Nil)
    ) {
        return Ok(RequestBuild::ReturnedError(parsed_url[1].clone()));
    }

    Ok(RequestBuild::Ready(Value::struct_value(
        TYPE_HTTP_REQUEST,
        request_fields(
            ("Method".into(), Value::string(method)),
            ("URL".into(), parsed_url[0].clone()),
            ("Header".into(), header.unwrap_or_else(empty_header_value)),
            ("Body".into(), shared_request_body_value(vm, body)),
            context,
        ),
    )))
}

fn request_context(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let request = request_receiver_value(vm, program, "(*http.Request).Context", &args[0])?;
    let ValueData::Struct(fields) = &request.data else {
        return Err(invalid_request_argument(
            vm,
            program,
            "(*http.Request).Context",
        ));
    };
    if let Some((_, context)) = fields
        .iter()
        .find(|(name, _)| name == REQUEST_CONTEXT_FIELD)
    {
        return Ok(context.clone());
    }

    context_impl::context_background(vm, program, &[])
}

fn request_with_context(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    if matches!(&args[1].data, ValueData::Nil) {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "nil context".into(),
        });
    }

    let request = request_receiver_value(vm, program, "(*http.Request).WithContext", &args[0])?;
    let ValueData::Struct(mut fields) = request.data else {
        return Err(invalid_request_argument(
            vm,
            program,
            "(*http.Request).WithContext",
        ));
    };
    maybe_share_request_body(vm, &mut fields);
    set_request_context(&mut fields, args[1].clone());
    Ok(vm.box_heap_value(
        Value::struct_value(TYPE_HTTP_REQUEST, fields),
        TYPE_HTTP_REQUEST_PTR,
    ))
}

fn request_clone(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    if matches!(&args[1].data, ValueData::Nil) {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "nil context".into(),
        });
    }

    let request = request_receiver_value(vm, program, "(*http.Request).Clone", &args[0])?;
    let ValueData::Struct(fields) = &request.data else {
        return Err(invalid_request_argument(
            vm,
            program,
            "(*http.Request).Clone",
        ));
    };
    let mut cloned_fields = clone_request_fields(vm, program, "(*http.Request).Clone", fields)?;
    set_request_context(&mut cloned_fields, args[1].clone());
    Ok(vm.box_heap_value(
        Value::struct_value(TYPE_HTTP_REQUEST, cloned_fields),
        TYPE_HTTP_REQUEST_PTR,
    ))
}

fn empty_header_value() -> Value {
    Value {
        typ: TYPE_HTTP_HEADER,
        data: ValueData::Map(MapValue::with_entries(
            Vec::new(),
            Value::nil_slice(),
            Some(ConcreteType::TypeId(TYPE_HTTP_HEADER)),
        )),
    }
}

pub(super) fn normalized_method(method: &str) -> Result<String, String> {
    if method.is_empty() {
        return Ok("GET".into());
    }
    if method.bytes().all(valid_method_byte) {
        return Ok(method.into());
    }
    Err(format!("invalid method {method:?}"))
}

pub(super) fn request_context_error(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    fields: &[(String, Value)],
) -> Result<Option<Value>, VmError> {
    let Some(context) = request_context_value(fields) else {
        return Ok(None);
    };
    context_impl::context_value_error(vm, program, builtin, &context)
}

pub(super) fn request_context_value(fields: &[(String, Value)]) -> Option<Value> {
    fields
        .iter()
        .find(|(name, _)| name == REQUEST_CONTEXT_FIELD)
        .map(|(_, context)| context.clone())
}

fn valid_method_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\''
            | b'*' | b'+' | b'-' | b'.' | b'^' | b'_'
            | b'`' | b'|' | b'~'
            | b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
    )
}

fn request_fields(
    method: (String, Value),
    url: (String, Value),
    header: (String, Value),
    body: (String, Value),
    context: Option<Value>,
) -> Vec<(String, Value)> {
    let mut fields = vec![method, url, header, body];
    if let Some(context) = context {
        set_request_context(&mut fields, context);
    }
    fields
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

fn set_request_context(fields: &mut Vec<(String, Value)>, context: Value) {
    if let Some((_, value)) = fields
        .iter_mut()
        .find(|(name, _)| name == REQUEST_CONTEXT_FIELD)
    {
        *value = context;
        return;
    }
    fields.push((REQUEST_CONTEXT_FIELD.into(), context));
}

fn clone_request_fields(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    fields: &[(String, Value)],
) -> Result<Vec<(String, Value)>, VmError> {
    let mut cloned = Vec::with_capacity(fields.len());
    for (name, value) in fields {
        let value = if name == "URL" {
            clone_request_url(vm, program, builtin, value)?
        } else if name == "Header" {
            clone_request_header(vm, program, builtin, value)?
        } else if name == "Body" {
            shared_request_body_value(vm, value)
        } else {
            value.clone()
        };
        cloned.push((name.clone(), value));
    }
    Ok(cloned)
}

fn clone_request_url(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    let ValueData::Pointer(pointer) = &value.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };
    if pointer.is_nil() {
        return Ok(Value::nil_pointer(TYPE_URL_PTR));
    }

    let url = vm.deref_pointer(program, value)?;
    if url.typ != TYPE_URL {
        return Err(invalid_request_argument(vm, program, builtin));
    }
    Ok(vm.box_heap_value(url, TYPE_URL_PTR))
}

fn clone_request_header(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<Value, VmError> {
    if value.typ != TYPE_HTTP_HEADER {
        return Err(invalid_request_argument(vm, program, builtin));
    }
    let ValueData::Map(header) = &value.data else {
        return Err(invalid_request_argument(vm, program, builtin));
    };

    Ok(Value {
        typ: TYPE_HTTP_HEADER,
        data: ValueData::Map(if header.is_nil() {
            MapValue::nil((*header.zero_value).clone(), header.concrete_type.clone())
        } else {
            MapValue::with_entries(
                header.entries_snapshot(),
                (*header.zero_value).clone(),
                header.concrete_type.clone(),
            )
        }),
    })
}

fn maybe_share_request_body(vm: &mut Vm, fields: &mut [(String, Value)]) {
    if let Some((_, body)) = fields.iter_mut().find(|(name, _)| name == "Body") {
        *body = shared_request_body_value(vm, body);
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
