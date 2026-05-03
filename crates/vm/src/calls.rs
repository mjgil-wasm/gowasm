use super::{
    function_name, resolve_stdlib_runtime_method, stdlib_function_param_types,
    stdlib_function_result_count, DeferredCall, DeferredCallKind, Instruction, Program,
    PromotedFieldAccess, PromotedFieldStep, ReturnTarget, StdlibFunctionId, TypeId, UnwindState,
    Value, ValueData, Vm, VmError, TYPE_ANY, TYPE_FS_FILE, TYPE_POINTER,
};

#[path = "calls_callback.rs"]
mod callback_impl;

enum ResolvedMethodCall {
    Function {
        function: usize,
        receiver: Value,
    },
    Stdlib {
        function: StdlibFunctionId,
        receiver: Value,
    },
}

impl Vm {
    pub(super) fn execute_call_function(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<usize>,
        dst: Option<usize>,
    ) -> Result<(), VmError> {
        let values = args
            .iter()
            .map(|register| self.read_register(program, *register))
            .collect::<Result<Vec<_>, _>>()?;
        self.push_frame_on_current_goroutine(
            program,
            function,
            values,
            match dst {
                Some(dst) => ReturnTarget::One(dst),
                None => ReturnTarget::None,
            },
        )
    }

    pub(super) fn execute_call_function_multi(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<usize>,
        dsts: Vec<usize>,
    ) -> Result<(), VmError> {
        let values = args
            .iter()
            .map(|register| self.read_register(program, *register))
            .collect::<Result<Vec<_>, _>>()?;
        self.push_frame_on_current_goroutine(program, function, values, ReturnTarget::Many(dsts))
    }

    pub(super) fn execute_call_closure(
        &mut self,
        program: &Program,
        callee: usize,
        args: Vec<usize>,
        dst: Option<usize>,
    ) -> Result<(), VmError> {
        let (function, captures) = self.read_function_value(program, callee)?;
        let mut values = captures;
        values.extend(self.read_register_list(program, &args)?);
        self.push_frame_on_current_goroutine(
            program,
            function,
            values,
            match dst {
                Some(dst) => ReturnTarget::One(dst),
                None => ReturnTarget::None,
            },
        )
    }

    pub(super) fn execute_call_closure_multi(
        &mut self,
        program: &Program,
        callee: usize,
        args: Vec<usize>,
        dsts: Vec<usize>,
    ) -> Result<(), VmError> {
        let (function, captures) = self.read_function_value(program, callee)?;
        let mut values = captures;
        values.extend(self.read_register_list(program, &args)?);
        self.push_frame_on_current_goroutine(program, function, values, ReturnTarget::Many(dsts))
    }

    pub(super) fn execute_go_call_closure(
        &mut self,
        program: &Program,
        callee: usize,
        args: Vec<usize>,
    ) -> Result<(), VmError> {
        let (function, captures) = self.read_function_value(program, callee)?;
        let mut values = captures;
        values.extend(self.read_register_list(program, &args)?);
        self.spawn_goroutine(program, function, values)?;
        Ok(())
    }

    pub(super) fn execute_call_stdlib_multi(
        &mut self,
        program: &Program,
        function: StdlibFunctionId,
        args: Vec<usize>,
        dsts: Vec<usize>,
    ) -> Result<(), VmError> {
        let values = args
            .iter()
            .map(|register| self.read_register(program, *register))
            .collect::<Result<Vec<_>, _>>()?;
        let results = self.execute_stdlib_multi(program, function, &values)?;
        if !dsts.is_empty() && results.len() != dsts.len() {
            return Err(VmError::ReturnValueCountMismatch {
                function: self.current_function_name(program)?,
                expected: dsts.len(),
                actual: results.len(),
            });
        }
        for (dst, value) in dsts.into_iter().zip(results) {
            self.set_register(program, dst, value)?;
        }
        Ok(())
    }

    pub(super) fn execute_defer_stdlib(
        &mut self,
        program: &Program,
        function: StdlibFunctionId,
        args: Vec<usize>,
    ) -> Result<(), VmError> {
        let values = self.read_register_list(program, &args)?;
        self.current_frame_mut().deferred.push(DeferredCall {
            kind: DeferredCallKind::Stdlib { function },
            args: values,
        });
        Ok(())
    }

    pub(super) fn execute_defer_closure(
        &mut self,
        program: &Program,
        callee: usize,
        args: Vec<usize>,
    ) -> Result<(), VmError> {
        let function = self.read_register(program, callee)?;
        if !matches!(&function.data, ValueData::Function(_)) {
            return Err(VmError::InvalidFunctionValue {
                function: self.current_function_name(program)?,
                target: super::describe_value(&function),
            });
        }
        let args = self.read_register_list(program, &args)?;
        self.current_frame_mut().deferred.push(DeferredCall {
            kind: DeferredCallKind::Closure { function },
            args,
        });
        Ok(())
    }

    pub(super) fn execute_call_method(
        &mut self,
        program: &Program,
        receiver: usize,
        method: String,
        args: Vec<usize>,
        dst: Option<usize>,
    ) -> Result<(), VmError> {
        let receiver_value = self.read_register(program, receiver)?;
        match self.resolve_method_call(program, receiver_value, &method)? {
            ResolvedMethodCall::Function { function, receiver } => {
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                for register in args {
                    values.push(self.read_register(program, register)?);
                }
                self.push_frame_on_current_goroutine(
                    program,
                    function,
                    values,
                    match dst {
                        Some(dst) => ReturnTarget::One(dst),
                        None => ReturnTarget::None,
                    },
                )
            }
            ResolvedMethodCall::Stdlib { function, receiver } => {
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                values.extend(self.read_register_list(program, &args)?);
                let result = self.execute_stdlib(program, function, &values)?;
                if let Some(dst) = dst {
                    self.set_register(program, dst, result)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn execute_call_method_multi(
        &mut self,
        program: &Program,
        receiver: usize,
        method: String,
        args: Vec<usize>,
        dsts: Vec<usize>,
    ) -> Result<(), VmError> {
        let receiver_value = self.read_register(program, receiver)?;
        match self.resolve_method_call(program, receiver_value, &method)? {
            ResolvedMethodCall::Function { function, receiver } => {
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                for register in args {
                    values.push(self.read_register(program, register)?);
                }
                self.push_frame_on_current_goroutine(
                    program,
                    function,
                    values,
                    ReturnTarget::Many(dsts),
                )
            }
            ResolvedMethodCall::Stdlib { function, receiver } => {
                let mut values = Vec::with_capacity(args.len() + 1);
                values.push(receiver);
                values.extend(self.read_register_list(program, &args)?);
                if method == "Read" && values[0].typ == TYPE_FS_FILE && values.len() == 2 {
                    let results = self.workspace_fs_read(program, &values[0], &values[1])?;
                    if results.len() != dsts.len() {
                        return Err(VmError::ReturnValueCountMismatch {
                            function: self.current_function_name(program)?,
                            expected: dsts.len(),
                            actual: results.len(),
                        });
                    }
                    for (dst, value) in dsts.into_iter().zip(results) {
                        self.set_register(program, dst, value)?;
                    }
                    return Ok(());
                }
                if stdlib_function_result_count(function) <= 1 {
                    let result = self.execute_stdlib(program, function, &values)?;
                    if let Some(dst) = dsts.first().copied() {
                        self.set_register(program, dst, result)?;
                    }
                    return Ok(());
                }
                let results = self.execute_stdlib_multi(program, function, &values)?;
                if results.len() != dsts.len() {
                    return Err(VmError::ReturnValueCountMismatch {
                        function: self.current_function_name(program)?,
                        expected: dsts.len(),
                        actual: results.len(),
                    });
                }
                for (dst, value) in dsts.into_iter().zip(results) {
                    self.set_register(program, dst, value)?;
                }
                Ok(())
            }
        }
    }

    pub(super) fn execute_call_method_multi_mutating_arg(
        &mut self,
        program: &Program,
        receiver: usize,
        method: String,
        args: Vec<usize>,
        dsts: Vec<usize>,
        mutated_arg: usize,
    ) -> Result<(), VmError> {
        let receiver_value = self.read_register(program, receiver)?;
        let Some(mutated_register) = args.get(mutated_arg).copied() else {
            return Err(VmError::UnsupportedMutatingMethod {
                function: self.current_function_name(program)?,
                method,
            });
        };
        let values = self.read_register_list(program, &args)?;
        let (results, mutated_value) = self.invoke_method_results_mutating_arg(
            program,
            receiver_value,
            &method,
            values,
            mutated_arg,
        )?;
        if results.len() != dsts.len() {
            return Err(VmError::ReturnValueCountMismatch {
                function: self.current_function_name(program)?,
                expected: dsts.len(),
                actual: results.len(),
            });
        }
        self.set_register(program, mutated_register, mutated_value)?;
        for (dst, value) in dsts.into_iter().zip(results) {
            self.set_register(program, dst, value)?;
        }
        Ok(())
    }

    pub(super) fn execute_go_call_method(
        &mut self,
        program: &Program,
        receiver: usize,
        method: String,
        args: Vec<usize>,
    ) -> Result<(), VmError> {
        let receiver_value = self.read_register(program, receiver)?;
        let ResolvedMethodCall::Function { function, receiver } =
            self.resolve_method_call(program, receiver_value, &method)?
        else {
            return Err(VmError::UnhandledPanic {
                function: self.current_function_name(program)?,
                value: format!(
                    "go statements do not yet support stdlib-backed interface method `{method}`"
                ),
            });
        };
        let mut values = Vec::with_capacity(args.len() + 1);
        values.push(receiver);
        for register in args {
            values.push(self.read_register(program, register)?);
        }
        self.spawn_goroutine(program, function, values)?;
        Ok(())
    }

    pub(super) fn execute_defer_function(
        &mut self,
        program: &Program,
        function: usize,
        args: Vec<usize>,
    ) -> Result<(), VmError> {
        let values = self.read_register_list(program, &args)?;
        self.current_frame_mut().deferred.push(DeferredCall {
            kind: DeferredCallKind::Function { function },
            args: values,
        });
        Ok(())
    }

    pub(super) fn execute_defer_method(
        &mut self,
        program: &Program,
        receiver: usize,
        method: String,
        args: Vec<usize>,
    ) -> Result<(), VmError> {
        let receiver_value = self.read_register(program, receiver)?;
        let resolved = self.resolve_method_call(program, receiver_value, &method)?;
        let mut values = Vec::with_capacity(args.len() + 1);
        let kind = match resolved {
            ResolvedMethodCall::Function { function, receiver } => {
                values.push(receiver);
                DeferredCallKind::Function { function }
            }
            ResolvedMethodCall::Stdlib { function, receiver } => {
                values.push(receiver);
                DeferredCallKind::Stdlib { function }
            }
        };
        for register in args {
            values.push(self.read_register(program, register)?);
        }
        self.current_frame_mut()
            .deferred
            .push(DeferredCall { kind, args: values });
        Ok(())
    }

    pub(super) fn execute_return(
        &mut self,
        program: &Program,
        src: Option<usize>,
    ) -> Result<(), VmError> {
        let results = match src {
            Some(src) => vec![self.read_register(program, src)?],
            None => Vec::new(),
        };
        self.current_frame_mut().unwind = Some(UnwindState::Return(results));
        self.continue_unwind(program)?;
        self.finish_current_goroutine_if_idle();
        Ok(())
    }

    pub(super) fn execute_return_multi(
        &mut self,
        program: &Program,
        srcs: Vec<usize>,
    ) -> Result<(), VmError> {
        let results = srcs
            .iter()
            .map(|register| self.read_register(program, *register))
            .collect::<Result<Vec<_>, _>>()?;
        self.current_frame_mut().unwind = Some(UnwindState::Return(results));
        self.continue_unwind(program)?;
        self.finish_current_goroutine_if_idle();
        Ok(())
    }

    pub(super) fn execute_panic(&mut self, program: &Program, src: usize) -> Result<(), VmError> {
        let value = self.read_register(program, src)?;
        self.propagate_panic_value(program, value)
    }

    pub(super) fn execute_recover(&mut self, program: &Program, dst: usize) -> Result<(), VmError> {
        let value = self.try_recover(program)?;
        self.set_register(program, dst, value)
    }

    pub(crate) fn propagate_panic_value(
        &mut self,
        program: &Program,
        value: Value,
    ) -> Result<(), VmError> {
        self.pending_panic_stack = Some(self.current_stack_debug_info(program));
        self.current_frame_mut().unwind = Some(UnwindState::Panic(value));
        self.continue_unwind(program)?;
        self.finish_current_goroutine_if_idle();
        Ok(())
    }

    fn continue_unwind(&mut self, program: &Program) -> Result<(), VmError> {
        loop {
            if self.current_goroutine().frames.is_empty() {
                return Ok(());
            }

            let deferred = {
                let frame = self.current_frame_mut();
                if frame.unwind.is_none() {
                    return Ok(());
                }
                frame.deferred.pop()
            };

            if let Some(deferred) = deferred {
                if self.execute_deferred_call(program, deferred)? {
                    return Ok(());
                }
                continue;
            }

            let finished = self
                .current_goroutine_mut()
                .frames
                .pop()
                .expect("frame should exist");
            let unwind = finished
                .unwind
                .expect("unwinding frame should have unwind state");
            if matches!(finished.return_target, ReturnTarget::Callback) {
                if matches!(&unwind, UnwindState::Return(_)) {
                    if let Some(arg_indices) = self.callback_capture_arg_indices.take() {
                        self.callback_captured_args = arg_indices
                            .iter()
                            .map(|arg_index| finished.registers.get(*arg_index).cloned())
                            .collect();
                    }
                } else {
                    self.callback_capture_arg_indices = None;
                    self.callback_captured_args = None;
                }
            }
            self.finish_unwound_frame(
                program,
                finished.function,
                finished.return_target,
                &finished.registers,
                unwind,
            )?;
            if self.current_goroutine().frames.is_empty() {
                return Ok(());
            }
        }
    }

    fn try_recover(&mut self, program: &Program) -> Result<Value, VmError> {
        let frame_count = self.current_goroutine().frames.len();
        if frame_count < 2 || !matches!(self.current_frame().return_target, ReturnTarget::Deferred)
        {
            return Ok(Value::nil());
        }

        let caller_index = frame_count - 2;
        let Some(UnwindState::Panic(value)) = self
            .current_goroutine()
            .frames
            .get(caller_index)
            .and_then(|frame| frame.unwind.clone())
        else {
            return Ok(Value::nil());
        };

        let caller = self
            .current_goroutine()
            .frames
            .get(caller_index)
            .expect("caller frame should exist");
        let recovered_unwind = if self.has_implicit_return_registers(program, caller.function) {
            UnwindState::Recovered
        } else {
            UnwindState::Return(self.default_results_for_target(&caller.return_target))
        };
        self.current_goroutine_mut().frames[caller_index].unwind = Some(recovered_unwind);
        self.pending_panic_stack = None;
        if matches!(value.data, ValueData::Nil) {
            Ok(value)
        } else {
            Ok(Value {
                typ: TYPE_ANY,
                data: value.data,
            })
        }
    }

    fn has_implicit_return_registers(&self, program: &Program, function: usize) -> bool {
        program
            .functions
            .get(function)
            .and_then(|function| function.code.last())
            .is_some_and(|instruction| {
                matches!(
                    instruction,
                    Instruction::Return { src: Some(_) } | Instruction::ReturnMulti { .. }
                )
            })
    }

    fn implicit_return_results(
        &self,
        program: &Program,
        function: usize,
        return_target: &ReturnTarget,
        registers: &[Value],
    ) -> Vec<Value> {
        let default_results = self.default_results_for_target(return_target);
        let Some(function) = program.functions.get(function) else {
            return default_results;
        };
        match function.code.last() {
            Some(Instruction::Return {
                src: Some(register),
            }) => registers
                .get(*register)
                .cloned()
                .map(|value| vec![value])
                .unwrap_or(default_results),
            Some(Instruction::ReturnMulti { srcs }) => {
                let mut results = Vec::with_capacity(srcs.len());
                for register in srcs {
                    let Some(value) = registers.get(*register).cloned() else {
                        return default_results;
                    };
                    results.push(value);
                }
                results
            }
            _ => default_results,
        }
    }

    fn execute_deferred_call(
        &mut self,
        program: &Program,
        deferred: DeferredCall,
    ) -> Result<bool, VmError> {
        match deferred.kind {
            DeferredCallKind::Stdlib { function } => {
                let _ = self.execute_stdlib(program, function, &deferred.args)?;
                Ok(false)
            }
            DeferredCallKind::Closure { function } => {
                let target = super::describe_value(&function);
                let ValueData::Function(function) = function.data else {
                    return Err(VmError::InvalidFunctionValue {
                        function: self.current_function_name(program)?,
                        target,
                    });
                };
                let mut values = function.captures;
                values.extend(deferred.args);
                self.push_frame_on_current_goroutine(
                    program,
                    function.function,
                    values,
                    ReturnTarget::Deferred,
                )?;
                Ok(true)
            }
            DeferredCallKind::Function { function } => {
                self.push_frame_on_current_goroutine(
                    program,
                    function,
                    deferred.args,
                    ReturnTarget::Deferred,
                )?;
                Ok(true)
            }
        }
    }

    fn resolve_method_call(
        &self,
        program: &Program,
        receiver_value: Value,
        method: &str,
    ) -> Result<ResolvedMethodCall, VmError> {
        let mut receiver_value = receiver_value;
        let receiver_type = self.method_dispatch_type(program, &receiver_value);
        if receiver_type != receiver_value.typ {
            receiver_value.typ = receiver_type;
        }
        if let Some(binding) = program
            .methods
            .iter()
            .find(|binding| binding.receiver_type == receiver_type && binding.name == method)
        {
            if !binding.promoted_fields.is_empty() {
                receiver_value = self.resolve_promoted_receiver(
                    program,
                    receiver_value,
                    &binding.promoted_fields,
                    binding.target_receiver_type,
                )?;
            }
            if receiver_value.typ != binding.target_receiver_type
                && matches!(
                    &receiver_value.data,
                    ValueData::Pointer(pointer) if !pointer.is_nil()
                )
            {
                let dereferenced = self.deref_pointer(program, &receiver_value)?;
                if dereferenced.typ == binding.target_receiver_type {
                    receiver_value = dereferenced;
                }
            }
            return Ok(ResolvedMethodCall::Function {
                function: binding.function,
                receiver: receiver_value,
            });
        }

        if let Some(function) = resolve_stdlib_runtime_method(receiver_type, method) {
            let receiver = self.adjust_stdlib_method_receiver(program, receiver_value, function)?;
            return Ok(ResolvedMethodCall::Stdlib { function, receiver });
        }

        Err(VmError::UnknownMethod {
            function: self.current_function_name(program)?,
            receiver_type: receiver_value.typ.0,
            method: method.into(),
        })
    }

    fn resolve_promoted_receiver(
        &self,
        program: &Program,
        receiver_value: Value,
        promoted_fields: &[PromotedFieldStep],
        target_receiver_type: TypeId,
    ) -> Result<Value, VmError> {
        let mut current = receiver_value;
        for (index, step) in promoted_fields.iter().enumerate() {
            let last = index + 1 == promoted_fields.len();
            match step.access {
                PromotedFieldAccess::Value => {
                    if matches!(&current.data, ValueData::Pointer(_)) {
                        current = self.deref_pointer(program, &current)?;
                    }
                    current = self.field_value(program, &current, &step.field)?;
                }
                PromotedFieldAccess::Pointer => {
                    let typ = if last {
                        target_receiver_type
                    } else {
                        TYPE_POINTER
                    };
                    current = self.project_field_pointer(program, &current, &step.field, typ)?;
                }
            }
        }
        Ok(current)
    }

    fn adjust_stdlib_method_receiver(
        &self,
        program: &Program,
        receiver_value: Value,
        function: StdlibFunctionId,
    ) -> Result<Value, VmError> {
        let expects_pointer = stdlib_function_param_types(function)
            .and_then(|params| params.first())
            .is_some_and(|receiver_type| receiver_type.starts_with('*'));
        if expects_pointer || !matches!(&receiver_value.data, ValueData::Pointer(_)) {
            return Ok(receiver_value);
        }
        self.deref_pointer(program, &receiver_value)
    }

    fn finish_unwound_frame(
        &mut self,
        program: &Program,
        function: usize,
        return_target: ReturnTarget,
        registers: &[Value],
        unwind: UnwindState,
    ) -> Result<(), VmError> {
        match unwind {
            UnwindState::Return(results) => match return_target {
                ReturnTarget::None => Ok(()),
                ReturnTarget::One(dst) => {
                    if results.len() != 1 {
                        return Err(VmError::ReturnValueCountMismatch {
                            function: function_name(program, function)?,
                            expected: 1,
                            actual: results.len(),
                        });
                    }
                    self.set_register_on_caller(program, dst, results[0].clone())
                }
                ReturnTarget::Many(dsts) => {
                    if results.len() != dsts.len() {
                        return Err(VmError::ReturnValueCountMismatch {
                            function: function_name(program, function)?,
                            expected: dsts.len(),
                            actual: results.len(),
                        });
                    }
                    for (dst, value) in dsts.into_iter().zip(results) {
                        self.set_register_on_caller(program, dst, value)?;
                    }
                    Ok(())
                }
                ReturnTarget::Deferred => self.continue_unwind(program),
                ReturnTarget::Callback => {
                    self.callback_result = Some(results);
                    Ok(())
                }
            },
            UnwindState::Recovered => {
                let results =
                    self.implicit_return_results(program, function, &return_target, registers);
                self.finish_unwound_frame(
                    program,
                    function,
                    return_target,
                    registers,
                    UnwindState::Return(results),
                )
            }
            UnwindState::Panic(value) => {
                if self.current_goroutine().frames.is_empty() {
                    return Err(VmError::UnhandledPanic {
                        function: function_name(program, function)?,
                        value: super::format_value(&value),
                    });
                }
                if matches!(return_target, ReturnTarget::Callback) {
                    self.callback_panic = Some(value);
                    return Ok(());
                }
                self.current_frame_mut().unwind = Some(UnwindState::Panic(value));
                Ok(())
            }
        }
    }
}
