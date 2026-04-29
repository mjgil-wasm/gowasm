#![cfg_attr(not(test), allow(dead_code))]

use std::collections::HashSet;

use super::{gc_metadata, ContextState, PointerTarget, Value, ValueData, Vm, TYPE_CONTEXT};

const CONTEXT_ID_FIELD: &str = "__context_id";

#[derive(Debug)]
struct GcMarks {
    heap_cells: Vec<bool>,
    channels: HashSet<u64>,
    contexts: HashSet<u64>,
}

impl GcMarks {
    fn new(heap_cells: usize) -> Self {
        Self {
            heap_cells: vec![false; heap_cells],
            channels: HashSet::new(),
            contexts: HashSet::new(),
        }
    }
}

impl Vm {
    pub(crate) fn collect_garbage(&mut self) -> usize {
        let mut marks = GcMarks::new(self.heap_cells.len());
        self.mark_gc_roots(&mut marks);
        let freed = self.sweep_heap_cells(&marks);
        self.allocations_since_gc = 0;
        self.last_gc_freed_cells = freed;
        self.total_gc_collections = self.total_gc_collections.saturating_add(1);
        self.total_gc_freed_cells = self.total_gc_freed_cells.saturating_add(freed);
        freed
    }

    pub(crate) fn record_heap_allocation(&mut self) {
        self.allocations_since_gc = self.allocations_since_gc.saturating_add(1);
    }

    pub(crate) fn maybe_collect_garbage(&mut self) -> usize {
        let Some(threshold) = self.gc_allocation_threshold else {
            return 0;
        };
        if self.allocations_since_gc < threshold {
            return 0;
        }
        self.collect_garbage()
    }

    fn mark_gc_roots(&self, marks: &mut GcMarks) {
        self.visit_gc_roots(|root| self.mark_value(root.value, marks));

        for timer in &self.time_channel_timers {
            self.mark_channel(timer.channel_id, marks);
        }
        for timer in &self.context_deadline_timers {
            self.mark_context(timer.context_id, marks);
        }
        for watchers in self.context_done_watchers.values() {
            for watcher in watchers {
                self.mark_context(watcher.context_id, marks);
            }
        }
    }

    fn mark_value(&self, value: &Value, marks: &mut GcMarks) {
        if let Some(context_id) = context_id(value) {
            self.mark_context(context_id, marks);
        }

        match &value.data {
            ValueData::Pointer(pointer) => self.mark_pointer_target(&pointer.target, marks),
            ValueData::Channel(channel) => {
                if let Some(channel_id) = channel.id {
                    self.mark_channel(channel_id, marks);
                }
            }
            _ => {}
        }

        let _ = gc_metadata::visit_value_gc_slots(value, |_, child| self.mark_value(child, marks));
    }

    fn mark_pointer_target(&self, target: &PointerTarget, marks: &mut GcMarks) {
        match target {
            PointerTarget::Nil => {}
            PointerTarget::HeapCell { cell } => self.mark_heap_cell(*cell, marks),
            PointerTarget::Global { global } => {
                if let Some(value) = self.globals.get(*global) {
                    self.mark_value(value, marks);
                }
            }
            PointerTarget::Local { frame_id, register } => {
                if let Some(value) = self.frame_register_value(*frame_id, *register) {
                    self.mark_value(value, marks);
                }
            }
            PointerTarget::ProjectedField { base, .. } => self.mark_pointer_target(base, marks),
            PointerTarget::ProjectedIndex { base, index } => {
                self.mark_pointer_target(base, marks);
                self.mark_value(index, marks);
            }
            PointerTarget::LocalField {
                frame_id, register, ..
            } => {
                if let Some(value) = self.frame_register_value(*frame_id, *register) {
                    self.mark_value(value, marks);
                }
            }
            PointerTarget::GlobalField { global, .. } => {
                if let Some(value) = self.globals.get(*global) {
                    self.mark_value(value, marks);
                }
            }
            PointerTarget::LocalIndex {
                frame_id,
                register,
                index,
            } => {
                if let Some(value) = self.frame_register_value(*frame_id, *register) {
                    self.mark_value(value, marks);
                }
                self.mark_value(index, marks);
            }
            PointerTarget::GlobalIndex { global, index } => {
                if let Some(value) = self.globals.get(*global) {
                    self.mark_value(value, marks);
                }
                self.mark_value(index, marks);
            }
        }
    }

    fn mark_heap_cell(&self, cell: usize, marks: &mut GcMarks) {
        let Some(marked) = marks.heap_cells.get_mut(cell) else {
            return;
        };
        if *marked {
            return;
        }
        let Some(Some(value)) = self.heap_cells.get(cell) else {
            return;
        };
        *marked = true;
        gc_metadata::visit_heap_cell_gc_slots(value, |_, child| self.mark_value(child, marks));
    }

    fn mark_channel(&self, channel_id: u64, marks: &mut GcMarks) {
        if !marks.channels.insert(channel_id) {
            return;
        }
        let Some(state) = self.channels.get(channel_id as usize) else {
            return;
        };
        gc_metadata::visit_channel_state_gc_slots(state, |_, value| self.mark_value(value, marks));
    }

    fn mark_context(&self, context_id: u64, marks: &mut GcMarks) {
        if !marks.contexts.insert(context_id) {
            return;
        }
        let Some(state) = self.context_values.get(&context_id) else {
            return;
        };
        self.mark_context_state(state, marks);
    }

    fn mark_context_state(&self, state: &ContextState, marks: &mut GcMarks) {
        if let Some(done_channel_id) = state.done_channel_id {
            self.mark_channel(done_channel_id, marks);
        }
        for child in &state.children {
            self.mark_context(*child, marks);
        }
        gc_metadata::visit_context_state_gc_slots(state, |_, value| self.mark_value(value, marks));
    }

    fn frame_register_value(&self, frame_id: u64, register: usize) -> Option<&Value> {
        self.goroutines
            .iter()
            .flat_map(|goroutine| goroutine.frames.iter())
            .find(|frame| frame.id == frame_id)
            .and_then(|frame| frame.registers.get(register))
    }

    fn sweep_heap_cells(&mut self, marks: &GcMarks) -> usize {
        let mut freed = 0;
        for (cell, slot) in self.heap_cells.iter_mut().enumerate() {
            let marked = marks.heap_cells.get(cell).copied().unwrap_or(false);
            if !marked && slot.is_some() {
                *slot = None;
                self.free_heap_cells.push(cell);
                freed += 1;
            }
        }
        freed
    }
}

fn context_id(value: &Value) -> Option<u64> {
    if value.typ != TYPE_CONTEXT {
        return None;
    }
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };
    fields.iter().find_map(|(name, value)| {
        if name != CONTEXT_ID_FIELD {
            return None;
        }
        match value.data {
            ValueData::Int(id) if id > 0 => Some(id as u64),
            _ => None,
        }
    })
}
