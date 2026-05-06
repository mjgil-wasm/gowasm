use super::{describe_value, ConcreteType, Program, Value, ValueData, Vm, VmError};
use crate::channels::{ChannelState, TrySendOutcome};

impl Vm {
    pub(super) fn alloc_channel_value(&mut self, capacity: usize, zero_value: Value) -> Value {
        self.alloc_channel_value_with_type(capacity, zero_value, None)
    }

    pub(super) fn alloc_channel_value_with_type(
        &mut self,
        capacity: usize,
        zero_value: Value,
        concrete_type: Option<ConcreteType>,
    ) -> Value {
        let id = self.next_channel_id;
        self.next_channel_id += 1;
        self.channels.push(ChannelState::new(capacity, zero_value));
        concrete_type
            .map(|concrete_type| Value::channel_typed(id, concrete_type))
            .unwrap_or_else(|| Value::channel(id))
    }

    pub(super) fn send_to_channel_value(
        &mut self,
        program: &Program,
        channel_id: u64,
        value: Value,
    ) -> Result<(), VmError> {
        let function = self.current_function_name(program)?;
        let value_description = describe_value(&value);
        let closed = {
            let state = self.channel_state_mut(program, channel_id)?;
            state.closed
        };
        if closed {
            return Err(VmError::SendOnClosedChannel {
                function,
                channel: format!("channel id {channel_id}"),
                value: value_description,
            });
        }

        match self.try_send_on_channel(program, channel_id, value.clone())? {
            TrySendOutcome::Delivered { receiver, value } => {
                self.complete_pending_receiver(program, receiver, value, true)
            }
            TrySendOutcome::Buffered => Ok(()),
            TrySendOutcome::NotReady => Err(VmError::UnhandledPanic {
                function,
                value: "timer channel send was not ready".into(),
            }),
        }
    }

    pub(super) fn read_channel_id(
        &self,
        program: &Program,
        value: &Value,
    ) -> Result<Option<u64>, VmError> {
        let ValueData::Channel(channel) = &value.data else {
            return Err(VmError::InvalidChannelValue {
                function: self.current_function_name(program)?,
                target: describe_value(value),
            });
        };
        Ok(channel.id)
    }

    pub(crate) fn drain_channel_buffered_value(
        &mut self,
        program: &Program,
        channel_id: u64,
    ) -> Result<bool, VmError> {
        let state = self.channel_state_mut(program, channel_id)?;
        Ok(state.buffer.pop_front().is_some())
    }

    pub(super) fn trigger_context_done_watchers(
        &mut self,
        program: &Program,
        channel_id: u64,
    ) -> Result<(), VmError> {
        let watchers = self
            .context_done_watchers
            .remove(&channel_id)
            .unwrap_or_default();
        for watcher in watchers {
            let err = self.invoke_method(program, watcher.parent, "Err", Vec::new())?;
            let error = match &err.data {
                ValueData::Nil | ValueData::Error(_) => err,
                _ => {
                    return Err(VmError::UnhandledPanic {
                        function: self.current_function_name(program)?,
                        value: "context parent Err() must return an error or nil".into(),
                    });
                }
            };
            let error = match &error.data {
                ValueData::Nil => Value::error("context canceled"),
                _ => error,
            };
            let _ = self.cancel_context_with_error(program, watcher.context_id, error)?;
        }
        Ok(())
    }

    pub(super) fn channel_state(
        &self,
        program: &Program,
        id: u64,
    ) -> Result<&ChannelState, VmError> {
        let function = self.current_function_name(program)?;
        if let Some(state) = self.channels.get(id as usize) {
            return Ok(state);
        }
        Err(VmError::UnknownChannel {
            function,
            channel: id,
        })
    }

    pub(super) fn channel_state_mut(
        &mut self,
        program: &Program,
        id: u64,
    ) -> Result<&mut ChannelState, VmError> {
        let function = self.current_function_name(program)?;
        if let Some(state) = self.channels.get_mut(id as usize) {
            return Ok(state);
        }
        Err(VmError::UnknownChannel {
            function,
            channel: id,
        })
    }
}
