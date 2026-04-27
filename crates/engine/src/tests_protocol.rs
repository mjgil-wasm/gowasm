use gowasm_host_types::{EngineRequest, EngineResponse, ENGINE_PROTOCOL_VERSION};

use super::{handle_request, ENGINE_NAME};

#[test]
fn boot_reports_the_shared_protocol_version() {
    let response = handle_request(EngineRequest::Boot);
    match response {
        EngineResponse::Ready { info } => {
            assert_eq!(info.protocol_version, ENGINE_PROTOCOL_VERSION);
            assert_eq!(info.engine_name, ENGINE_NAME);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
