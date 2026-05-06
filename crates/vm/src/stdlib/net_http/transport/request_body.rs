use crate::{
    CapabilityRequest, FetchBodyAbortRequest, FetchBodyChunkRequest, FetchBodyCompleteRequest,
    FetchHeader, FetchStartRequest, HttpRequestUploadPhase, HttpRequestUploadState, Program, Value,
    ValueData, Vm, VmError,
};

const REQUEST_BODY_BUFFER_SIZE: usize = 8 * 1024;

pub(super) enum RequestBodyError {
    Returned(Value),
    Fatal(VmError),
}

enum RequestBodyStep {
    Chunk { bytes: Vec<u8>, eof: bool },
    Complete,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn request_body_upload_request(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    method: &str,
    url: &str,
    headers: &[FetchHeader],
    body: &Value,
    context_deadline_unix_millis: Option<i64>,
) -> Result<CapabilityRequest, RequestBodyError> {
    let Some(upload) = vm.http_request_upload_state() else {
        if !matches!(&body.data, ValueData::Nil) {
            match vm.supports_method_results_mutating_arg(program, body.clone(), "Read", 0, 1) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(RequestBodyError::Returned(Value::error(format!(
                        "net/http: {builtin} body must implement io.Reader"
                    ))));
                }
                Err(VmError::UnknownMethod { .. })
                | Err(VmError::UnsupportedMutatingMethod { .. }) => {
                    return Err(RequestBodyError::Returned(Value::error(format!(
                        "net/http: {builtin} body must implement io.Reader"
                    ))));
                }
                Err(error) => return Err(RequestBodyError::Fatal(error)),
            }
        }

        let session_id = vm.allocate_fetch_session_id();
        vm.set_http_request_upload_state(Some(HttpRequestUploadState {
            session_id,
            phase: HttpRequestUploadPhase::Started,
            error: None,
        }));
        return Ok(CapabilityRequest::FetchStart {
            request: FetchStartRequest {
                session_id,
                method: method.into(),
                url: url.into(),
                headers: headers.to_vec(),
                context_deadline_unix_millis,
            },
        });
    };

    match upload.phase {
        HttpRequestUploadPhase::Started | HttpRequestUploadPhase::Streaming => {
            match next_request_body_step(vm, program, builtin, body) {
                Ok(RequestBodyStep::Chunk { bytes, eof }) => {
                    vm.set_http_request_upload_state(Some(HttpRequestUploadState {
                        session_id: upload.session_id,
                        phase: if eof {
                            HttpRequestUploadPhase::Closing
                        } else {
                            HttpRequestUploadPhase::Streaming
                        },
                        error: None,
                    }));
                    Ok(CapabilityRequest::FetchBodyChunk {
                        request: FetchBodyChunkRequest {
                            session_id: upload.session_id,
                            chunk: bytes,
                        },
                    })
                }
                Ok(RequestBodyStep::Complete) => {
                    vm.set_http_request_upload_state(Some(HttpRequestUploadState {
                        session_id: upload.session_id,
                        phase: HttpRequestUploadPhase::AwaitingResponse,
                        error: None,
                    }));
                    Ok(CapabilityRequest::FetchBodyComplete {
                        request: FetchBodyCompleteRequest {
                            session_id: upload.session_id,
                        },
                    })
                }
                Err(RequestBodyError::Returned(error)) => {
                    vm.set_http_request_upload_state(Some(HttpRequestUploadState {
                        session_id: upload.session_id,
                        phase: HttpRequestUploadPhase::Aborting,
                        error: Some(error),
                    }));
                    Ok(CapabilityRequest::FetchBodyAbort {
                        request: FetchBodyAbortRequest {
                            session_id: upload.session_id,
                        },
                    })
                }
                Err(RequestBodyError::Fatal(error)) => Err(RequestBodyError::Fatal(error)),
            }
        }
        HttpRequestUploadPhase::Closing => {
            vm.set_http_request_upload_state(Some(HttpRequestUploadState {
                session_id: upload.session_id,
                phase: HttpRequestUploadPhase::AwaitingResponse,
                error: None,
            }));
            Ok(CapabilityRequest::FetchBodyComplete {
                request: FetchBodyCompleteRequest {
                    session_id: upload.session_id,
                },
            })
        }
        HttpRequestUploadPhase::AwaitingResponse => {
            unreachable!("fetch upload should not be polled while awaiting a response")
        }
        HttpRequestUploadPhase::Aborting => {
            vm.set_http_request_upload_state(None);
            Err(RequestBodyError::Returned(upload.error.expect(
                "aborting fetch uploads must preserve the read error",
            )))
        }
    }
}

fn next_request_body_step(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    body: &Value,
) -> Result<RequestBodyStep, RequestBodyError> {
    if matches!(&body.data, ValueData::Nil) {
        return Ok(RequestBodyStep::Complete);
    }

    let (results, buffer) = vm
        .invoke_method_results_mutating_arg(
            program,
            body.clone(),
            "Read",
            vec![Value::slice(vec![Value::int(0); REQUEST_BODY_BUFFER_SIZE])],
            0,
        )
        .map_err(|error| match error {
            VmError::UnknownMethod { .. } | VmError::UnsupportedMutatingMethod { .. } => {
                RequestBodyError::Returned(Value::error(format!(
                    "net/http: {builtin} body must implement io.Reader"
                )))
            }
            other => RequestBodyError::Fatal(other),
        })?;

    let (read_count, read_error) =
        normalize_request_body_read(vm, program, builtin, &buffer, &results)
            .map_err(RequestBodyError::Fatal)?;
    let bytes = if read_count == 0 {
        Vec::new()
    } else {
        read_buffer_prefix(vm, program, builtin, &buffer, read_count)
            .map_err(RequestBodyError::Fatal)?
    };

    match read_error {
        Some(error) if is_eof_error(&error) && bytes.is_empty() => Ok(RequestBodyStep::Complete),
        Some(error) if is_eof_error(&error) => Ok(RequestBodyStep::Chunk { bytes, eof: true }),
        Some(error) => Err(RequestBodyError::Returned(error)),
        None if bytes.is_empty() => Ok(RequestBodyStep::Complete),
        None => Ok(RequestBodyStep::Chunk { bytes, eof: false }),
    }
}

fn normalize_request_body_read(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    buffer: &Value,
    results: &[Value],
) -> Result<(usize, Option<Value>), VmError> {
    if results.len() != 2 {
        return Err(invalid_body_reader(vm, program, builtin));
    }

    let buffer_len = match &buffer.data {
        ValueData::Slice(slice) => slice.len(),
        _ => return Err(invalid_body_reader(vm, program, builtin)),
    };

    let ValueData::Int(read_count) = &results[0].data else {
        return Err(invalid_body_reader(vm, program, builtin));
    };
    if *read_count < 0 || *read_count as usize > buffer_len {
        return Err(invalid_body_reader(vm, program, builtin));
    }

    let read_error = match &results[1].data {
        ValueData::Nil => None,
        ValueData::Error(_) => Some(results[1].clone()),
        _ => return Err(invalid_body_reader(vm, program, builtin)),
    };

    Ok((*read_count as usize, read_error))
}

fn read_buffer_prefix(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    buffer: &Value,
    read_count: usize,
) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &buffer.data else {
        return Err(invalid_body_reader(vm, program, builtin));
    };

    slice
        .values_snapshot()
        .iter()
        .take(read_count)
        .map(|value| match &value.data {
            ValueData::Int(number) if (0..=255).contains(number) => Ok(*number as u8),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: vm
                    .current_function_name(program)
                    .unwrap_or_else(|_| "<unknown>".into()),
                builtin: builtin.into(),
                expected: "a body reader writing into a []byte buffer".into(),
            }),
        })
        .collect()
}

fn invalid_body_reader(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        builtin: builtin.into(),
        expected: "a body reader returning (int, error)".into(),
    }
}

fn is_eof_error(value: &Value) -> bool {
    matches!(
        &value.data,
        ValueData::Error(error) if error.message == "EOF"
    )
}
