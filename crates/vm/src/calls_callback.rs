use super::{
    stdlib_function_result_count, Program, ResolvedMethodCall, ReturnTarget, Value, ValueData, Vm,
    VmError,
};
use crate::{
    format_value, FunctionValue, TYPE_FS_FILE, TYPE_HTTP_REQUEST_BODY, TYPE_HTTP_RESPONSE_BODY,
};

enum CallbackOutcome {
    Returned(Vec<Value>),
    Panicked(Value),
}

impl Vm {
    pub(crate) fn invoke_callback(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
    ) -> Result<Value, VmError> {
        let results = self.invoke_callback_results(program, function, args)?;
        if results.len() != 1 {
            return Err(VmError::ReturnValueCountMismatch {
                function: self.current_function_name(program)?,
                expected: 1,
                actual: results.len(),
            });
        }
        Ok(results.into_iter().next().unwrap())
    }

    #[cfg(test)]
    pub(crate) fn invoke_callback_no_result(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
    ) -> Result<(), VmError> {
        match self.invoke_callback_no_result_or_panic(program, function, args)? {
            Ok(()) => Ok(()),
            Err(value) => Err(VmError::UnhandledPanic {
                function: self.current_function_name(program)?,
                value: format_value(&value),
            }),
        }
    }

    pub(crate) fn invoke_method(
        &mut self,
        program: &Program,
        receiver: Value,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, VmError> {
        let results = self.invoke_method_results(program, receiver, method, args)?;
        if results.len() != 1 {
            return Err(VmError::ReturnValueCountMismatch {
                function: self.current_function_name(program)?,
                expected: 1,
                actual: results.len(),
            });
        }
        Ok(results.into_iter().next().unwrap())
    }

    pub(crate) fn invoke_method_results(
        &mut self,
        program: &Program,
        receiver: Value,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, VmError> {
        match self.resolve_method_call(program, receiver, method)? {
            ResolvedMethodCall::Function { function, receiver } => {
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                values.extend(args);
                self.invoke_callback_results(program, function, values)
            }
            ResolvedMethodCall::Stdlib { function, receiver } => {
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                values.extend(args);
                if method == "Read" && values[0].typ == TYPE_FS_FILE && values.len() == 2 {
                    return self.workspace_fs_read(program, &values[0], &values[1]);
                }
                match stdlib_function_result_count(function) {
                    0 => {
                        let _ = self.execute_stdlib(program, function, &values)?;
                        Ok(Vec::new())
                    }
                    1 => Ok(vec![self.execute_stdlib(program, function, &values)?]),
                    _ => self.execute_stdlib_multi(program, function, &values),
                }
            }
        }
    }

    pub(crate) fn invoke_method_results_mutating_arg(
        &mut self,
        program: &Program,
        receiver: Value,
        method: &str,
        args: Vec<Value>,
        mutated_arg: usize,
    ) -> Result<(Vec<Value>, Value), VmError> {
        match self.resolve_method_call(program, receiver, method)? {
            ResolvedMethodCall::Function { function, receiver } => {
                if mutated_arg >= args.len() {
                    return Err(VmError::UnsupportedMutatingMethod {
                        function: self.current_function_name(program)?,
                        method: method.into(),
                    });
                }
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                values.extend(args);
                self.invoke_callback_results_mutating_arg(
                    program,
                    function,
                    values,
                    mutated_arg + 1,
                )
            }
            ResolvedMethodCall::Stdlib { receiver, .. } => {
                if method != "Read" || mutated_arg >= args.len() {
                    return Err(VmError::UnsupportedMutatingMethod {
                        function: self.current_function_name(program)?,
                        method: method.into(),
                    });
                }

                let mut buffer = args[mutated_arg].clone();
                let results = if receiver.typ == TYPE_FS_FILE {
                    self.workspace_fs_read_into(program, &receiver, &mut buffer)?
                } else if receiver.typ == TYPE_HTTP_REQUEST_BODY {
                    crate::stdlib::request_body_read_into(self, program, &receiver, &mut buffer)?
                } else if receiver.typ == TYPE_HTTP_RESPONSE_BODY {
                    crate::stdlib::response_body_read_into(self, program, &receiver, &mut buffer)?
                } else {
                    return Err(VmError::UnsupportedMutatingMethod {
                        function: self.current_function_name(program)?,
                        method: method.into(),
                    });
                };
                Ok((results, buffer))
            }
        }
    }

    pub(crate) fn supports_method_results_mutating_arg(
        &self,
        program: &Program,
        receiver: Value,
        method: &str,
        mutated_arg: usize,
        arg_count: usize,
    ) -> Result<bool, VmError> {
        match self.resolve_method_call(program, receiver, method)? {
            ResolvedMethodCall::Function { .. } => Ok(mutated_arg < arg_count),
            ResolvedMethodCall::Stdlib { receiver, .. } => {
                if method != "Read" || mutated_arg >= arg_count {
                    return Ok(false);
                }
                if receiver.typ == TYPE_HTTP_REQUEST_BODY {
                    let Some(body_id) = crate::stdlib::request_body_id(&receiver) else {
                        return Ok(false);
                    };
                    let Some(state) = self.http_request_bodies.get(&body_id) else {
                        return Ok(false);
                    };
                    return self.supports_method_results_mutating_arg(
                        program,
                        state.reader.clone(),
                        method,
                        mutated_arg,
                        arg_count,
                    );
                }
                Ok(matches!(
                    receiver.typ,
                    TYPE_FS_FILE | TYPE_HTTP_RESPONSE_BODY
                ))
            }
        }
    }

    pub(crate) fn invoke_method_results_mutating_receiver_and_arg(
        &mut self,
        program: &Program,
        receiver: Value,
        method: &str,
        args: Vec<Value>,
        mutated_arg: usize,
    ) -> Result<(Vec<Value>, Value, Value), VmError> {
        match self.resolve_method_call(program, receiver, method)? {
            ResolvedMethodCall::Function { function, receiver } => {
                if mutated_arg >= args.len() {
                    return Err(VmError::UnsupportedMutatingMethod {
                        function: self.current_function_name(program)?,
                        method: method.into(),
                    });
                }
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                values.extend(args);
                let (results, mut captured) = self.invoke_callback_results_capturing_args(
                    program,
                    function,
                    values,
                    vec![0, mutated_arg + 1],
                )?;
                let receiver = captured.remove(0);
                let arg = captured.remove(0);
                Ok((results, receiver, arg))
            }
            ResolvedMethodCall::Stdlib { receiver, .. } => {
                let (results, mutated_arg_value) = self.invoke_method_results_mutating_arg(
                    program,
                    receiver.clone(),
                    method,
                    args,
                    mutated_arg,
                )?;
                Ok((results, receiver, mutated_arg_value))
            }
        }
    }

    fn invoke_callback_results(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, VmError> {
        match self.invoke_callback_outcome(program, function, args)? {
            CallbackOutcome::Returned(results) => Ok(results),
            CallbackOutcome::Panicked(value) => Err(VmError::UnhandledPanic {
                function: self.current_function_name(program)?,
                value: format_value(&value),
            }),
        }
    }

    pub(crate) fn invoke_callback_no_result_or_panic(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
    ) -> Result<Result<(), Value>, VmError> {
        match self.invoke_callback_outcome(program, function, args)? {
            CallbackOutcome::Returned(results) => {
                if !results.is_empty() {
                    return Err(VmError::ReturnValueCountMismatch {
                        function: self.current_function_name(program)?,
                        expected: 0,
                        actual: results.len(),
                    });
                }
                Ok(Ok(()))
            }
            CallbackOutcome::Panicked(value) => Ok(Err(value)),
        }
    }

    fn invoke_callback_outcome(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
    ) -> Result<CallbackOutcome, VmError> {
        self.callback_result = None;
        self.callback_panic = None;
        self.callback_capture_arg_indices = None;
        self.callback_captured_args = None;
        self.push_frame_on_current_goroutine(program, function, args, ReturnTarget::Callback)?;

        loop {
            if let Some(results) = self.callback_result.take() {
                return Ok(CallbackOutcome::Returned(results));
            }
            if let Some(value) = self.callback_panic.take() {
                return Ok(CallbackOutcome::Panicked(value));
            }
            if !self.advance_to_next_runnable() {
                if self.has_blocked_goroutines() {
                    return Err(VmError::Deadlock);
                }
                return Err(VmError::UnsupportedConcurrencyOpcode {
                    opcode: "callback scheduler blocked without runnable goroutine".into(),
                });
            }
            if let Some(error) = self.take_pending_error_for_current_goroutine() {
                return Err(error);
            }
            self.ensure_instruction_budget(program)?;
            let instruction = self.fetch_next_instruction(program)?;
            match self.execute_instruction(program, instruction) {
                Ok(()) => self.finish_executed_instruction(program),
                Err(error) => {
                    if !matches!(error, VmError::CapabilityRequest { .. }) {
                        self.finish_executed_instruction(program);
                    }
                    return Err(error);
                }
            }
        }
    }

    fn invoke_callback_results_mutating_arg(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
        mutated_arg: usize,
    ) -> Result<(Vec<Value>, Value), VmError> {
        let (results, mut captured) = self.invoke_callback_results_capturing_args(
            program,
            function,
            args,
            vec![mutated_arg],
        )?;
        Ok((results, captured.remove(0)))
    }

    fn invoke_callback_results_capturing_args(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<Value>,
        captured_args: Vec<usize>,
    ) -> Result<(Vec<Value>, Vec<Value>), VmError> {
        self.callback_result = None;
        self.callback_panic = None;
        self.callback_capture_arg_indices = Some(captured_args);
        self.callback_captured_args = None;
        self.push_frame_on_current_goroutine(program, function, args, ReturnTarget::Callback)?;

        loop {
            if let Some(results) = self.callback_result.take() {
                let captured = self.callback_captured_args.take().ok_or_else(|| {
                    VmError::UnsupportedConcurrencyOpcode {
                        opcode: "callback return lost captured argument values".into(),
                    }
                })?;
                self.callback_capture_arg_indices = None;
                return Ok((results, captured));
            }
            if let Some(value) = self.callback_panic.take() {
                self.callback_capture_arg_indices = None;
                self.callback_captured_args = None;
                return Err(VmError::UnhandledPanic {
                    function: self.current_function_name(program)?,
                    value: format_value(&value),
                });
            }
            if !self.advance_to_next_runnable() {
                if self.has_blocked_goroutines() {
                    return Err(VmError::Deadlock);
                }
                return Err(VmError::UnsupportedConcurrencyOpcode {
                    opcode: "callback scheduler blocked without runnable goroutine".into(),
                });
            }
            if let Some(error) = self.take_pending_error_for_current_goroutine() {
                return Err(error);
            }
            self.ensure_instruction_budget(program)?;
            let instruction = self.fetch_next_instruction(program)?;
            match self.execute_instruction(program, instruction) {
                Ok(()) => self.finish_executed_instruction(program),
                Err(error) => {
                    if !matches!(error, VmError::CapabilityRequest { .. }) {
                        self.finish_executed_instruction(program);
                    }
                    return Err(error);
                }
            }
        }
    }

    pub(super) fn default_results_for_target(&self, target: &ReturnTarget) -> Vec<Value> {
        match target {
            ReturnTarget::None | ReturnTarget::Deferred | ReturnTarget::Callback => Vec::new(),
            ReturnTarget::One(_) => vec![Value::nil()],
            ReturnTarget::Many(dsts) => vec![Value::nil(); dsts.len()],
        }
    }

    pub(super) fn read_function_value(
        &self,
        program: &Program,
        register: usize,
    ) -> Result<(usize, Vec<Value>), VmError> {
        let function = self.read_function_struct(program, register)?;
        Ok((function.function, function.captures))
    }

    pub(super) fn read_function_struct(
        &self,
        program: &Program,
        register: usize,
    ) -> Result<FunctionValue, VmError> {
        let value = self.read_register(program, register)?;
        let ValueData::Function(function) = value.data else {
            return Err(VmError::InvalidFunctionValue {
                function: self.current_function_name(program)?,
                target: crate::describe_value(&value),
            });
        };
        Ok(function)
    }
}
