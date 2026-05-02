use std::collections::VecDeque;

use super::{describe_value, scheduler::GoroutineId, Program, Value, Vm, VmError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingSend {
    pub(super) goroutine: GoroutineId,
    pub(super) value: Value,
    pub(super) select: Option<SelectWait>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingRecv {
    pub(super) goroutine: GoroutineId,
    pub(super) dst: usize,
    pub(super) ok_dst: Option<usize>,
    pub(super) select: Option<SelectWait>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SelectWait {
    pub(super) select_id: u64,
    pub(super) case_index: usize,
    pub(super) choice_dst: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ChannelState {
    pub(super) capacity: usize,
    pub(super) closed: bool,
    pub(super) buffer: VecDeque<Value>,
    pub(super) pending_sends: VecDeque<PendingSend>,
    pub(super) pending_receivers: VecDeque<PendingRecv>,
    pub(super) zero_value: Value,
}

impl ChannelState {
    pub(super) fn new(capacity: usize, zero_value: Value) -> Self {
        Self {
            capacity,
            closed: false,
            buffer: VecDeque::new(),
            pending_sends: VecDeque::new(),
            pending_receivers: VecDeque::new(),
            zero_value,
        }
    }
}

enum SendOutcome {
    Delivered { receiver: PendingRecv, value: Value },
    Buffered,
    Blocked,
}

enum ReceiveOutcome {
    Received {
        sender: Box<Option<PendingSend>>,
        value: Value,
        ok: bool,
    },
    Blocked,
}

enum TryReceiveOutcome {
    Received {
        sender: Box<Option<PendingSend>>,
        value: Value,
        ok: bool,
    },
    NotReady,
}

pub(super) enum TrySendOutcome {
    Delivered { receiver: PendingRecv, value: Value },
    Buffered,
    NotReady,
}

impl Vm {
    pub(super) fn execute_chan_send(
        &mut self,
        program: &Program,
        chan: usize,
        value: usize,
    ) -> Result<(), VmError> {
        let chan = self.read_register(program, chan)?;
        let value = self.read_register(program, value)?;
        let Some(channel_id) = self.read_channel_id(program, &chan)? else {
            self.block_current_goroutine();
            return Ok(());
        };

        let current = self.current_goroutine_id();
        let function = self.current_function_name(program)?;
        let channel = describe_value(&chan);
        let value_description = describe_value(&value);
        let closed = {
            let state = self.channel_state_mut(program, channel_id)?;
            state.closed
        };
        if closed {
            return Err(VmError::SendOnClosedChannel {
                function,
                channel,
                value: value_description,
            });
        }
        let outcome = match self.try_send_on_channel(program, channel_id, value.clone())? {
            TrySendOutcome::Delivered { receiver, value } => {
                SendOutcome::Delivered { receiver, value }
            }
            TrySendOutcome::Buffered => SendOutcome::Buffered,
            TrySendOutcome::NotReady => {
                let state = self.channel_state_mut(program, channel_id)?;
                state.pending_sends.push_back(PendingSend {
                    goroutine: current,
                    value,
                    select: None,
                });
                SendOutcome::Blocked
            }
        };

        match outcome {
            SendOutcome::Delivered { receiver, value } => {
                self.complete_pending_receiver(program, receiver, value, true)?;
            }
            SendOutcome::Buffered => {}
            SendOutcome::Blocked => self.block_current_goroutine(),
        }
        self.debug_assert_channel_wait_queue_invariants();
        Ok(())
    }

    pub(super) fn execute_close_channel(
        &mut self,
        program: &Program,
        chan: usize,
    ) -> Result<(), VmError> {
        let chan = self.read_register(program, chan)?;
        let Some(channel_id) = self.read_channel_id(program, &chan)? else {
            return Err(VmError::CloseNilChannel {
                function: self.current_function_name(program)?,
                channel: describe_value(&chan),
            });
        };

        let function = self.current_function_name(program)?;
        let channel = describe_value(&chan);
        let (pending_receivers, pending_senders, zero_value) = {
            let state = self.channel_state_mut(program, channel_id)?;
            if state.closed {
                return Err(VmError::CloseClosedChannel { function, channel });
            }
            state.closed = true;
            let pending_receivers = state.pending_receivers.drain(..).collect::<Vec<_>>();
            let pending_senders = state.pending_sends.drain(..).collect::<Vec<_>>();
            (pending_receivers, pending_senders, state.zero_value.clone())
        };

        for receiver in pending_receivers {
            self.complete_pending_receiver(program, receiver, zero_value.clone(), false)?;
        }
        for sender in pending_senders {
            self.complete_pending_sender_closed(program, channel_id, sender)?;
        }
        self.trigger_context_done_watchers(program, channel_id)?;
        self.debug_assert_channel_wait_queue_invariants();
        Ok(())
    }

    pub(crate) fn close_channel_by_id(
        &mut self,
        program: &Program,
        channel_id: u64,
    ) -> Result<(), VmError> {
        let (pending_receivers, pending_senders, zero_value, already_closed) = {
            let state = self.channel_state_mut(program, channel_id)?;
            let already_closed = state.closed;
            if !already_closed {
                state.closed = true;
            }
            let pending_receivers = state.pending_receivers.drain(..).collect::<Vec<_>>();
            let pending_senders = state.pending_sends.drain(..).collect::<Vec<_>>();
            (
                pending_receivers,
                pending_senders,
                state.zero_value.clone(),
                already_closed,
            )
        };

        if already_closed {
            return Ok(());
        }

        for receiver in pending_receivers {
            self.complete_pending_receiver(program, receiver, zero_value.clone(), false)?;
        }
        for sender in pending_senders {
            self.complete_pending_sender_closed(program, channel_id, sender)?;
        }
        self.trigger_context_done_watchers(program, channel_id)?;
        self.debug_assert_channel_wait_queue_invariants();
        Ok(())
    }

    pub(super) fn execute_chan_recv(
        &mut self,
        program: &Program,
        dst: usize,
        chan: usize,
    ) -> Result<(), VmError> {
        self.execute_chan_recv_inner(program, dst, None, chan)
    }

    pub(super) fn execute_chan_recv_ok(
        &mut self,
        program: &Program,
        value_dst: usize,
        ok_dst: usize,
        chan: usize,
    ) -> Result<(), VmError> {
        self.execute_chan_recv_inner(program, value_dst, Some(ok_dst), chan)
    }

    pub(super) fn execute_select(
        &mut self,
        program: &Program,
        choice_dst: usize,
        cases: &[super::SelectCaseOp],
        default_case: Option<usize>,
    ) -> Result<(), VmError> {
        let mut resolved_cases = Vec::with_capacity(cases.len());
        for case in cases {
            let chan = self.read_register(program, case.chan)?;
            resolved_cases.push((self.read_channel_id(program, &chan)?, case.clone()));
        }

        let start = self.next_select_start(resolved_cases.len());
        for offset in 0..resolved_cases.len() {
            let case_index = (start + offset) % resolved_cases.len();
            let (channel_id, case) = &resolved_cases[case_index];
            let Some(channel_id) = channel_id else {
                continue;
            };
            match &case.kind {
                super::SelectCaseOpKind::Recv { value_dst, ok_dst } => {
                    match self.try_receive_on_channel(program, *channel_id)? {
                        TryReceiveOutcome::Received { sender, value, ok } => {
                            self.set_register(program, choice_dst, Value::int(case_index as i64))?;
                            self.set_register(program, *value_dst, value)?;
                            if let Some(ok_dst) = ok_dst {
                                self.set_register(program, *ok_dst, Value::bool(ok))?;
                            }
                            if let Some(ref sender) = *sender {
                                self.complete_pending_sender_success(program, (*sender).clone())?;
                            }
                            return Ok(());
                        }
                        TryReceiveOutcome::NotReady => {}
                    }
                }
                super::SelectCaseOpKind::Send { value } => {
                    let channel = describe_value(&self.read_register(program, case.chan)?);
                    let value = self.read_register(program, *value)?;
                    let value_description = describe_value(&value);
                    let closed = {
                        let state = self.channel_state_mut(program, *channel_id)?;
                        state.closed
                    };
                    if closed {
                        return Err(VmError::SendOnClosedChannel {
                            function: self.current_function_name(program)?,
                            channel,
                            value: value_description,
                        });
                    }
                    match self.try_send_on_channel(program, *channel_id, value)? {
                        TrySendOutcome::Delivered { receiver, value } => {
                            self.set_register(program, choice_dst, Value::int(case_index as i64))?;
                            self.complete_pending_receiver(program, receiver, value, true)?;
                            return Ok(());
                        }
                        TrySendOutcome::Buffered => {
                            self.set_register(program, choice_dst, Value::int(case_index as i64))?;
                            return Ok(());
                        }
                        TrySendOutcome::NotReady => {}
                    }
                }
            }
        }

        if let Some(default_case) = default_case {
            self.set_register(program, choice_dst, Value::int(default_case as i64))?;
            return Ok(());
        }

        let current = self.current_goroutine_id();
        let select_id = self.alloc_select_id();
        let mut queued_waiter = false;
        for (case_index, (channel_id, case)) in resolved_cases.into_iter().enumerate() {
            let Some(channel_id) = channel_id else {
                continue;
            };
            queued_waiter = true;
            match case.kind {
                super::SelectCaseOpKind::Recv { value_dst, ok_dst } => {
                    let state = self.channel_state_mut(program, channel_id)?;
                    state.pending_receivers.push_back(PendingRecv {
                        goroutine: current,
                        dst: value_dst,
                        ok_dst,
                        select: Some(SelectWait {
                            select_id,
                            case_index,
                            choice_dst,
                        }),
                    });
                }
                super::SelectCaseOpKind::Send { value } => {
                    let value = self.read_register(program, value)?;
                    let state = self.channel_state_mut(program, channel_id)?;
                    state.pending_sends.push_back(PendingSend {
                        goroutine: current,
                        value,
                        select: Some(SelectWait {
                            select_id,
                            case_index,
                            choice_dst,
                        }),
                    });
                }
            }
        }
        if queued_waiter {
            self.set_current_goroutine_select(Some(select_id));
        }
        self.block_current_goroutine();
        self.debug_assert_channel_wait_queue_invariants();
        Ok(())
    }

    pub(super) fn execute_chan_try_recv(
        &mut self,
        program: &Program,
        ready_dst: usize,
        value_dst: usize,
        chan: usize,
    ) -> Result<(), VmError> {
        self.execute_chan_try_recv_inner(program, ready_dst, value_dst, None, chan)
    }

    pub(super) fn execute_chan_try_recv_ok(
        &mut self,
        program: &Program,
        ready_dst: usize,
        value_dst: usize,
        ok_dst: usize,
        chan: usize,
    ) -> Result<(), VmError> {
        self.execute_chan_try_recv_inner(program, ready_dst, value_dst, Some(ok_dst), chan)
    }

    pub(super) fn execute_chan_try_send(
        &mut self,
        program: &Program,
        ready_dst: usize,
        chan: usize,
        value: usize,
    ) -> Result<(), VmError> {
        let chan = self.read_register(program, chan)?;
        let value = self.read_register(program, value)?;
        let Some(channel_id) = self.read_channel_id(program, &chan)? else {
            self.set_register(program, ready_dst, Value::bool(false))?;
            return Ok(());
        };

        let function = self.current_function_name(program)?;
        let channel = describe_value(&chan);
        let value_description = describe_value(&value);
        let closed = {
            let state = self.channel_state_mut(program, channel_id)?;
            state.closed
        };
        if closed {
            return Err(VmError::SendOnClosedChannel {
                function,
                channel,
                value: value_description,
            });
        }
        let outcome = match self.try_send_on_channel(program, channel_id, value)? {
            TrySendOutcome::Delivered { receiver, value } => {
                SendOutcome::Delivered { receiver, value }
            }
            TrySendOutcome::Buffered => SendOutcome::Buffered,
            TrySendOutcome::NotReady => SendOutcome::Blocked,
        };

        match outcome {
            SendOutcome::Delivered { receiver, value } => {
                self.complete_pending_receiver(program, receiver, value, true)?;
                self.set_register(program, ready_dst, Value::bool(true))?;
            }
            SendOutcome::Buffered => {
                self.set_register(program, ready_dst, Value::bool(true))?;
            }
            SendOutcome::Blocked => {
                self.set_register(program, ready_dst, Value::bool(false))?;
            }
        }
        Ok(())
    }

    fn execute_chan_recv_inner(
        &mut self,
        program: &Program,
        dst: usize,
        ok_dst: Option<usize>,
        chan: usize,
    ) -> Result<(), VmError> {
        let chan = self.read_register(program, chan)?;
        let Some(channel_id) = self.read_channel_id(program, &chan)? else {
            self.block_current_goroutine();
            return Ok(());
        };

        let current = self.current_goroutine_id();
        let outcome = match self.try_receive_on_channel(program, channel_id)? {
            TryReceiveOutcome::Received { sender, value, ok } => {
                ReceiveOutcome::Received { sender, value, ok }
            }
            TryReceiveOutcome::NotReady => {
                let state = self.channel_state_mut(program, channel_id)?;
                state.pending_receivers.push_back(PendingRecv {
                    goroutine: current,
                    dst,
                    ok_dst,
                    select: None,
                });
                ReceiveOutcome::Blocked
            }
        };

        match outcome {
            ReceiveOutcome::Received { sender, value, ok } => {
                self.set_register(program, dst, value)?;
                if let Some(ok_dst) = ok_dst {
                    self.set_register(program, ok_dst, Value::bool(ok))?;
                }
                if let Some(ref sender) = *sender {
                    self.complete_pending_sender_success(program, sender.clone())?;
                }
            }
            ReceiveOutcome::Blocked => self.block_current_goroutine(),
        }
        self.debug_assert_channel_wait_queue_invariants();
        Ok(())
    }

    fn execute_chan_try_recv_inner(
        &mut self,
        program: &Program,
        ready_dst: usize,
        value_dst: usize,
        ok_dst: Option<usize>,
        chan: usize,
    ) -> Result<(), VmError> {
        let chan = self.read_register(program, chan)?;
        let Some(channel_id) = self.read_channel_id(program, &chan)? else {
            self.set_register(program, ready_dst, Value::bool(false))?;
            if let Some(ok_dst) = ok_dst {
                self.set_register(program, ok_dst, Value::bool(false))?;
            }
            return Ok(());
        };

        match self.try_receive_on_channel(program, channel_id)? {
            TryReceiveOutcome::Received { sender, value, ok } => {
                self.set_register(program, value_dst, value)?;
                self.set_register(program, ready_dst, Value::bool(true))?;
                if let Some(ok_dst) = ok_dst {
                    self.set_register(program, ok_dst, Value::bool(ok))?;
                }
                if let Some(ref sender) = *sender {
                    self.complete_pending_sender_success(program, sender.clone())?;
                }
            }
            TryReceiveOutcome::NotReady => {
                self.set_register(program, ready_dst, Value::bool(false))?;
                if let Some(ok_dst) = ok_dst {
                    self.set_register(program, ok_dst, Value::bool(false))?;
                }
            }
        }
        Ok(())
    }

    fn try_receive_on_channel(
        &mut self,
        program: &Program,
        channel_id: u64,
    ) -> Result<TryReceiveOutcome, VmError> {
        let buffered_value = {
            let state = self.channel_state_mut(program, channel_id)?;
            state.buffer.pop_front()
        };
        if let Some(value) = buffered_value {
            if let Some(sender) = self.pop_ready_sender(program, channel_id)? {
                let refill = sender.value.clone();
                let state = self.channel_state_mut(program, channel_id)?;
                state.buffer.push_back(refill);
                return Ok(TryReceiveOutcome::Received {
                    sender: Box::new(Some(sender)),
                    value,
                    ok: true,
                });
            }
            return Ok(TryReceiveOutcome::Received {
                sender: Box::new(None),
                value,
                ok: true,
            });
        }
        if let Some(sender) = self.pop_ready_sender(program, channel_id)? {
            let value = sender.value.clone();
            return Ok(TryReceiveOutcome::Received {
                sender: Box::new(Some(sender)),
                value,
                ok: true,
            });
        }
        let (closed, zero_value) = {
            let state = self.channel_state_mut(program, channel_id)?;
            (state.closed, state.zero_value.clone())
        };
        if closed {
            return Ok(TryReceiveOutcome::Received {
                sender: Box::new(None),
                value: zero_value,
                ok: false,
            });
        }
        Ok(TryReceiveOutcome::NotReady)
    }

    pub(super) fn try_send_on_channel(
        &mut self,
        program: &Program,
        channel_id: u64,
        value: Value,
    ) -> Result<TrySendOutcome, VmError> {
        if let Some(receiver) = self.pop_ready_receiver(program, channel_id)? {
            return Ok(TrySendOutcome::Delivered { receiver, value });
        }
        let state = self.channel_state_mut(program, channel_id)?;
        if state.buffer.len() < state.capacity {
            state.buffer.push_back(value);
            Ok(TrySendOutcome::Buffered)
        } else {
            Ok(TrySendOutcome::NotReady)
        }
    }

    fn pop_ready_receiver(
        &mut self,
        program: &Program,
        channel_id: u64,
    ) -> Result<Option<PendingRecv>, VmError> {
        loop {
            let receiver = {
                let state = self.channel_state_mut(program, channel_id)?;
                state.pending_receivers.pop_front()
            };
            let Some(receiver) = receiver else {
                return Ok(None);
            };
            if self.pending_receiver_is_ready(&receiver) {
                return Ok(Some(receiver));
            }
        }
    }

    fn pop_ready_sender(
        &mut self,
        program: &Program,
        channel_id: u64,
    ) -> Result<Option<PendingSend>, VmError> {
        loop {
            let sender = {
                let state = self.channel_state_mut(program, channel_id)?;
                state.pending_sends.pop_front()
            };
            let Some(sender) = sender else {
                return Ok(None);
            };
            if self.pending_sender_is_ready(&sender) {
                return Ok(Some(sender));
            }
        }
    }

    fn pending_receiver_is_ready(&self, receiver: &PendingRecv) -> bool {
        match receiver.select.as_ref() {
            Some(select) => self.goroutine_has_active_select(receiver.goroutine, select.select_id),
            None => true,
        }
    }

    fn pending_sender_is_ready(&self, sender: &PendingSend) -> bool {
        match sender.select.as_ref() {
            Some(select) => self.goroutine_has_active_select(sender.goroutine, select.select_id),
            None => true,
        }
    }

    pub(super) fn complete_pending_receiver(
        &mut self,
        program: &Program,
        receiver: PendingRecv,
        value: Value,
        ok: bool,
    ) -> Result<(), VmError> {
        if let Some(select) = receiver.select.as_ref() {
            if !self.goroutine_has_active_select(receiver.goroutine, select.select_id) {
                return Ok(());
            }
            self.set_register_on_goroutine(
                program,
                receiver.goroutine,
                select.choice_dst,
                Value::int(select.case_index as i64),
            )?;
            self.clear_goroutine_select(receiver.goroutine, select.select_id);
        }
        self.set_register_on_goroutine(program, receiver.goroutine, receiver.dst, value)?;
        if let Some(ok_dst) = receiver.ok_dst {
            self.set_register_on_goroutine(program, receiver.goroutine, ok_dst, Value::bool(ok))?;
        }
        self.wake_goroutine(receiver.goroutine);
        Ok(())
    }

    fn complete_pending_sender_success(
        &mut self,
        program: &Program,
        sender: PendingSend,
    ) -> Result<(), VmError> {
        if let Some(select) = sender.select.as_ref() {
            if !self.goroutine_has_active_select(sender.goroutine, select.select_id) {
                return Ok(());
            }
            self.set_register_on_goroutine(
                program,
                sender.goroutine,
                select.choice_dst,
                Value::int(select.case_index as i64),
            )?;
            self.clear_goroutine_select(sender.goroutine, select.select_id);
        }
        self.wake_goroutine(sender.goroutine);
        Ok(())
    }

    fn complete_pending_sender_closed(
        &mut self,
        program: &Program,
        channel_id: u64,
        sender: PendingSend,
    ) -> Result<(), VmError> {
        if let Some(select) = sender.select.as_ref() {
            if !self.goroutine_has_active_select(sender.goroutine, select.select_id) {
                return Ok(());
            }
            self.clear_goroutine_select(sender.goroutine, select.select_id);
        }
        let function = self.goroutine_function_name(program, sender.goroutine)?;
        self.set_pending_error_on_goroutine(
            sender.goroutine,
            VmError::SendOnClosedChannel {
                function,
                channel: format!("channel id {channel_id}"),
                value: describe_value(&sender.value),
            },
        )?;
        self.wake_goroutine(sender.goroutine);
        Ok(())
    }
}
