#![cfg_attr(not(test), allow(dead_code))]

use super::{scheduler::GoroutineId, DeferredCallKind, Frame, Value, Vm};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GcRootLocation {
    Global {
        global: usize,
    },
    FrameRegister {
        goroutine: GoroutineId,
        frame_id: u64,
        register: usize,
    },
    DeferredCallSlot {
        goroutine: GoroutineId,
        frame_id: u64,
        deferred: usize,
        slot: &'static str,
    },
    CallbackResult {
        index: usize,
    },
    CallbackCapturedArg {
        index: usize,
    },
    PendingHttpRequestContext,
    HttpRequestBodyReader {
        body_id: u64,
    },
    HttpResponseBodyRequestContext {
        body_id: u64,
    },
    ContextDoneWatcherParent {
        channel_id: u64,
        context_id: u64,
        watcher: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GcRootRef<'a> {
    pub(crate) location: GcRootLocation,
    pub(crate) value: &'a Value,
}

impl Vm {
    pub(crate) fn visit_gc_roots<'a>(&'a self, mut visit: impl FnMut(GcRootRef<'a>)) {
        for (global, value) in self.globals.iter().enumerate() {
            visit(GcRootRef {
                location: GcRootLocation::Global { global },
                value,
            });
        }

        for goroutine in &self.goroutines {
            for frame in &goroutine.frames {
                visit_frame_roots(goroutine.id, frame, &mut visit);
            }
        }

        if let Some(results) = &self.callback_result {
            for (index, value) in results.iter().enumerate() {
                visit(GcRootRef {
                    location: GcRootLocation::CallbackResult { index },
                    value,
                });
            }
        }

        if let Some(values) = &self.callback_captured_args {
            for (index, value) in values.iter().enumerate() {
                visit(GcRootRef {
                    location: GcRootLocation::CallbackCapturedArg { index },
                    value,
                });
            }
        }

        if let Some(value) = &self.pending_http_request_context {
            visit(GcRootRef {
                location: GcRootLocation::PendingHttpRequestContext,
                value,
            });
        }

        let mut request_body_entries = self.http_request_bodies.iter().collect::<Vec<_>>();
        request_body_entries.sort_by_key(|(body_id, _)| *body_id);
        for (body_id, state) in request_body_entries {
            visit(GcRootRef {
                location: GcRootLocation::HttpRequestBodyReader { body_id: *body_id },
                value: &state.reader,
            });
        }

        let mut response_body_entries = self.http_response_bodies.iter().collect::<Vec<_>>();
        response_body_entries.sort_by_key(|(body_id, _)| *body_id);
        for (body_id, state) in response_body_entries {
            if let Some(request_context) = &state.request_context {
                visit(GcRootRef {
                    location: GcRootLocation::HttpResponseBodyRequestContext { body_id: *body_id },
                    value: request_context,
                });
            }
        }

        let mut watcher_entries = self.context_done_watchers.iter().collect::<Vec<_>>();
        watcher_entries.sort_by_key(|(channel_id, _)| *channel_id);
        for (channel_id, watchers) in watcher_entries {
            for (watcher_index, watcher) in watchers.iter().enumerate() {
                visit(GcRootRef {
                    location: GcRootLocation::ContextDoneWatcherParent {
                        channel_id: *channel_id,
                        context_id: watcher.context_id,
                        watcher: watcher_index,
                    },
                    value: &watcher.parent,
                });
            }
        }
    }
}

fn visit_frame_roots<'a>(
    goroutine: GoroutineId,
    frame: &'a Frame,
    visit: &mut impl FnMut(GcRootRef<'a>),
) {
    for (register, value) in frame.registers.iter().enumerate() {
        visit(GcRootRef {
            location: GcRootLocation::FrameRegister {
                goroutine,
                frame_id: frame.id,
                register,
            },
            value,
        });
    }

    for (deferred_index, deferred) in frame.deferred.iter().enumerate() {
        if let DeferredCallKind::Closure { function } = &deferred.kind {
            visit(GcRootRef {
                location: GcRootLocation::DeferredCallSlot {
                    goroutine,
                    frame_id: frame.id,
                    deferred: deferred_index,
                    slot: "closure_function",
                },
                value: function,
            });
        }
        for value in &deferred.args {
            visit(GcRootRef {
                location: GcRootLocation::DeferredCallSlot {
                    goroutine,
                    frame_id: frame.id,
                    deferred: deferred_index,
                    slot: "args",
                },
                value,
            });
        }
    }
}
