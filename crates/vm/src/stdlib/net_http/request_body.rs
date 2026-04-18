use super::super::{
    unsupported_multi_result_stdlib, StdlibFunction, StdlibMethod, NET_HTTP_REQUEST_BODY_READ,
};
use crate::{Program, Value, ValueData, Vm, VmError, TYPE_HTTP_REQUEST_BODY};

pub(super) const REQUEST_BODY_ID_FIELD: &str = "__http_request_body_id";

pub(crate) const NET_HTTP_REQUEST_BODY_METHODS: &[StdlibMethod] = &[StdlibMethod {
    receiver_type: "http.__requestBody",
    method: "Read",
    function: NET_HTTP_REQUEST_BODY_READ,
}];

pub(crate) const NET_HTTP_REQUEST_BODY_METHOD_FUNCTIONS: &[StdlibFunction] = &[StdlibFunction {
    id: NET_HTTP_REQUEST_BODY_READ,
    symbol: "Read",
    returns_value: false,
    handler: unsupported_multi_result_stdlib,
}];

pub(crate) fn shared_request_body_value(vm: &mut Vm, body: &Value) -> Value {
    if matches!(&body.data, ValueData::Nil) || body.typ == TYPE_HTTP_REQUEST_BODY {
        return body.clone();
    }

    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.http_request_bodies.insert(
        id,
        crate::HttpRequestBodyState {
            reader: body.clone(),
        },
    );
    Value::struct_value(
        TYPE_HTTP_REQUEST_BODY,
        vec![(REQUEST_BODY_ID_FIELD.into(), Value::int(id as i64))],
    )
}

pub(crate) fn request_body_read_into(
    vm: &mut Vm,
    program: &Program,
    body: &Value,
    buffer: &mut Value,
) -> Result<Vec<Value>, VmError> {
    let body_id = extract_request_body_id(vm, program, "(*http.RequestBody).Read", body)?;
    let Some(state) = vm.http_request_bodies.get(&body_id) else {
        return Err(invalid_request_body_argument(
            vm,
            program,
            "(*http.RequestBody).Read",
        ));
    };
    let reader = state.reader.clone();
    let (results, reader, mutated_buffer) = vm.invoke_method_results_mutating_receiver_and_arg(
        program,
        reader,
        "Read",
        vec![buffer.clone()],
        0,
    )?;
    if let Some(state) = vm.http_request_bodies.get_mut(&body_id) {
        state.reader = reader;
    }
    *buffer = mutated_buffer;
    Ok(results)
}

pub(crate) fn request_body_id(value: &Value) -> Option<u64> {
    if value.typ != TYPE_HTTP_REQUEST_BODY {
        return None;
    }
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };
    let (_, id) = fields
        .iter()
        .find(|(name, _)| name == REQUEST_BODY_ID_FIELD)?;
    let ValueData::Int(id) = &id.data else {
        return None;
    };
    u64::try_from(*id).ok()
}

fn extract_request_body_id(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<u64, VmError> {
    request_body_id(value).ok_or_else(|| invalid_request_body_argument(vm, program, builtin))
}

fn invalid_request_body_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<engine>".into()),
        builtin: builtin.into(),
        expected: "a valid http.__requestBody receiver".into(),
    }
}
