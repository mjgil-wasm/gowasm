use std::collections::HashMap;

use super::{
    gc_roots::GcRootLocation,
    scheduler::{Goroutine, GoroutineId, GoroutineStatus},
    ContextDoneWatcher, DeferredCall, DeferredCallKind, Frame, HttpRequestBodyState,
    HttpResponseBodyState, ReturnTarget, UnwindState, Value, Vm,
};

fn frame(id: u64, registers: Vec<Value>, deferred: Vec<DeferredCall>) -> Frame {
    Frame {
        id,
        function: 0,
        pc: 0,
        registers,
        deferred,
        unwind: None,
        return_target: ReturnTarget::None,
    }
}

fn goroutine(id: u64, status: GoroutineStatus, frames: Vec<Frame>) -> Goroutine {
    Goroutine {
        id: GoroutineId(id),
        status,
        frames,
        pending_error: None,
        active_select: None,
    }
}

#[test]
fn visits_all_live_vm_root_sources_in_stable_order() {
    let vm = Vm {
        globals: vec![Value::int(1), Value::string("global")],
        goroutines: vec![
            goroutine(
                0,
                GoroutineStatus::Runnable,
                vec![frame(
                    10,
                    vec![Value::int(2), Value::function(30, vec![Value::int(31)])],
                    vec![
                        DeferredCall {
                            kind: DeferredCallKind::Closure {
                                function: Value::function(40, vec![Value::int(41)]),
                            },
                            args: vec![Value::int(42)],
                        },
                        DeferredCall {
                            kind: DeferredCallKind::Function { function: 50 },
                            args: vec![Value::string("deferred-arg")],
                        },
                    ],
                )],
            ),
            goroutine(
                3,
                GoroutineStatus::Blocked,
                vec![
                    frame(20, vec![Value::string("blocked")], Vec::new()),
                    Frame {
                        id: 21,
                        function: 0,
                        pc: 0,
                        registers: Vec::new(),
                        deferred: Vec::new(),
                        unwind: Some(UnwindState::Panic(Value::string("ignored"))),
                        return_target: ReturnTarget::Deferred,
                    },
                ],
            ),
            goroutine(9, GoroutineStatus::Done, Vec::new()),
        ],
        callback_result: Some(vec![Value::string("callback-result")]),
        callback_captured_args: Some(vec![Value::bool(true)]),
        pending_http_request_context: Some(Value::string("pending-request-context")),
        http_request_bodies: HashMap::from([(
            4,
            HttpRequestBodyState {
                reader: Value::string("request-body-reader"),
            },
        )]),
        http_response_bodies: HashMap::from([(
            6,
            HttpResponseBodyState {
                buffered: Vec::new(),
                read_offset: 0,
                closed: false,
                session_id: Some(9),
                eof: false,
                terminal_error: None,
                request_context: Some(Value::string("response-body-context")),
            },
        )]),
        context_done_watchers: HashMap::from([
            (
                2,
                vec![ContextDoneWatcher {
                    context_id: 77,
                    parent: Value::string("watcher-low"),
                }],
            ),
            (
                7,
                vec![
                    ContextDoneWatcher {
                        context_id: 88,
                        parent: Value::string("watcher-high-a"),
                    },
                    ContextDoneWatcher {
                        context_id: 89,
                        parent: Value::string("watcher-high-b"),
                    },
                ],
            ),
        ]),
        ..Default::default()
    };

    let mut roots = Vec::new();
    vm.visit_gc_roots(|root| roots.push((root.location, root.value.clone())));

    assert_eq!(
        roots,
        vec![
            (GcRootLocation::Global { global: 0 }, Value::int(1)),
            (
                GcRootLocation::Global { global: 1 },
                Value::string("global"),
            ),
            (
                GcRootLocation::FrameRegister {
                    goroutine: GoroutineId(0),
                    frame_id: 10,
                    register: 0,
                },
                Value::int(2),
            ),
            (
                GcRootLocation::FrameRegister {
                    goroutine: GoroutineId(0),
                    frame_id: 10,
                    register: 1,
                },
                Value::function(30, vec![Value::int(31)]),
            ),
            (
                GcRootLocation::DeferredCallSlot {
                    goroutine: GoroutineId(0),
                    frame_id: 10,
                    deferred: 0,
                    slot: "closure_function",
                },
                Value::function(40, vec![Value::int(41)]),
            ),
            (
                GcRootLocation::DeferredCallSlot {
                    goroutine: GoroutineId(0),
                    frame_id: 10,
                    deferred: 0,
                    slot: "args",
                },
                Value::int(42),
            ),
            (
                GcRootLocation::DeferredCallSlot {
                    goroutine: GoroutineId(0),
                    frame_id: 10,
                    deferred: 1,
                    slot: "args",
                },
                Value::string("deferred-arg"),
            ),
            (
                GcRootLocation::FrameRegister {
                    goroutine: GoroutineId(3),
                    frame_id: 20,
                    register: 0,
                },
                Value::string("blocked"),
            ),
            (
                GcRootLocation::CallbackResult { index: 0 },
                Value::string("callback-result"),
            ),
            (
                GcRootLocation::CallbackCapturedArg { index: 0 },
                Value::bool(true),
            ),
            (
                GcRootLocation::PendingHttpRequestContext,
                Value::string("pending-request-context"),
            ),
            (
                GcRootLocation::HttpRequestBodyReader { body_id: 4 },
                Value::string("request-body-reader"),
            ),
            (
                GcRootLocation::HttpResponseBodyRequestContext { body_id: 6 },
                Value::string("response-body-context"),
            ),
            (
                GcRootLocation::ContextDoneWatcherParent {
                    channel_id: 2,
                    context_id: 77,
                    watcher: 0,
                },
                Value::string("watcher-low"),
            ),
            (
                GcRootLocation::ContextDoneWatcherParent {
                    channel_id: 7,
                    context_id: 88,
                    watcher: 0,
                },
                Value::string("watcher-high-a"),
            ),
            (
                GcRootLocation::ContextDoneWatcherParent {
                    channel_id: 7,
                    context_id: 89,
                    watcher: 1,
                },
                Value::string("watcher-high-b"),
            ),
        ]
    );
}
