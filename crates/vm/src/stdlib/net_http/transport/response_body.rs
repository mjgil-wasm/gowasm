use std::collections::BTreeMap;

use super::super::{
    core::{canonicalize_header_key_text, status_text_for_code},
    response::RESPONSE_URL_FIELD,
};
use super::RESPONSE_BODY_ID_FIELD;
use crate::stdlib::context_impl;
use crate::{
    CapabilityRequest, ConcreteType, FetchHeader, FetchResponse, FetchResponseChunkRequest,
    FetchResponseCloseRequest, FetchResponseStart, FetchResult, HttpResponseBodyState, MapValue,
    Program, Value, ValueData, Vm, VmError, TYPE_HTTP_HEADER, TYPE_HTTP_RESPONSE,
    TYPE_HTTP_RESPONSE_BODY, TYPE_HTTP_RESPONSE_PTR,
};

pub(crate) fn response_body_read(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let mut buffer = args[1].clone();
    response_body_read_into(vm, program, &args[0], &mut buffer)
}

pub(crate) fn response_body_read_into(
    vm: &mut Vm,
    program: &Program,
    body: &Value,
    buffer: &mut Value,
) -> Result<Vec<Value>, VmError> {
    let body_id = extract_response_body_id(vm, program, "(*http.ResponseBody).Read", body)?;
    let Some((is_closed, request_context)) = vm
        .http_response_bodies
        .get(&body_id)
        .map(|state| (state.closed, state.request_context.clone()))
    else {
        return Err(invalid_response_body_argument(
            vm,
            program,
            "(*http.ResponseBody).Read",
        ));
    };
    if is_closed {
        return Ok(vec![
            Value::int(0),
            Value::error("http: read on closed response body"),
        ]);
    }
    if let Some(error) = response_body_context_error(vm, program, request_context.as_ref())? {
        if let Some(state) = vm.http_response_bodies.get_mut(&body_id) {
            state.buffered.clear();
            state.read_offset = 0;
            state.eof = false;
            state.terminal_error = Some(error.clone());
        }
        return Ok(vec![Value::int(0), error]);
    }

    let ValueData::Slice(slice) = &buffer.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "(*http.ResponseBody).Read".into(),
            expected: "a response body value and []byte buffer".into(),
        });
    };
    if slice.is_empty() {
        return Ok(vec![Value::int(0), Value::nil()]);
    }

    if let Some(read_count) = copy_response_body_bytes(vm, body_id, slice) {
        return Ok(vec![Value::int(read_count as i64), Value::nil()]);
    }

    let Some(state) = vm.http_response_bodies.get(&body_id) else {
        return Err(invalid_response_body_argument(
            vm,
            program,
            "(*http.ResponseBody).Read",
        ));
    };

    if let Some(err) = state.terminal_error.clone() {
        return Ok(vec![Value::int(0), err]);
    }

    if state.eof {
        return Ok(vec![Value::int(0), Value::error("EOF")]);
    }

    let Some(session_id) = state.session_id else {
        return Ok(vec![Value::int(0), Value::error("EOF")]);
    };

    Err(VmError::CapabilityRequest {
        kind: CapabilityRequest::FetchResponseChunk {
            request: FetchResponseChunkRequest {
                session_id,
                max_bytes: u32::try_from(slice.len()).unwrap_or(u32::MAX),
            },
        },
    })
}

pub(super) fn response_body_close(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let body_id = extract_response_body_id(vm, program, "(*http.ResponseBody).Close", &args[0])?;
    let Some(state) = vm.http_response_bodies.get(&body_id) else {
        return Err(invalid_response_body_argument(
            vm,
            program,
            "(*http.ResponseBody).Close",
        ));
    };
    if state.closed {
        return Ok(Value::nil());
    }
    if let Some(session_id) = state.session_id {
        return Err(VmError::CapabilityRequest {
            kind: CapabilityRequest::FetchResponseClose {
                request: FetchResponseCloseRequest { session_id },
            },
        });
    }
    if let Some(state) = vm.http_response_bodies.get_mut(&body_id) {
        state.closed = true;
    }
    Ok(Value::nil())
}

pub(super) fn fetch_response_results(vm: &mut Vm) -> Option<Vec<Value>> {
    if let Some(pending) = vm.take_fetch_response_start() {
        let request_context = vm.take_pending_http_request_context();
        let response = fetch_streamed_response_value(
            vm,
            pending.session_id,
            pending.response,
            request_context,
        );
        return Some(vec![
            vm.box_heap_value(response, TYPE_HTTP_RESPONSE_PTR),
            Value::nil(),
        ]);
    }

    match vm.take_fetch_result()? {
        FetchResult::Response { response } => {
            vm.set_pending_http_request_context(None);
            let response = fetch_response_value(vm, response);
            Some(vec![
                vm.box_heap_value(response, TYPE_HTTP_RESPONSE_PTR),
                Value::nil(),
            ])
        }
        FetchResult::Error { message } => {
            vm.set_pending_http_request_context(None);
            Some(vec![
                Value::nil_pointer(TYPE_HTTP_RESPONSE_PTR),
                Value::error(normalize_fetch_error_text(&message)),
            ])
        }
    }
}

pub(super) fn http_header_value(headers: Vec<FetchHeader>) -> Value {
    let mut canonicalized = BTreeMap::<String, Vec<String>>::new();
    for header in headers {
        let name = canonicalize_header_key_text(&header.name);
        canonicalized.entry(name).or_default().extend(header.values);
    }

    Value {
        typ: TYPE_HTTP_HEADER,
        data: ValueData::Map(MapValue::with_entries(
            canonicalized
                .into_iter()
                .map(|(name, values)| {
                    (
                        Value::string(name),
                        Value::slice(values.into_iter().map(Value::string).collect()),
                    )
                })
                .collect(),
            Value::nil_slice(),
            Some(ConcreteType::TypeId(TYPE_HTTP_HEADER)),
        )),
    }
}

fn fetch_response_value(vm: &mut Vm, response: FetchResponse) -> Value {
    let FetchResponse {
        status_code,
        status,
        url,
        headers,
        body,
    } = response;
    Value::struct_value(
        TYPE_HTTP_RESPONSE,
        vec![
            (
                "Status".into(),
                Value::string(normalize_fetch_status(status_code, &status)),
            ),
            ("StatusCode".into(), Value::int(status_code)),
            ("Header".into(), http_header_value(headers)),
            ("Body".into(), buffered_response_body_value(vm, body)),
            (RESPONSE_URL_FIELD.into(), Value::string(url)),
        ],
    )
}

fn fetch_streamed_response_value(
    vm: &mut Vm,
    session_id: u64,
    response: FetchResponseStart,
    request_context: Option<Value>,
) -> Value {
    let FetchResponseStart {
        status_code,
        status,
        url,
        headers,
    } = response;
    Value::struct_value(
        TYPE_HTTP_RESPONSE,
        vec![
            (
                "Status".into(),
                Value::string(normalize_fetch_status(status_code, &status)),
            ),
            ("StatusCode".into(), Value::int(status_code)),
            ("Header".into(), http_header_value(headers)),
            (
                "Body".into(),
                streaming_response_body_value(vm, session_id, request_context),
            ),
            (RESPONSE_URL_FIELD.into(), Value::string(url)),
        ],
    )
}

fn normalize_fetch_status(status_code: i64, status: &str) -> String {
    let numeric_status = status_code.to_string();
    if !status.is_empty() && status != numeric_status {
        return status.to_string();
    }

    let reason = status_text_for_code(status_code);
    if reason.is_empty() {
        numeric_status
    } else {
        format!("{numeric_status} {reason}")
    }
}

fn buffered_response_body_value(vm: &mut Vm, body: Vec<u8>) -> Value {
    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.http_response_bodies.insert(
        id,
        HttpResponseBodyState {
            buffered: body,
            read_offset: 0,
            closed: false,
            session_id: None,
            eof: true,
            terminal_error: None,
            request_context: None,
        },
    );
    Value::struct_value(
        TYPE_HTTP_RESPONSE_BODY,
        vec![(RESPONSE_BODY_ID_FIELD.into(), Value::int(id as i64))],
    )
}

fn streaming_response_body_value(
    vm: &mut Vm,
    session_id: u64,
    request_context: Option<Value>,
) -> Value {
    vm.next_stdlib_object_id += 1;
    let id = vm.next_stdlib_object_id;
    vm.http_response_bodies.insert(
        id,
        HttpResponseBodyState {
            buffered: Vec::new(),
            read_offset: 0,
            closed: false,
            session_id: Some(session_id),
            eof: false,
            terminal_error: None,
            request_context,
        },
    );
    Value::struct_value(
        TYPE_HTTP_RESPONSE_BODY,
        vec![(RESPONSE_BODY_ID_FIELD.into(), Value::int(id as i64))],
    )
}

fn copy_response_body_bytes(
    vm: &mut Vm,
    body_id: u64,
    buffer: &crate::SliceValue,
) -> Option<usize> {
    let state = vm.http_response_bodies.get_mut(&body_id)?;
    if state.read_offset >= state.buffered.len() {
        return None;
    }

    let read_count = buffer
        .len()
        .min(state.buffered.len().saturating_sub(state.read_offset));
    for (index, byte) in state.buffered[state.read_offset..]
        .iter()
        .take(read_count)
        .enumerate()
    {
        assert!(
            buffer.set(index, Value::int(i64::from(*byte))),
            "response body copy should stay within the provided buffer window"
        );
    }
    state.read_offset += read_count;

    if state.read_offset >= state.buffered.len() {
        state.buffered.clear();
        state.read_offset = 0;
    }

    Some(read_count)
}

fn extract_response_body_id(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<u64, VmError> {
    if value.typ != TYPE_HTTP_RESPONSE_BODY {
        return Err(invalid_response_body_argument(vm, program, builtin));
    }
    let ValueData::Struct(fields) = &value.data else {
        return Err(invalid_response_body_argument(vm, program, builtin));
    };
    fields
        .iter()
        .find(|(name, _)| name == RESPONSE_BODY_ID_FIELD)
        .and_then(|(_, value)| match &value.data {
            ValueData::Int(id) if *id >= 0 => Some(*id as u64),
            _ => None,
        })
        .ok_or_else(|| invalid_response_body_argument(vm, program, builtin))
}

fn invalid_response_body_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a valid http response body receiver".into(),
    }
}

fn response_body_context_error(
    vm: &mut Vm,
    program: &Program,
    request_context: Option<&Value>,
) -> Result<Option<Value>, VmError> {
    let Some(request_context) = request_context else {
        return Ok(None);
    };
    context_impl::context_value_error(vm, program, "(*http.ResponseBody).Read", request_context)
}

fn normalize_fetch_error_text(message: &str) -> String {
    if is_context_transport_error(message) {
        message.to_string()
    } else {
        format!("net/http: {message}")
    }
}

fn is_context_transport_error(message: &str) -> bool {
    matches!(message, "context canceled" | "context deadline exceeded")
}
