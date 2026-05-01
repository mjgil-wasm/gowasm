#![cfg_attr(not(test), allow(dead_code))]

use super::{
    channels::ChannelState, ContextState, DeferredCall, DeferredCallKind, Value, ValueData,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GcObjectKind {
    HeapCell,
    ClosureEnv,
    ArrayValue,
    StructValue,
    SliceValue,
    MapValue,
    ErrorValue,
    ChannelState,
    DeferredCall,
    ContextState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum GcSlotKind {
    Value,
    OptionalValue,
    ValueVec,
    ValuePairVec,
    PointerTarget,
    PointerTargetVec,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GcSlotDescriptor {
    pub(crate) name: &'static str,
    pub(crate) kind: GcSlotKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GcObjectDescriptor {
    pub(crate) kind: GcObjectKind,
    pub(crate) fixed_slots: &'static [GcSlotDescriptor],
    pub(crate) variable_slots: &'static [GcSlotDescriptor],
}

const NO_SLOTS: &[GcSlotDescriptor] = &[];

const HEAP_CELL_FIXED_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "payload",
    kind: GcSlotKind::Value,
}];

const CLOSURE_ENV_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "captures",
    kind: GcSlotKind::ValueVec,
}];

const ARRAY_VALUE_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "elements",
    kind: GcSlotKind::ValueVec,
}];

const STRUCT_VALUE_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "fields",
    kind: GcSlotKind::ValueVec,
}];

const SLICE_VALUE_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "elements",
    kind: GcSlotKind::ValueVec,
}];

const MAP_VALUE_FIXED_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "zero_value",
    kind: GcSlotKind::Value,
}];

const MAP_VALUE_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "entries",
    kind: GcSlotKind::ValuePairVec,
}];

const ERROR_VALUE_FIXED_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "wrapped",
    kind: GcSlotKind::OptionalValue,
}];

const CHANNEL_STATE_FIXED_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "zero_value",
    kind: GcSlotKind::Value,
}];

const CHANNEL_STATE_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[
    GcSlotDescriptor {
        name: "buffer",
        kind: GcSlotKind::ValueVec,
    },
    GcSlotDescriptor {
        name: "pending_send_values",
        kind: GcSlotKind::ValueVec,
    },
];

const DEFERRED_CALL_FIXED_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "closure_function",
    kind: GcSlotKind::OptionalValue,
}];

const DEFERRED_CALL_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "args",
    kind: GcSlotKind::ValueVec,
}];

const CONTEXT_STATE_FIXED_SLOTS: &[GcSlotDescriptor] = &[
    GcSlotDescriptor {
        name: "parent_value",
        kind: GcSlotKind::OptionalValue,
    },
    GcSlotDescriptor {
        name: "err",
        kind: GcSlotKind::OptionalValue,
    },
];

const CONTEXT_STATE_VARIABLE_SLOTS: &[GcSlotDescriptor] = &[GcSlotDescriptor {
    name: "key_values",
    kind: GcSlotKind::ValuePairVec,
}];

const HEAP_CELL_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::HeapCell,
    fixed_slots: HEAP_CELL_FIXED_SLOTS,
    variable_slots: NO_SLOTS,
};

const CLOSURE_ENV_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::ClosureEnv,
    fixed_slots: NO_SLOTS,
    variable_slots: CLOSURE_ENV_VARIABLE_SLOTS,
};

const ARRAY_VALUE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::ArrayValue,
    fixed_slots: NO_SLOTS,
    variable_slots: ARRAY_VALUE_VARIABLE_SLOTS,
};

const STRUCT_VALUE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::StructValue,
    fixed_slots: NO_SLOTS,
    variable_slots: STRUCT_VALUE_VARIABLE_SLOTS,
};

const SLICE_VALUE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::SliceValue,
    fixed_slots: NO_SLOTS,
    variable_slots: SLICE_VALUE_VARIABLE_SLOTS,
};

const MAP_VALUE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::MapValue,
    fixed_slots: MAP_VALUE_FIXED_SLOTS,
    variable_slots: MAP_VALUE_VARIABLE_SLOTS,
};

const ERROR_VALUE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::ErrorValue,
    fixed_slots: ERROR_VALUE_FIXED_SLOTS,
    variable_slots: NO_SLOTS,
};

const CHANNEL_STATE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::ChannelState,
    fixed_slots: CHANNEL_STATE_FIXED_SLOTS,
    variable_slots: CHANNEL_STATE_VARIABLE_SLOTS,
};

const DEFERRED_CALL_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::DeferredCall,
    fixed_slots: DEFERRED_CALL_FIXED_SLOTS,
    variable_slots: DEFERRED_CALL_VARIABLE_SLOTS,
};

const CONTEXT_STATE_DESCRIPTOR: GcObjectDescriptor = GcObjectDescriptor {
    kind: GcObjectKind::ContextState,
    fixed_slots: CONTEXT_STATE_FIXED_SLOTS,
    variable_slots: CONTEXT_STATE_VARIABLE_SLOTS,
};

pub(crate) fn heap_cell_descriptor() -> &'static GcObjectDescriptor {
    &HEAP_CELL_DESCRIPTOR
}

pub(crate) fn value_gc_descriptor(value: &Value) -> Option<&'static GcObjectDescriptor> {
    match &value.data {
        ValueData::Function(_) => Some(&CLOSURE_ENV_DESCRIPTOR),
        ValueData::Array(_) => Some(&ARRAY_VALUE_DESCRIPTOR),
        ValueData::Struct(_) => Some(&STRUCT_VALUE_DESCRIPTOR),
        ValueData::Slice(_) => Some(&SLICE_VALUE_DESCRIPTOR),
        ValueData::Map(_) => Some(&MAP_VALUE_DESCRIPTOR),
        ValueData::Error(_) => Some(&ERROR_VALUE_DESCRIPTOR),
        _ => None,
    }
}

pub(crate) fn channel_state_descriptor() -> &'static GcObjectDescriptor {
    &CHANNEL_STATE_DESCRIPTOR
}

pub(crate) fn deferred_call_descriptor() -> &'static GcObjectDescriptor {
    &DEFERRED_CALL_DESCRIPTOR
}

pub(crate) fn context_state_descriptor() -> &'static GcObjectDescriptor {
    &CONTEXT_STATE_DESCRIPTOR
}

pub(crate) fn visit_heap_cell_gc_slots<'a>(
    value: &'a Value,
    mut visit: impl for<'b> FnMut(&'static str, &'b Value),
) {
    visit("payload", value);
}

pub(crate) fn visit_value_gc_slots<'a>(
    value: &'a Value,
    mut visit: impl for<'b> FnMut(&'static str, &'b Value),
) -> Option<&'static GcObjectDescriptor> {
    match &value.data {
        ValueData::Function(function) => {
            for capture in &function.captures {
                visit("captures", capture);
            }
            Some(&CLOSURE_ENV_DESCRIPTOR)
        }
        ValueData::Array(array) => {
            for element in array.values_snapshot() {
                visit("elements", &element);
            }
            Some(&ARRAY_VALUE_DESCRIPTOR)
        }
        ValueData::Struct(fields) => {
            for (_, field_value) in fields {
                visit("fields", field_value);
            }
            Some(&STRUCT_VALUE_DESCRIPTOR)
        }
        ValueData::Slice(slice) => {
            for element in slice.values_snapshot() {
                visit("elements", &element);
            }
            Some(&SLICE_VALUE_DESCRIPTOR)
        }
        ValueData::Map(map) => {
            visit("zero_value", &map.zero_value);
            if let Some(entries) = &map.entries {
                let entries = entries.borrow();
                for (key, value) in entries.iter() {
                    visit("entries", key);
                    visit("entries", value);
                }
            }
            Some(&MAP_VALUE_DESCRIPTOR)
        }
        ValueData::Error(error) => {
            if let Some(wrapped) = error.wrapped.as_deref() {
                visit("wrapped", wrapped);
            }
            Some(&ERROR_VALUE_DESCRIPTOR)
        }
        _ => None,
    }
}

pub(crate) fn visit_channel_state_gc_slots<'a>(
    state: &'a ChannelState,
    mut visit: impl for<'b> FnMut(&'static str, &'b Value),
) {
    visit("zero_value", &state.zero_value);
    for value in &state.buffer {
        visit("buffer", value);
    }
    for pending in &state.pending_sends {
        visit("pending_send_values", &pending.value);
    }
}

pub(crate) fn visit_deferred_call_gc_slots<'a>(
    deferred: &'a DeferredCall,
    mut visit: impl for<'b> FnMut(&'static str, &'b Value),
) {
    if let DeferredCallKind::Closure { function } = &deferred.kind {
        visit("closure_function", function);
    }
    for arg in &deferred.args {
        visit("args", arg);
    }
}

pub(crate) fn visit_context_state_gc_slots<'a>(
    state: &'a ContextState,
    mut visit: impl for<'b> FnMut(&'static str, &'b Value),
) {
    if let Some(parent) = state.parent_value.as_ref() {
        visit("parent_value", parent);
    }
    if let Some(err) = state.err.as_ref() {
        visit("err", err);
    }
    for (key, value) in &state.values {
        visit("key_values", key);
        visit("key_values", value);
    }
}
