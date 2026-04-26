use gowasm_vm::{CapabilityRequest, FetchResponse, FetchStartRequest, Program, RunOutcome, Vm};

pub(crate) fn complete_streamed_fetch_with_buffered_response(
    vm: &mut Vm,
    program: &Program,
    expected_start: FetchStartRequest,
    expected_chunks: &[&[u8]],
    response: FetchResponse,
) {
    let session_id = match vm
        .start_program(program)
        .expect("program should pause for streamed fetch start")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::FetchStart { request }) => {
            assert_eq!(request, expected_start);
            request.session_id
        }
        other => panic!("unexpected run outcome: {other:?}"),
    };

    vm.acknowledge_fetch_start();

    for expected_chunk in expected_chunks {
        match vm
            .resume_program(program)
            .expect("program should request the next upload chunk")
        {
            RunOutcome::CapabilityRequest(CapabilityRequest::FetchBodyChunk { request }) => {
                assert_eq!(request.session_id, session_id);
                assert_eq!(request.chunk, *expected_chunk);
            }
            other => panic!("unexpected chunk outcome: {other:?}"),
        }
        vm.acknowledge_fetch_body_chunk();
    }

    match vm
        .resume_program(program)
        .expect("program should complete the streamed upload")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::FetchBodyComplete { request }) => {
            assert_eq!(request.session_id, session_id);
        }
        other => panic!("unexpected completion outcome: {other:?}"),
    }

    vm.set_fetch_response(response);

    match vm
        .resume_program(program)
        .expect("program should complete after fetch response")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected resumed outcome: {other:?}"),
    }
}
