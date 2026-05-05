use super::*;

pub(super) fn reflect_value_of(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let inventory = reflect_inventory(vm, program)?;
    build_reflect_value_from_dynamic(&args[0], true, &inventory, vm, program)
}

pub(super) fn reflect_value_kind(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = extract_reflect_value_receiver(vm, program, args, "reflect.Value.Kind")?;
    if !reflect_value_valid(receiver)? {
        return Ok(reflect_kind_value(KIND_INVALID));
    }
    let type_value = reflect_value_type_required(vm, program, receiver, "Kind")?;
    Ok(reflect_kind_value(hidden_kind(&type_value)?))
}

pub(super) fn reflect_value_type(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "Type")?;
    reflect_value_type_required(vm, program, receiver, "Type")
}

pub(super) fn reflect_value_is_valid(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = extract_reflect_value_receiver(vm, program, args, "reflect.Value.IsValid")?;
    Ok(Value::bool(reflect_value_valid(receiver)?))
}

pub(super) fn reflect_value_can_interface_method(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "CanInterface")?;
    Ok(Value::bool(reflect_value_can_interface(receiver)?))
}

pub(super) fn reflect_value_is_nil(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "IsNil")?;
    let kind = hidden_kind(&reflect_value_type_required(
        vm, program, receiver, "IsNil",
    )?)?;
    let underlying = reflect_value_underlying(receiver)?;
    let is_nil = match (&underlying.data, kind) {
        (ValueData::Slice(slice), KIND_SLICE) => slice.is_nil,
        (ValueData::Map(map), KIND_MAP) => map.is_nil(),
        (ValueData::Channel(channel), KIND_CHAN) => channel.is_nil(),
        (ValueData::Pointer(pointer), KIND_PTR) => pointer.is_nil(),
        (ValueData::Nil, KIND_FUNC | KIND_INTERFACE) => true,
        (_, KIND_FUNC | KIND_INTERFACE) => false,
        _ => return Err(reflect_value_kind_panic(vm, program, receiver, "IsNil")),
    };
    Ok(Value::bool(is_nil))
}

pub(super) fn reflect_value_len(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "Len")?;
    let kind = hidden_kind(&reflect_value_type_required(vm, program, receiver, "Len")?)?;
    let underlying = reflect_value_underlying(receiver)?;
    let length = match &underlying.data {
        ValueData::Array(array) if kind == KIND_ARRAY => array.len() as i64,
        ValueData::Slice(slice) if kind == KIND_SLICE => slice.len() as i64,
        ValueData::String(text) if kind == KIND_STRING => text.len() as i64,
        ValueData::Map(map) if kind == KIND_MAP => map.len() as i64,
        ValueData::Channel(channel) if kind == KIND_CHAN => match channel.id {
            Some(id) => vm
                .channels
                .get(id as usize)
                .map(|state| state.buffer.len() as i64)
                .ok_or_else(|| reflect_panic(vm, program, "Len of unknown channel"))?,
            None => 0,
        },
        _ => return Err(reflect_value_kind_panic(vm, program, receiver, "Len")),
    };
    Ok(Value::int(length))
}

pub(super) fn reflect_value_num_field(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "NumField")?;
    let type_value = reflect_value_type_required(vm, program, receiver, "NumField")?;
    if hidden_kind(&type_value)? != KIND_STRUCT {
        return Err(reflect_value_kind_panic(vm, program, receiver, "NumField"));
    }
    Ok(Value::int(
        hidden_slice(&type_value, TYPE_FIELDS_FIELD)?.len() as i64,
    ))
}

pub(super) fn reflect_value_bool(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = readable_reflect_value_receiver(vm, program, args, "Bool")?;
    if hidden_kind(&reflect_value_type_required(vm, program, receiver, "Bool")?)? != KIND_BOOL {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Bool"));
    }
    let underlying = reflect_value_underlying(receiver)?;
    let ValueData::Bool(value) = &underlying.data else {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Bool"));
    };
    Ok(Value::bool(*value))
}

pub(super) fn reflect_value_int(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = readable_reflect_value_receiver(vm, program, args, "Int")?;
    if hidden_kind(&reflect_value_type_required(vm, program, receiver, "Int")?)? != KIND_INT {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Int"));
    }
    let underlying = reflect_value_underlying(receiver)?;
    let ValueData::Int(value) = &underlying.data else {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Int"));
    };
    Ok(Value::int(*value))
}

pub(super) fn reflect_value_float(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = readable_reflect_value_receiver(vm, program, args, "Float")?;
    if hidden_kind(&reflect_value_type_required(
        vm, program, receiver, "Float",
    )?)? != KIND_FLOAT64
    {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Float"));
    }
    let underlying = reflect_value_underlying(receiver)?;
    let ValueData::Float(value) = &underlying.data else {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Float"));
    };
    Ok(Value::float(value.0))
}

pub(super) fn reflect_value_string(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = extract_reflect_value_receiver(vm, program, args, "reflect.Value.String")?;
    if !reflect_value_valid(receiver)? {
        return Ok(Value::string("<invalid Value>"));
    }
    let type_value = reflect_value_type_required(vm, program, receiver, "String")?;
    if hidden_kind(&type_value)? == KIND_STRING {
        if !reflect_value_can_interface(receiver)? {
            return Err(reflect_panic(vm, program, "cannot String unexported value"));
        }
        let underlying = reflect_value_underlying(receiver)?;
        let ValueData::String(value) = underlying.data else {
            return Err(reflect_value_kind_panic(vm, program, receiver, "String"));
        };
        return Ok(Value::string(value));
    }
    Ok(Value::string(format!(
        "<{} Value>",
        hidden_string(&type_value, TYPE_STRING_FIELD)?
    )))
}

pub(super) fn reflect_value_elem(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "Elem")?;
    let type_value = reflect_value_type_required(vm, program, receiver, "Elem")?;
    let kind = hidden_kind(&type_value)?;
    let can_interface = reflect_value_can_interface(receiver)?;
    let underlying = reflect_value_underlying(receiver)?;
    match kind {
        KIND_PTR => {
            let ValueData::Pointer(pointer) = &underlying.data else {
                return Err(reflect_value_kind_panic(vm, program, receiver, "Elem"));
            };
            if pointer.is_nil() {
                return Ok(build_invalid_reflect_value());
            }
            let elem_type = reflect_type_member(vm, program, &type_value, TYPE_ELEM_FIELD, "Elem")?;
            let element = vm.deref_pointer(program, &underlying)?;
            Ok(build_reflect_value_with_type_value(
                element,
                elem_type,
                can_interface,
            ))
        }
        KIND_INTERFACE => {
            if matches!(&underlying.data, ValueData::Nil) {
                return Ok(build_invalid_reflect_value());
            }
            let inventory = reflect_inventory(vm, program)?;
            build_reflect_value_from_dynamic(&underlying, can_interface, &inventory, vm, program)
        }
        _ => Err(reflect_value_kind_panic(vm, program, receiver, "Elem")),
    }
}

pub(super) fn reflect_value_index(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let receiver =
        valid_reflect_value_value(vm, program, &args[0], "reflect.Value.Index", "Index")?;
    let index = extract_index_arg(vm, program, args, "Index")?;
    let type_value = reflect_value_type_required(vm, program, receiver, "Index")?;
    let can_interface = reflect_value_can_interface(receiver)?;
    let underlying = reflect_value_underlying(receiver)?;
    match (&underlying.data, hidden_kind(&type_value)?) {
        (ValueData::Array(array), KIND_ARRAY) => {
            let Some(value) = array.get(index as usize) else {
                return Err(reflect_panic(vm, program, "Index out of range"));
            };
            let elem_type =
                reflect_type_member(vm, program, &type_value, TYPE_ELEM_FIELD, "Index")?;
            Ok(build_reflect_value_with_type_value(
                value,
                elem_type,
                can_interface,
            ))
        }
        (ValueData::Slice(slice), KIND_SLICE) => {
            let Some(value) = slice.get(index as usize) else {
                return Err(reflect_panic(vm, program, "Index out of range"));
            };
            let elem_type =
                reflect_type_member(vm, program, &type_value, TYPE_ELEM_FIELD, "Index")?;
            Ok(build_reflect_value_with_type_value(
                value,
                elem_type,
                can_interface,
            ))
        }
        (ValueData::String(text), KIND_STRING) => {
            let Some(byte) = text.as_bytes().get(index as usize) else {
                return Err(reflect_panic(vm, program, "Index out of range"));
            };
            let inventory = reflect_inventory(vm, program)?;
            let elem_type = type_value_from_type_id(TYPE_INT, &inventory, vm, program)?;
            Ok(build_reflect_value_with_type_value(
                Value::int(*byte as i64),
                elem_type,
                can_interface,
            ))
        }
        _ => Err(reflect_value_kind_panic(vm, program, receiver, "Index")),
    }
}

pub(super) fn reflect_value_field(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let receiver =
        valid_reflect_value_value(vm, program, &args[0], "reflect.Value.Field", "Field")?;
    let index = extract_index_arg(vm, program, args, "Field")?;
    let type_value = reflect_value_type_required(vm, program, receiver, "Field")?;
    if hidden_kind(&type_value)? != KIND_STRUCT {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Field"));
    }
    let underlying = reflect_value_underlying(receiver)?;
    let ValueData::Struct(fields) = &underlying.data else {
        return Err(reflect_value_kind_panic(vm, program, receiver, "Field"));
    };
    let field_descriptors = hidden_slice(&type_value, TYPE_FIELDS_FIELD)?;
    let Some((_, value)) = fields.get(index as usize) else {
        return Err(reflect_panic(vm, program, "Field index out of range"));
    };
    let Some(field_desc) = field_descriptors.get(index as usize) else {
        return Err(reflect_panic(vm, program, "Field index out of range"));
    };
    let field_type = reflect_struct_field_type(vm, program, field_desc)?;
    let can_interface = reflect_value_can_interface(receiver)?
        && hidden_string(field_desc, TYPE_REF_FIELD_PKG_PATH)?.is_empty();
    Ok(build_reflect_value_with_type_value(
        value.clone(),
        field_type,
        can_interface,
    ))
}

pub(super) fn reflect_value_map_index(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let receiver =
        valid_reflect_value_value(vm, program, &args[0], "reflect.Value.MapIndex", "MapIndex")?;
    let type_value = reflect_value_type_required(vm, program, receiver, "MapIndex")?;
    if hidden_kind(&type_value)? != KIND_MAP {
        return Err(reflect_value_kind_panic(vm, program, receiver, "MapIndex"));
    }
    let underlying = reflect_value_underlying(receiver)?;
    let ValueData::Map(map) = &underlying.data else {
        return Err(reflect_value_kind_panic(vm, program, receiver, "MapIndex"));
    };
    let Some(value) = map.get(&args[1]) else {
        return Ok(build_invalid_reflect_value());
    };
    let elem_type = reflect_type_member(vm, program, &type_value, TYPE_ELEM_FIELD, "MapIndex")?;
    Ok(build_reflect_value_with_type_value(
        value,
        elem_type,
        reflect_value_can_interface(receiver)?,
    ))
}

pub(super) fn reflect_value_map_keys(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, "MapKeys")?;
    let type_value = reflect_value_type_required(vm, program, receiver, "MapKeys")?;
    if hidden_kind(&type_value)? != KIND_MAP {
        return Err(reflect_value_kind_panic(vm, program, receiver, "MapKeys"));
    }
    let underlying = reflect_value_underlying(receiver)?;
    let ValueData::Map(map) = &underlying.data else {
        return Err(reflect_value_kind_panic(vm, program, receiver, "MapKeys"));
    };
    let key_type = reflect_type_member(vm, program, &type_value, TYPE_KEY_FIELD, "MapKeys")?;
    let can_interface = reflect_value_can_interface(receiver)?;
    let keys = map
        .entries_snapshot()
        .into_iter()
        .map(|(key, _)| build_reflect_value_with_type_value(key, key_type.clone(), can_interface))
        .collect::<Vec<_>>();
    Ok(Value::slice_typed(
        keys,
        ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_REFLECT_VALUE)),
        },
    ))
}

pub(super) fn reflect_value_interface(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = readable_reflect_value_receiver(vm, program, args, "Interface")?;
    reflect_value_underlying(receiver)
}

fn valid_reflect_value_receiver<'a>(
    vm: &Vm,
    program: &Program,
    args: &'a [Value],
    method: &str,
) -> Result<&'a Value, VmError> {
    let receiver =
        extract_reflect_value_receiver(vm, program, args, &format!("reflect.Value.{method}"))?;
    if !reflect_value_valid(receiver)? {
        return Err(reflect_invalid_value_panic(vm, program, method));
    }
    Ok(receiver)
}

fn valid_reflect_value_value<'a>(
    vm: &Vm,
    program: &Program,
    value: &'a Value,
    function_name: &str,
    method: &str,
) -> Result<&'a Value, VmError> {
    let receiver = extract_reflect_value_value(vm, program, value, function_name)?;
    if !reflect_value_valid(receiver)? {
        return Err(reflect_invalid_value_panic(vm, program, method));
    }
    Ok(receiver)
}

fn readable_reflect_value_receiver<'a>(
    vm: &Vm,
    program: &Program,
    args: &'a [Value],
    method: &str,
) -> Result<&'a Value, VmError> {
    let receiver = valid_reflect_value_receiver(vm, program, args, method)?;
    if !reflect_value_can_interface(receiver)? {
        return Err(reflect_panic(
            vm,
            program,
            &format!("cannot {method} unexported value"),
        ));
    }
    Ok(receiver)
}

fn extract_index_arg(
    vm: &Vm,
    program: &Program,
    args: &[Value],
    method: &str,
) -> Result<i64, VmError> {
    let ValueData::Int(index) = &args[1].data else {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} index must be an int"),
        ));
    };
    if *index < 0 {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} index out of range"),
        ));
    }
    Ok(*index)
}

fn reflect_value_kind_panic(vm: &Vm, program: &Program, receiver: &Value, method: &str) -> VmError {
    let label = reflect_value_type_label(receiver).unwrap_or_else(|_| "unknown".into());
    reflect_panic(
        vm,
        program,
        &format!("call of reflect.Value.{method} on {label} Value"),
    )
}

fn reflect_value_type_label(receiver: &Value) -> Result<String, VmError> {
    let type_value = hidden_field(receiver, VALUE_TYPE_FIELD)?;
    if matches!(&type_value.data, ValueData::Nil) {
        return Ok("invalid".into());
    }
    hidden_string(&type_value, TYPE_STRING_FIELD)
}
