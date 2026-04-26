use std::any::Any;
use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, UnwindSafe};

use gowasm_host_types::{EngineRequest, EngineResponse, ErrorCategory};

const DEFAULT_BROWSER_INSTRUCTION_BUDGET: u64 = 100_000;

thread_local! {
    static ENGINE: RefCell<gowasm_engine::Engine> = RefCell::new(
        gowasm_engine::Engine::with_instruction_budget(DEFAULT_BROWSER_INSTRUCTION_BUDGET)
    );
    static RESPONSE_BUFFER: RefCell<OwnedBuffer> = const { RefCell::new(OwnedBuffer::empty()) };
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WasmAbiStatus {
    Ok = 0,
    InvalidRequestBuffer = 1,
    InvalidUtf8 = 2,
    InvalidProtocol = 3,
    Panic = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OwnedBuffer {
    ptr: *mut u8,
    len: usize,
}

impl OwnedBuffer {
    const fn empty() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
        }
    }

    fn matches(&self, ptr: *mut u8, len: usize) -> bool {
        self.ptr == ptr && self.len == len
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AbiResponse {
    status: WasmAbiStatus,
    body: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_request_buffer(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

/// # Safety
/// `ptr` and `len` must represent a valid buffer previously returned by `alloc_request_buffer`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_request_buffer(ptr: *mut u8, len: usize) {
    free_foreign_buffer(ptr, len);
}

/// # Safety
/// `ptr` and `len` must represent a valid request buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn handle_request(ptr: *mut u8, len: usize) -> u32 {
    let response = match request_bytes(ptr, len) {
        Ok(bytes) => process_request(bytes, |request| {
            ENGINE.with(|engine| engine.borrow_mut().handle_request(request))
        }),
        Err(message) => AbiResponse {
            status: WasmAbiStatus::InvalidRequestBuffer,
            body: serialize_fatal(&message),
        },
    };

    store_response_bytes(response.body);
    response.status as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn response_ptr() -> *const u8 {
    RESPONSE_BUFFER.with(|buffer| buffer.borrow().ptr.cast_const())
}

#[unsafe(no_mangle)]
pub extern "C" fn response_len() -> usize {
    RESPONSE_BUFFER.with(|buffer| buffer.borrow().len)
}

/// # Safety
/// `ptr` and `len` must represent a valid buffer previously returned by `response_ptr`/`response_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_response_buffer(ptr: *mut u8, len: usize) {
    RESPONSE_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        if !buffer.matches(ptr, len) {
            return;
        }
        *buffer = OwnedBuffer::empty();
        free_foreign_buffer(ptr, len);
    });
}

fn request_bytes<'a>(ptr: *mut u8, len: usize) -> Result<&'a [u8], String> {
    if ptr.is_null() {
        return Err("request buffer pointer was null".into());
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr.cast_const(), len) })
}

fn process_request(
    bytes: &[u8],
    handler: impl FnOnce(EngineRequest) -> EngineResponse + UnwindSafe,
) -> AbiResponse {
    let input = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return AbiResponse {
                status: WasmAbiStatus::InvalidUtf8,
                body: serialize_fatal(&format!("request bytes were not valid utf-8: {error}")),
            };
        }
    };

    let request = match serde_json::from_str::<EngineRequest>(input) {
        Ok(request) => request,
        Err(error) => {
            return AbiResponse {
                status: WasmAbiStatus::InvalidProtocol,
                body: serialize_fatal(&format!("invalid engine request json: {error}")),
            };
        }
    };

    match std::panic::catch_unwind(AssertUnwindSafe(|| handler(request))) {
        Ok(response) => AbiResponse {
            status: WasmAbiStatus::Ok,
            body: serialize_response(&response),
        },
        Err(payload) => AbiResponse {
            status: WasmAbiStatus::Panic,
            body: serialize_fatal(&format!(
                "engine request panicked before producing a response: {}",
                panic_message(payload.as_ref())
            )),
        },
    }
}

fn serialize_response(response: &EngineResponse) -> Vec<u8> {
    serde_json::to_vec(response).expect("engine responses should always serialize")
}

fn serialize_fatal(message: &str) -> Vec<u8> {
    serialize_response(&EngineResponse::Fatal {
        message: message.into(),
        category: ErrorCategory::ProtocolError,
    })
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "non-string panic payload".into()
}

fn store_response_bytes(bytes: Vec<u8>) {
    RESPONSE_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        if !buffer.ptr.is_null() {
            free_foreign_buffer(buffer.ptr, buffer.len);
        }
        *buffer = leak_bytes(bytes);
    });
}

fn leak_bytes(bytes: Vec<u8>) -> OwnedBuffer {
    let mut bytes = bytes.into_boxed_slice();
    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();
    std::mem::forget(bytes);
    OwnedBuffer { ptr, len }
}

fn free_foreign_buffer(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        alloc_request_buffer, free_request_buffer, free_response_buffer, handle_request,
        process_request, request_bytes, response_len, response_ptr, WasmAbiStatus,
    };
    use gowasm_host_types::{EngineResponse, ErrorCategory};

    fn send_request_bytes(bytes: &[u8]) -> (u32, EngineResponse) {
        let ptr = alloc_request_buffer(bytes.len());
        unsafe {
            std::slice::from_raw_parts_mut(ptr, bytes.len()).copy_from_slice(bytes);
        }

        let status = unsafe { handle_request(ptr, bytes.len()) };
        unsafe { free_request_buffer(ptr, bytes.len()) };

        let response_ptr = response_ptr() as *mut u8;
        let response_len = response_len();
        let response_bytes =
            unsafe { std::slice::from_raw_parts(response_ptr.cast_const(), response_len) }.to_vec();
        unsafe { free_response_buffer(response_ptr, response_len) };

        let response =
            serde_json::from_slice(&response_bytes).expect("response should decode as json");
        (status, response)
    }

    #[test]
    fn wasm_bridge_round_trips_request_and_response_allocations() {
        let (status, response) = send_request_bytes(br#"{"kind":"boot"}"#);
        assert_eq!(status, WasmAbiStatus::Ok as u32);
        assert!(matches!(response, EngineResponse::Ready { .. }));
    }

    #[test]
    fn wasm_bridge_reports_invalid_utf8_requests() {
        let (status, response) = send_request_bytes(&[0xff, 0xfe, 0xfd]);
        assert_eq!(status, WasmAbiStatus::InvalidUtf8 as u32);
        assert!(matches!(
            response,
            EngineResponse::Fatal { message, category }
                if message.contains("valid utf-8")
                    && category == ErrorCategory::ProtocolError
        ));
    }

    #[test]
    fn wasm_bridge_reports_invalid_protocol_payloads() {
        let (status, response) = send_request_bytes(br#"{"kind":"not_a_real_request"}"#);
        assert_eq!(status, WasmAbiStatus::InvalidProtocol as u32);
        assert!(matches!(
            response,
            EngineResponse::Fatal { message, category }
                if message.contains("invalid engine request json")
                    && category == ErrorCategory::ProtocolError
        ));
    }

    #[test]
    fn wasm_bridge_serializes_panics_as_fatal_responses() {
        let response = process_request(br#"{"kind":"boot"}"#, |_| panic!("bridge boom"));
        assert_eq!(response.status, WasmAbiStatus::Panic);
        let response: EngineResponse =
            serde_json::from_slice(&response.body).expect("response should decode");
        assert!(matches!(
            response,
            EngineResponse::Fatal { message, category }
                if message.contains("bridge boom")
                    && category == ErrorCategory::ProtocolError
        ));
    }

    #[test]
    fn wasm_bridge_supports_repeated_calls_with_explicit_buffer_frees() {
        let (first_status, first_response) = send_request_bytes(br#"{"kind":"boot"}"#);
        let (second_status, second_response) = send_request_bytes(br#"{"kind":"boot"}"#);
        assert_eq!(first_status, WasmAbiStatus::Ok as u32);
        assert_eq!(second_status, WasmAbiStatus::Ok as u32);
        assert!(matches!(first_response, EngineResponse::Ready { .. }));
        assert!(matches!(second_response, EngineResponse::Ready { .. }));
    }

    #[test]
    fn wasm_bridge_rejects_null_request_pointers() {
        let response = match request_bytes(std::ptr::null_mut(), 0) {
            Ok(_) => panic!("null request pointer should fail"),
            Err(message) => message,
        };
        assert!(response.contains("pointer was null"));
    }
}
