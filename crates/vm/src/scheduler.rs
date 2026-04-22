use std::collections::{HashMap, HashSet};

use super::{function_name, Frame, Program, ReturnTarget, Value, Vm, VmError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoroutineId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoroutineStatus {
    Runnable,
    Blocked,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerState {
    Runnable,
    Blocked,
    Done,
    PausedHostWait,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Goroutine {
    pub(super) id: GoroutineId,
    pub(super) status: GoroutineStatus,
    pub(super) frames: Vec<Frame>,
    pub(super) pending_error: Option<VmError>,
    pub(super) active_select: Option<u64>,
}

impl Vm {
    pub(super) fn reset_scheduler(&mut self) {
        self.goroutines.clear();
        self.current_goroutine = 0;
        self.next_goroutine_id = 0;
        self.next_frame_id = 0;
        self.next_channel_id = 0;
        self.next_select_id = 0;
        self.next_select_start = 0;
        self.paused_host_wait = false;
        self.cancelled = false;
    }

    pub(super) fn spawn_goroutine(
        &mut self,
        program: &Program,
        function_id: usize,
        args: Vec<Value>,
    ) -> Result<GoroutineId, VmError> {
        let function = program
            .functions
            .get(function_id)
            .ok_or(VmError::UnknownFunction {
                function: function_id,
            })?;
        if args.len() != function.param_count {
            return Err(VmError::WrongArgumentCount {
                function: function.name.clone(),
                expected: function.param_count,
                actual: args.len(),
            });
        }

        let mut registers = vec![Value::nil(); function.register_count];
        for (index, value) in args.into_iter().enumerate() {
            registers[index] = value;
        }

        let id = GoroutineId(self.next_goroutine_id);
        let frame_id = self.alloc_frame_id();
        self.next_goroutine_id += 1;
        self.goroutines.push(Goroutine {
            id,
            status: GoroutineStatus::Runnable,
            frames: vec![Frame {
                id: frame_id,
                function: function_id,
                pc: 0,
                registers,
                deferred: Vec::new(),
                unwind: None,
                return_target: ReturnTarget::None,
            }],
            pending_error: None,
            active_select: None,
        });
        if self.goroutines.len() == 1 {
            self.current_goroutine = 0;
        }
        Ok(id)
    }

    pub(super) fn push_frame_on_current_goroutine(
        &mut self,
        program: &Program,
        function_id: usize,
        args: Vec<Value>,
        return_target: ReturnTarget,
    ) -> Result<(), VmError> {
        let frame_id = self.alloc_frame_id();
        let frame = build_frame(program, function_id, args, return_target, frame_id)?;
        self.current_goroutine_mut().frames.push(frame);
        Ok(())
    }

    pub(super) fn has_live_goroutines(&self) -> bool {
        self.goroutines
            .iter()
            .any(|goroutine| goroutine.status != GoroutineStatus::Done)
    }

    pub(super) fn has_blocked_goroutines(&self) -> bool {
        self.goroutines
            .iter()
            .any(|goroutine| goroutine.status == GoroutineStatus::Blocked)
    }

    #[cfg(test)]
    pub(crate) fn scheduler_state(&self) -> SchedulerState {
        if self.cancelled {
            return SchedulerState::Cancelled;
        }
        if self.paused_host_wait {
            return SchedulerState::PausedHostWait;
        }
        if self
            .goroutines
            .iter()
            .any(|goroutine| goroutine.status == GoroutineStatus::Runnable)
        {
            return SchedulerState::Runnable;
        }
        if self.has_live_goroutines() {
            SchedulerState::Blocked
        } else {
            SchedulerState::Done
        }
    }

    #[cfg(test)]
    pub(crate) fn cancel_run(&mut self) {
        self.paused_host_wait = false;
        self.cancelled = true;
        self.sleeping_goroutines.clear();
        self.time_channel_timers.clear();
        self.context_deadline_timers.clear();
    }

    pub(super) fn advance_to_next_runnable(&mut self) -> bool {
        if self.goroutines.is_empty() {
            return false;
        }

        let total = self.goroutines.len();
        for offset in 0..total {
            let index = (self.current_goroutine + offset + 1) % total;
            if self.goroutines[index].status == GoroutineStatus::Runnable {
                self.current_goroutine = index;
                let goroutine_id = self.goroutines[index].id.0;
                self.record_scheduler_pick(goroutine_id);
                return true;
            }
        }

        false
    }

    pub(super) fn finish_current_goroutine_if_idle(&mut self) {
        if self.current_goroutine().frames.is_empty() {
            self.current_goroutine_mut().status = GoroutineStatus::Done;
        }
    }

    pub(super) fn current_goroutine(&self) -> &Goroutine {
        self.goroutines
            .get(self.current_goroutine)
            .expect("current goroutine should exist")
    }

    pub(super) fn current_goroutine_id(&self) -> GoroutineId {
        self.current_goroutine().id
    }

    pub(super) fn current_goroutine_mut(&mut self) -> &mut Goroutine {
        self.goroutines
            .get_mut(self.current_goroutine)
            .expect("current goroutine should exist")
    }

    pub(super) fn block_current_goroutine(&mut self) {
        self.current_goroutine_mut().status = GoroutineStatus::Blocked;
    }

    pub(super) fn wake_goroutine(&mut self, id: GoroutineId) {
        if self.cancelled {
            return;
        }
        self.sleeping_goroutines.remove(&id);
        if let Some(goroutine) = self
            .goroutines
            .iter_mut()
            .find(|candidate| candidate.id == id)
        {
            if goroutine.status != GoroutineStatus::Done {
                goroutine.status = GoroutineStatus::Runnable;
            }
        }
    }

    pub(super) fn alloc_select_id(&mut self) -> u64 {
        let select_id = self.next_select_id;
        self.next_select_id += 1;
        select_id
    }

    pub(super) fn next_select_start(&mut self, case_count: usize) -> usize {
        if case_count == 0 {
            return 0;
        }
        let start = self.next_select_start % case_count;
        self.next_select_start = self.next_select_start.wrapping_add(1);
        self.record_select_start(case_count, start);
        start
    }

    pub(super) fn set_current_goroutine_select(&mut self, select_id: Option<u64>) {
        self.current_goroutine_mut().active_select = select_id;
    }

    pub(super) fn goroutine_has_active_select(&self, id: GoroutineId, select_id: u64) -> bool {
        self.goroutines
            .iter()
            .find(|candidate| candidate.id == id)
            .and_then(|goroutine| goroutine.active_select)
            == Some(select_id)
    }

    pub(super) fn clear_goroutine_select(&mut self, id: GoroutineId, select_id: u64) {
        if let Some(goroutine) = self
            .goroutines
            .iter_mut()
            .find(|candidate| candidate.id == id)
        {
            if goroutine.active_select == Some(select_id) {
                goroutine.active_select = None;
            }
        }
    }

    pub(super) fn set_pending_error_on_goroutine(
        &mut self,
        id: GoroutineId,
        error: VmError,
    ) -> Result<(), VmError> {
        let Some(goroutine) = self
            .goroutines
            .iter_mut()
            .find(|candidate| candidate.id == id)
        else {
            return Err(VmError::UnknownGoroutine { goroutine: id.0 });
        };
        goroutine.pending_error = Some(error);
        Ok(())
    }

    pub(super) fn take_pending_error_for_current_goroutine(&mut self) -> Option<VmError> {
        self.current_goroutine_mut().pending_error.take()
    }

    pub(super) fn current_frame(&self) -> &Frame {
        self.current_goroutine()
            .frames
            .last()
            .expect("frame should exist")
    }

    pub(super) fn current_frame_mut(&mut self) -> &mut Frame {
        self.current_goroutine_mut()
            .frames
            .last_mut()
            .expect("frame should exist")
    }

    pub(super) fn current_function_name(&self, program: &Program) -> Result<String, VmError> {
        let Some(goroutine) = self.goroutines.get(self.current_goroutine) else {
            return Err(VmError::UnknownGoroutine {
                goroutine: self.current_goroutine as u64,
            });
        };
        let Some(frame) = goroutine.frames.last() else {
            return Err(VmError::UnknownGoroutine {
                goroutine: goroutine.id.0,
            });
        };
        function_name(program, frame.function)
    }

    pub(super) fn goroutine_function_name(
        &self,
        program: &Program,
        goroutine: GoroutineId,
    ) -> Result<String, VmError> {
        let Some(goroutine) = self
            .goroutines
            .iter()
            .find(|candidate| candidate.id == goroutine)
        else {
            return Err(VmError::UnknownGoroutine {
                goroutine: goroutine.0,
            });
        };
        let frame = goroutine.frames.last().ok_or(VmError::UnknownGoroutine {
            goroutine: goroutine.id.0,
        })?;
        function_name(program, frame.function)
    }

    pub(super) fn set_register_on_goroutine(
        &mut self,
        program: &Program,
        goroutine: GoroutineId,
        register: usize,
        value: Value,
    ) -> Result<(), VmError> {
        self.debug_assert_value_invariants(program, &value);
        let Some(goroutine) = self
            .goroutines
            .iter_mut()
            .find(|candidate| candidate.id == goroutine)
        else {
            return Err(VmError::UnknownGoroutine {
                goroutine: goroutine.0,
            });
        };
        let frame = goroutine
            .frames
            .last_mut()
            .ok_or(VmError::UnknownGoroutine {
                goroutine: goroutine.id.0,
            })?;
        let function_name = function_name(program, frame.function)?;
        let slot = frame
            .registers
            .get_mut(register)
            .ok_or(VmError::InvalidRegister {
                function: function_name,
                register,
            })?;
        *slot = value;
        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn assert_channel_wait_queue_invariants(&self) {
        if let Err(message) = self.channel_wait_queue_invariant_error() {
            panic!("{message}");
        }
    }

    pub(crate) fn debug_assert_channel_wait_queue_invariants(&self) {
        #[cfg(debug_assertions)]
        self.assert_channel_wait_queue_invariants();
    }

    pub(crate) fn channel_wait_queue_invariant_error(&self) -> Result<(), String> {
        let goroutines_by_id: HashMap<_, _> = self
            .goroutines
            .iter()
            .map(|goroutine| (goroutine.id, goroutine))
            .collect();
        let mut active_select_waiters = HashSet::new();
        let mut active_select_presence = HashSet::new();
        let mut non_select_waiters = HashSet::new();

        for (channel_id, state) in self.channels.iter().enumerate() {
            for sender in &state.pending_sends {
                self.validate_pending_sender(
                    &goroutines_by_id,
                    &mut active_select_waiters,
                    &mut active_select_presence,
                    &mut non_select_waiters,
                    channel_id,
                    sender,
                )?;
            }
            for receiver in &state.pending_receivers {
                self.validate_pending_receiver(
                    &goroutines_by_id,
                    &mut active_select_waiters,
                    &mut active_select_presence,
                    &mut non_select_waiters,
                    channel_id,
                    receiver,
                )?;
            }
        }

        for goroutine in &self.goroutines {
            if goroutine.status != GoroutineStatus::Blocked {
                continue;
            }
            let Some(select_id) = goroutine.active_select else {
                continue;
            };
            if !active_select_presence.contains(&(goroutine.id.0, select_id)) {
                return Err(format!(
                    "blocked select goroutine {} with select {} has no queued waiters",
                    goroutine.id.0, select_id
                ));
            }
        }

        Ok(())
    }

    fn validate_pending_sender(
        &self,
        goroutines_by_id: &HashMap<GoroutineId, &Goroutine>,
        active_select_waiters: &mut HashSet<(u64, u64, usize)>,
        active_select_presence: &mut HashSet<(u64, u64)>,
        non_select_waiters: &mut HashSet<u64>,
        channel_id: usize,
        sender: &super::channels::PendingSend,
    ) -> Result<(), String> {
        let goroutine = goroutines_by_id.get(&sender.goroutine).ok_or_else(|| {
            format!(
                "pending sender on channel {} references unknown goroutine {}",
                channel_id, sender.goroutine.0
            )
        })?;
        match sender.select.as_ref() {
            Some(select) => {
                if goroutine.active_select == Some(select.select_id) {
                    if goroutine.status != GoroutineStatus::Blocked {
                        return Err(format!(
                            "active select sender on channel {} references non-blocked goroutine {}",
                            channel_id, goroutine.id.0
                        ));
                    }
                    let key = (goroutine.id.0, select.select_id, select.case_index);
                    if !active_select_waiters.insert(key) {
                        return Err(format!(
                            "duplicate active select waiter for goroutine {} select {} case {}",
                            goroutine.id.0, select.select_id, select.case_index
                        ));
                    }
                    active_select_presence.insert((goroutine.id.0, select.select_id));
                }
            }
            None => {
                if goroutine.status != GoroutineStatus::Blocked {
                    return Err(format!(
                        "non-select sender on channel {} references non-blocked goroutine {}",
                        channel_id, goroutine.id.0
                    ));
                }
                if goroutine.active_select.is_some() {
                    return Err(format!(
                        "non-select sender on channel {} references goroutine {} with active select",
                        channel_id, goroutine.id.0
                    ));
                }
                if !non_select_waiters.insert(goroutine.id.0) {
                    return Err(format!(
                        "duplicate non-select waiter for goroutine {}",
                        goroutine.id.0
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_pending_receiver(
        &self,
        goroutines_by_id: &HashMap<GoroutineId, &Goroutine>,
        active_select_waiters: &mut HashSet<(u64, u64, usize)>,
        active_select_presence: &mut HashSet<(u64, u64)>,
        non_select_waiters: &mut HashSet<u64>,
        channel_id: usize,
        receiver: &super::channels::PendingRecv,
    ) -> Result<(), String> {
        let goroutine = goroutines_by_id.get(&receiver.goroutine).ok_or_else(|| {
            format!(
                "pending receiver on channel {} references unknown goroutine {}",
                channel_id, receiver.goroutine.0
            )
        })?;
        match receiver.select.as_ref() {
            Some(select) => {
                if goroutine.active_select == Some(select.select_id) {
                    if goroutine.status != GoroutineStatus::Blocked {
                        return Err(format!(
                            "active select receiver on channel {} references non-blocked goroutine {}",
                            channel_id, goroutine.id.0
                        ));
                    }
                    let key = (goroutine.id.0, select.select_id, select.case_index);
                    if !active_select_waiters.insert(key) {
                        return Err(format!(
                            "duplicate active select waiter for goroutine {} select {} case {}",
                            goroutine.id.0, select.select_id, select.case_index
                        ));
                    }
                    active_select_presence.insert((goroutine.id.0, select.select_id));
                }
            }
            None => {
                if goroutine.status != GoroutineStatus::Blocked {
                    return Err(format!(
                        "non-select receiver on channel {} references non-blocked goroutine {}",
                        channel_id, goroutine.id.0
                    ));
                }
                if goroutine.active_select.is_some() {
                    return Err(format!(
                        "non-select receiver on channel {} references goroutine {} with active select",
                        channel_id, goroutine.id.0
                    ));
                }
                if !non_select_waiters.insert(goroutine.id.0) {
                    return Err(format!(
                        "duplicate non-select waiter for goroutine {}",
                        goroutine.id.0
                    ));
                }
            }
        }
        Ok(())
    }

    fn alloc_frame_id(&mut self) -> u64 {
        let frame_id = self.next_frame_id;
        self.next_frame_id += 1;
        frame_id
    }
}

fn build_frame(
    program: &Program,
    function_id: usize,
    args: Vec<Value>,
    return_target: ReturnTarget,
    frame_id: u64,
) -> Result<Frame, VmError> {
    let function = program
        .functions
        .get(function_id)
        .ok_or(VmError::UnknownFunction {
            function: function_id,
        })?;
    if args.len() != function.param_count {
        return Err(VmError::WrongArgumentCount {
            function: function.name.clone(),
            expected: function.param_count,
            actual: args.len(),
        });
    }

    let mut registers = vec![Value::nil(); function.register_count];
    for (index, value) in args.into_iter().enumerate() {
        registers[index] = value;
    }

    Ok(Frame {
        id: frame_id,
        function: function_id,
        pc: 0,
        registers,
        deferred: Vec::new(),
        unwind: None,
        return_target,
    })
}
