use super::{
    program_type_inventory, ConcreteType, Instruction, Program, RuntimeTypeKind, Value, ValueData,
    Vm, VmError, TYPE_ANY, TYPE_ERROR,
};

impl Vm {
    pub(super) fn execute_instruction(
        &mut self,
        program: &Program,
        instruction: Instruction,
    ) -> Result<(), VmError> {
        match instruction {
            Instruction::LoadInt { dst, value } => {
                self.set_register(program, dst, Value::int(value))?
            }
            Instruction::LoadFloat { dst, value } => {
                self.set_register(program, dst, Value::float(value.0))?
            }
            Instruction::LoadBool { dst, value } => {
                self.set_register(program, dst, Value::bool(value))?
            }
            Instruction::LoadString { dst, value } => {
                self.set_register(program, dst, Value::string(value))?
            }
            Instruction::LoadNil { dst } => self.set_register(program, dst, Value::nil())?,
            Instruction::LoadNilChannel { dst, concrete_type } => {
                let value = concrete_type
                    .map(Value::nil_channel_typed)
                    .unwrap_or_else(Value::nil_channel);
                self.set_register(program, dst, value)?
            }
            Instruction::LoadNilPointer {
                dst,
                typ,
                concrete_type,
            } => {
                let value = concrete_type
                    .map(|concrete_type| Value::nil_pointer_typed(typ, concrete_type))
                    .unwrap_or_else(|| Value::nil_pointer(typ));
                self.set_register(program, dst, value)?
            }
            Instruction::BoxHeap { dst, src, typ } => {
                let value = self.read_register(program, src)?;
                let boxed = self.box_heap_value(value, typ);
                self.set_register(program, dst, boxed)?
            }
            Instruction::AddressLocal { dst, src, typ } => {
                self.set_register(program, dst, self.pointer_to_local(src, typ))?
            }
            Instruction::AddressGlobal { dst, global, typ } => {
                self.set_register(program, dst, self.pointer_to_global(global, typ))?
            }
            Instruction::ProjectFieldPointer {
                dst,
                src,
                field,
                typ,
            } => {
                let pointer = self.read_register(program, src)?;
                let pointer = self.project_field_pointer(program, &pointer, &field, typ)?;
                self.set_register(program, dst, pointer)?
            }
            Instruction::ProjectIndexPointer {
                dst,
                src,
                index,
                typ,
            } => {
                let pointer = self.read_register(program, src)?;
                let index = self.read_register(program, index)?;
                let pointer = self.project_index_pointer(program, &pointer, index, typ)?;
                self.set_register(program, dst, pointer)?
            }
            Instruction::AddressLocalField {
                dst,
                src,
                field,
                typ,
            } => self.set_register(program, dst, self.pointer_to_local_field(src, &field, typ))?,
            Instruction::AddressGlobalField {
                dst,
                global,
                field,
                typ,
            } => self.set_register(
                program,
                dst,
                self.pointer_to_global_field(global, &field, typ),
            )?,
            Instruction::AddressLocalIndex {
                dst,
                src,
                index,
                typ,
            } => {
                let index = self.read_register(program, index)?;
                self.set_register(program, dst, self.pointer_to_local_index(src, index, typ))?
            }
            Instruction::AddressGlobalIndex {
                dst,
                global,
                index,
                typ,
            } => {
                let index = self.read_register(program, index)?;
                self.set_register(
                    program,
                    dst,
                    self.pointer_to_global_index(global, index, typ),
                )?
            }
            Instruction::LoadNilSlice { dst, concrete_type } => {
                let value = concrete_type
                    .map(Value::nil_slice_typed)
                    .unwrap_or_else(Value::nil_slice);
                self.set_register(program, dst, value)?
            }
            Instruction::LoadErrorMessage { dst, src } => {
                let value = self.read_register(program, src)?;
                let ValueData::Error(err) = value.data else {
                    return Err(VmError::InvalidErrorValue {
                        function: self.current_function_name(program)?,
                    });
                };
                self.set_register(program, dst, Value::string(err.message))?
            }
            Instruction::LoadGlobal { dst, global } => {
                let value = self.read_global(global)?;
                self.set_register(program, dst, value)?
            }
            Instruction::StoreGlobal { global, src } => {
                let value = self.read_register(program, src)?;
                self.set_global(global, value)?
            }
            Instruction::MakeArray {
                dst,
                concrete_type,
                items,
            } => {
                let values = self.read_register_list(program, &items)?;
                let value = match concrete_type {
                    Some(concrete_type) => Value::array_typed(values, concrete_type),
                    None => Value::array(values),
                };
                self.set_register(program, dst, value)?
            }
            Instruction::MakeSlice {
                dst,
                concrete_type,
                items,
            } => {
                let values = self.read_register_list(program, &items)?;
                let value = match concrete_type {
                    Some(concrete_type) => Value::slice_typed(values, concrete_type),
                    None => Value::slice(values),
                };
                self.set_register(program, dst, value)?
            }
            Instruction::MakeChannel {
                dst,
                concrete_type,
                cap,
                zero,
            } => {
                let capacity = match cap {
                    Some(cap) => {
                        let cap = self.read_register(program, cap)?;
                        let ValueData::Int(cap) = cap.data else {
                            return Err(VmError::InvalidMakeCapacity {
                                function: self.current_function_name(program)?,
                            });
                        };
                        if cap < 0 {
                            return Err(VmError::NegativeMakeCapacity {
                                function: self.current_function_name(program)?,
                                cap,
                            });
                        }
                        cap as usize
                    }
                    None => 0,
                };
                let zero_value = self.read_register(program, zero)?;
                let channel =
                    self.alloc_channel_value_with_type(capacity, zero_value, concrete_type);
                self.set_register(program, dst, channel)?
            }
            Instruction::MakeMap {
                dst,
                concrete_type,
                entries,
                zero,
            } => {
                let value = self.map_value(program, &entries, zero, concrete_type.as_ref())?;
                self.set_register(program, dst, value)?
            }
            Instruction::MakeNilMap {
                dst,
                concrete_type,
                zero,
            } => {
                let zero_value = self.read_register(program, zero)?;
                let value = match concrete_type {
                    Some(concrete_type) => Value::nil_map_typed(zero_value, concrete_type),
                    None => Value::nil_map(zero_value),
                };
                self.set_register(program, dst, value)?
            }
            Instruction::MakeStruct { dst, typ, fields } => {
                let value = self.struct_value(program, typ, &fields)?;
                self.set_register(program, dst, value)?
            }
            Instruction::Index { dst, target, index } => {
                let target = self.read_register(program, target)?;
                let index = self.read_register(program, index)?;
                let value = self.index_value(program, &target, &index)?;
                self.set_register(program, dst, value)?
            }
            Instruction::Slice {
                dst,
                target,
                low,
                high,
            } => {
                let target = self.read_register(program, target)?;
                let low = low.map(|r| self.read_register(program, r)).transpose()?;
                let high = high.map(|r| self.read_register(program, r)).transpose()?;
                let value = self.slice_value(program, &target, low.as_ref(), high.as_ref())?;
                self.set_register(program, dst, value)?
            }
            Instruction::MapContains { dst, target, index } => {
                let target = self.read_register(program, target)?;
                let index = self.read_register(program, index)?;
                let value = self.map_contains(program, &target, &index)?;
                self.set_register(program, dst, Value::bool(value))?
            }
            Instruction::GetField { dst, target, field } => {
                let target = self.read_register(program, target)?;
                let value = self.field_value(program, &target, &field)?;
                self.set_register(program, dst, value)?
            }
            Instruction::AssertType { dst, src, target } => {
                let value = self.read_register(program, src)?;
                let value = self.assert_type(program, &value, &target)?;
                self.set_register(program, dst, value)?
            }
            Instruction::TypeMatches { dst, src, target } => {
                let value = self.read_register(program, src)?;
                let matches = self.value_matches_type(program, &value, &target);
                self.set_register(program, dst, Value::bool(matches))?
            }
            Instruction::IsNil { dst, src } => {
                let value = self.read_register(program, src)?;
                let is_nil = if value.typ == TYPE_ANY
                    || value.typ == TYPE_ERROR
                    || program_type_inventory(program)
                        .and_then(|inventory| inventory.type_info_for_type_id(value.typ))
                        .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
                {
                    matches!(value.data, ValueData::Nil)
                } else {
                    match &value.data {
                        ValueData::Nil => true,
                        ValueData::Slice(slice) => slice.is_nil,
                        ValueData::Map(map) => map.entries.is_none(),
                        ValueData::Channel(ch) => ch.is_nil(),
                        ValueData::Pointer(ptr) => ptr.is_nil(),
                        _ => false,
                    }
                };
                self.set_register(program, dst, Value::bool(is_nil))?
            }
            Instruction::SetField { target, field, src } => {
                let value = self.read_register(program, src)?;
                self.set_field_on_register(program, target, &field, value)?
            }
            Instruction::SetIndex { target, index, src } => {
                let index = self.read_register(program, index)?;
                let value = self.read_register(program, src)?;
                self.set_index_on_register(program, target, &index, value)?
            }
            Instruction::StoreIndirect { target, src } => {
                let pointer = self.read_register(program, target)?;
                let value = self.read_register(program, src)?;
                self.store_indirect(program, &pointer, value)?
            }
            Instruction::Copy {
                target,
                src,
                count_dst,
            } => {
                let source = self.read_register(program, src)?;
                let count = self.copy_into_register(program, target, &source)?;
                if let Some(count_dst) = count_dst {
                    self.set_register(program, count_dst, Value::int(count as i64))?;
                }
            }
            Instruction::Move { dst, src } => {
                let value = self.read_register(program, src)?;
                self.set_register(program, dst, value)?
            }
            Instruction::Deref { dst, src } => {
                let pointer = self.read_register(program, src)?;
                let value = self.deref_pointer(program, &pointer)?;
                self.set_register(program, dst, value)?
            }
            Instruction::Not { dst, src } => {
                let value = self.read_register(program, src)?;
                let result = self.not_value(program, &value)?;
                self.set_register(program, dst, result)?
            }
            Instruction::Negate { dst, src } => {
                let value = self.read_register(program, src)?;
                let result = self.negate_value(program, &value)?;
                self.set_register(program, dst, result)?
            }
            Instruction::BitNot { dst, src } => {
                let value = self.read_register(program, src)?;
                let result = self.bit_not_value(program, &value)?;
                self.set_register(program, dst, result)?
            }
            Instruction::Add { dst, left, right } => {
                let left = self.read_register(program, left)?;
                let right = self.read_register(program, right)?;
                let result = self.add_values(program, &left, &right)?;
                self.set_register(program, dst, result)?
            }
            Instruction::Subtract { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "-")?
            }
            Instruction::BitXor { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "^")?
            }
            Instruction::BitAnd { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "&")?
            }
            Instruction::BitClear { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "&^")?
            }
            Instruction::BitOr { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "|")?
            }
            Instruction::Multiply { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "*")?
            }
            Instruction::Divide { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "/")?
            }
            Instruction::Modulo { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "%")?
            }
            Instruction::ShiftLeft { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, "<<")?
            }
            Instruction::ShiftRight { dst, left, right } => {
                self.execute_int_instruction(program, dst, left, right, ">>")?
            }
            Instruction::Compare {
                dst,
                op,
                left,
                right,
            } => {
                let left = self.read_register(program, left)?;
                let right = self.read_register(program, right)?;
                let result = self.compare_values(program, op, &left, &right)?;
                self.set_register(program, dst, result)?
            }
            Instruction::Jump { target } => self.set_current_pc(target),
            Instruction::JumpIfFalse { cond, target } => {
                let condition = self.read_register(program, cond)?;
                match condition.data {
                    ValueData::Bool(value) => {
                        if !value {
                            self.set_current_pc(target);
                        }
                    }
                    _ => {
                        return Err(VmError::InvalidConditionValue {
                            function: self.current_function_name(program)?,
                        });
                    }
                }
            }
            Instruction::Select {
                choice_dst,
                cases,
                default_case,
            } => self.execute_select(program, choice_dst, &cases, default_case)?,
            Instruction::GoCall { function, args } => {
                let values = args
                    .iter()
                    .map(|register| self.read_register(program, *register))
                    .collect::<Result<Vec<_>, _>>()?;
                self.spawn_goroutine(program, function, values)?;
            }
            Instruction::GoCallClosure { callee, args } => {
                self.execute_go_call_closure(program, callee, args)?
            }
            Instruction::GoCallMethod {
                receiver,
                method,
                args,
            } => self.execute_go_call_method(program, receiver, method, args)?,
            Instruction::GoCallStdlib { function, args } => {
                let values = args
                    .iter()
                    .map(|register| self.read_register(program, *register))
                    .collect::<Result<Vec<_>, _>>()?;
                self.execute_stdlib(program, function, &values)?;
            }
            Instruction::ChanSend { chan, value } => {
                self.execute_chan_send(program, chan, value)?
            }
            Instruction::ChanRecv { dst, chan } => self.execute_chan_recv(program, dst, chan)?,
            Instruction::ChanRecvOk {
                value_dst,
                ok_dst,
                chan,
            } => self.execute_chan_recv_ok(program, value_dst, ok_dst, chan)?,
            Instruction::ChanTryRecv {
                ready_dst,
                value_dst,
                chan,
            } => self.execute_chan_try_recv(program, ready_dst, value_dst, chan)?,
            Instruction::ChanTryRecvOk {
                ready_dst,
                value_dst,
                ok_dst,
                chan,
            } => self.execute_chan_try_recv_ok(program, ready_dst, value_dst, ok_dst, chan)?,
            Instruction::ChanTrySend {
                ready_dst,
                chan,
                value,
            } => self.execute_chan_try_send(program, ready_dst, chan, value)?,
            Instruction::CloseChannel { chan } => self.execute_close_channel(program, chan)?,
            Instruction::CallStdlib {
                function,
                args,
                dst,
            } => {
                let values = args
                    .iter()
                    .map(|register| self.read_register(program, *register))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.execute_stdlib(program, function, &values)?;
                if let Some(dst) = dst {
                    self.set_register(program, dst, result)?;
                }
            }
            Instruction::CallStdlibMulti {
                function,
                args,
                dsts,
            } => self.execute_call_stdlib_multi(program, function, args, dsts)?,
            Instruction::DeferStdlib { function, args } => {
                self.execute_defer_stdlib(program, function, args)?
            }
            Instruction::CallFunction {
                function,
                args,
                dst,
            } => self.execute_call_function(program, function, args, dst)?,
            Instruction::MakeClosure {
                dst,
                concrete_type,
                function,
                captures,
            } => {
                let captures = self.read_register_list(program, &captures)?;
                let value = match concrete_type {
                    Some(concrete_type) => Value::function_typed(function, captures, concrete_type),
                    None => Value::function(function, captures),
                };
                self.set_register(program, dst, value)?
            }
            Instruction::CallClosure { callee, args, dst } => {
                self.execute_call_closure(program, callee, args, dst)?
            }
            Instruction::DeferClosure { callee, args } => {
                self.execute_defer_closure(program, callee, args)?
            }
            Instruction::DeferFunction { function, args } => {
                self.execute_defer_function(program, function, args)?
            }
            Instruction::CallFunctionMulti {
                function,
                args,
                dsts,
            } => self.execute_call_function_multi(program, function, args, dsts)?,
            Instruction::CallClosureMulti { callee, args, dsts } => {
                self.execute_call_closure_multi(program, callee, args, dsts)?
            }
            Instruction::CallMethod {
                receiver,
                method,
                args,
                dst,
            } => self.execute_call_method(program, receiver, method, args, dst)?,
            Instruction::DeferMethod {
                receiver,
                method,
                args,
            } => self.execute_defer_method(program, receiver, method, args)?,
            Instruction::CallMethodMulti {
                receiver,
                method,
                args,
                dsts,
            } => self.execute_call_method_multi(program, receiver, method, args, dsts)?,
            Instruction::CallMethodMultiMutatingArg {
                receiver,
                method,
                args,
                dsts,
                mutated_arg,
            } => self.execute_call_method_multi_mutating_arg(
                program,
                receiver,
                method,
                args,
                dsts,
                mutated_arg,
            )?,
            Instruction::Return { src } => self.execute_return(program, src)?,
            Instruction::ReturnMulti { srcs } => self.execute_return_multi(program, srcs)?,
            Instruction::Panic { src } => self.execute_panic(program, src)?,
            Instruction::Recover { dst } => self.execute_recover(program, dst)?,
            Instruction::ConvertToInt { dst, src } => {
                self.execute_convert_to_int(program, dst, src)?
            }
            Instruction::ConvertToFloat64 { dst, src } => {
                self.execute_convert_to_float64(program, dst, src)?
            }
            Instruction::ConvertToString { dst, src } => {
                self.execute_convert_to_string(program, dst, src)?
            }
            Instruction::ConvertToByte { dst, src } => {
                self.execute_convert_to_byte(program, dst, src)?
            }
            Instruction::ConvertToByteSlice { dst, src } => {
                self.execute_convert_to_byte_slice(program, dst, src)?
            }
            Instruction::ConvertToRuneSlice { dst, src } => {
                self.execute_convert_to_rune_slice(program, dst, src)?
            }
            Instruction::ConvertRuneSliceToString { dst, src } => {
                self.execute_convert_rune_slice_to_string(program, dst, src)?
            }
            Instruction::Retag { dst, src, typ } => {
                let mut value = self.read_register(program, src)?;
                retag_value(program, &mut value, typ);
                self.set_register(program, dst, value)?
            }
        }
        Ok(())
    }
}

fn retag_value(program: &Program, value: &mut Value, typ: super::TypeId) {
    value.typ = typ;
    let Some(inventory) = program_type_inventory(program) else {
        return;
    };
    let Some(info) = inventory.type_info_for_type_id(typ) else {
        return;
    };
    if matches!(info.kind, RuntimeTypeKind::Interface) {
        return;
    }
    let concrete_type = ConcreteType::TypeId(typ);
    match &mut value.data {
        ValueData::Array(array) if matches!(info.kind, RuntimeTypeKind::Array) => {
            array.concrete_type = Some(concrete_type);
        }
        ValueData::Slice(slice) if matches!(info.kind, RuntimeTypeKind::Slice) => {
            slice.concrete_type = Some(concrete_type);
        }
        ValueData::Map(map) if matches!(info.kind, RuntimeTypeKind::Map) => {
            map.concrete_type = Some(concrete_type);
        }
        ValueData::Pointer(pointer) if matches!(info.kind, RuntimeTypeKind::Pointer) => {
            pointer.concrete_type = Some(concrete_type);
        }
        ValueData::Function(function) if matches!(info.kind, RuntimeTypeKind::Function) => {
            function.concrete_type = Some(concrete_type);
        }
        ValueData::Channel(channel) if matches!(info.kind, RuntimeTypeKind::Channel) => {
            channel.concrete_type = Some(concrete_type);
        }
        _ => {}
    }
}
