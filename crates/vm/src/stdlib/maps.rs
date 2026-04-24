use super::{
    StdlibFunction, MAPS_CLONE, MAPS_COPY, MAPS_DELETE_FUNC, MAPS_EQUAL, MAPS_EQUAL_FUNC,
    MAPS_KEYS, MAPS_VALUES,
};
use crate::{FunctionValue, MapValue, Program, Value, ValueData, Vm, VmError, TYPE_ANY, TYPE_MAP};

pub(super) const MAPS_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: MAPS_KEYS,
        symbol: "Keys",
        returns_value: true,
        handler: maps_keys,
    },
    StdlibFunction {
        id: MAPS_VALUES,
        symbol: "Values",
        returns_value: true,
        handler: maps_values,
    },
    StdlibFunction {
        id: MAPS_EQUAL,
        symbol: "Equal",
        returns_value: true,
        handler: maps_equal,
    },
    StdlibFunction {
        id: MAPS_EQUAL_FUNC,
        symbol: "EqualFunc",
        returns_value: true,
        handler: maps_equal_func,
    },
    StdlibFunction {
        id: MAPS_CLONE,
        symbol: "Clone",
        returns_value: true,
        handler: maps_clone,
    },
    StdlibFunction {
        id: MAPS_COPY,
        symbol: "Copy",
        returns_value: true,
        handler: maps_copy,
    },
    StdlibFunction {
        id: MAPS_DELETE_FUNC,
        symbol: "DeleteFunc",
        returns_value: true,
        handler: maps_delete_func,
    },
];

fn maps_keys(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let entries = extract_map(vm, program, "maps.Keys", args, 1)?;
    let keys: Vec<Value> = entries.iter().map(|(k, _)| k.clone()).collect();
    Ok(Value::slice(keys))
}

fn maps_values(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let entries = extract_map(vm, program, "maps.Values", args, 1)?;
    let values: Vec<Value> = entries.iter().map(|(_, v)| v.clone()).collect();
    Ok(Value::slice(values))
}

fn maps_equal(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (a, b) = extract_two_map_values(vm, program, "maps.Equal", args)?;
    let a_entries = a.entries_snapshot();
    let b_entries = b.entries_snapshot();
    if a_entries.len() != b_entries.len() {
        return Ok(Value::bool(false));
    }
    for (key, val_a) in &a_entries {
        match b.get(key) {
            Some(val_b) if val_a == &val_b => {}
            _ => return Ok(Value::bool(false)),
        }
    }
    Ok(Value::bool(true))
}

fn maps_equal_func(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let a = extract_map_value(&args[0], vm, program, "maps.EqualFunc")?;
    let b = extract_map_value(&args[1], vm, program, "maps.EqualFunc")?;
    let ValueData::Function(eq_func) = &args[2].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[2]),
        });
    };
    let eq_func = eq_func.clone();

    let a_entries = a.entries_snapshot();
    let b_entries = b.entries_snapshot();
    if a_entries.len() != b_entries.len() {
        return Ok(Value::bool(false));
    }
    for (key, val_a) in &a_entries {
        match b.get(key) {
            Some(val_b) => {
                if !invoke_eq_callback(vm, program, &eq_func, val_a, &val_b)? {
                    return Ok(Value::bool(false));
                }
            }
            None => return Ok(Value::bool(false)),
        }
    }
    Ok(Value::bool(true))
}

fn maps_clone(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Map(map) = &args[0].data else {
        return Err(invalid_maps_argument(
            vm,
            program,
            "maps.Clone",
            "a map argument",
        )?);
    };
    if map.is_nil() {
        let mut result = args[0].clone();
        if result.typ == TYPE_ANY {
            result.typ = TYPE_MAP;
        }
        return Ok(result);
    }
    Ok(clone_map_value(&args[0], map))
}

fn maps_copy(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Map(dst_map) = &args[0].data else {
        return Err(invalid_maps_argument(
            vm,
            program,
            "maps.Copy",
            "a map argument",
        )?);
    };
    let ValueData::Map(src_map) = &args[1].data else {
        return Err(invalid_maps_argument(
            vm,
            program,
            "maps.Copy",
            "a map argument",
        )?);
    };
    if dst_map.is_nil() {
        if src_map.len() == 0 {
            return Ok(args[0].clone());
        }
        return Err(VmError::AssignToNilMap {
            function: vm
                .current_function_name(program)
                .unwrap_or_else(|_| "<engine>".into()),
            target: crate::describe_value(&args[0]),
        });
    }
    let src_entries = src_map.entries_snapshot();
    let result = args[0].clone();
    let ValueData::Map(result_map) = &result.data else {
        unreachable!("maps.Copy should keep a map receiver");
    };
    for (key, val) in src_entries {
        let inserted = result_map.insert(key, val);
        debug_assert!(inserted, "non-nil map copy target should stay writable");
    }
    Ok(result)
}

fn maps_delete_func(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Map(map) = &args[0].data else {
        return Err(invalid_maps_argument(
            vm,
            program,
            "maps.DeleteFunc",
            "a map argument",
        )?);
    };
    let ValueData::Function(del_func) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[1]),
        });
    };
    let del_func = del_func.clone();

    let result = args[0].clone();
    let entries = map.entries_snapshot();
    for (key, val) in &entries {
        if !invoke_kv_predicate(vm, program, &del_func, key, val)? {
            continue;
        }
        let removed = map.remove(key);
        debug_assert!(
            removed || map.is_nil(),
            "DeleteFunc should only skip removal for nil maps"
        );
    }
    Ok(result)
}

fn extract_map(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
    expected: usize,
) -> Result<Vec<(Value, Value)>, VmError> {
    if args.len() != expected {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected,
            actual: args.len(),
        });
    }
    extract_map_entries(&args[0], vm, program, builtin)
}

fn extract_map_entries(
    arg: &Value,
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
) -> Result<Vec<(Value, Value)>, VmError> {
    let map = extract_map_value(arg, vm, program, builtin)?;
    Ok(map.entries_snapshot())
}

fn clone_map_value(receiver: &Value, map: &MapValue) -> Value {
    let entries = map.entries_snapshot();
    let mut cloned = receiver.clone();
    cloned.data = ValueData::Map(if map.is_nil() {
        MapValue::nil((*map.zero_value).clone(), map.concrete_type.clone())
    } else {
        MapValue::with_entries(
            entries,
            (*map.zero_value).clone(),
            map.concrete_type.clone(),
        )
    });
    cloned
}

fn extract_map_value<'a>(
    arg: &'a Value,
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
) -> Result<&'a MapValue, VmError> {
    let ValueData::Map(map) = &arg.data else {
        return Err(invalid_maps_argument(
            vm,
            program,
            builtin,
            "a map argument",
        )?);
    };
    Ok(map)
}

fn extract_two_map_values<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a MapValue, &'a MapValue), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let a = extract_map_value(&args[0], vm, program, builtin)?;
    let b = extract_map_value(&args[1], vm, program, builtin)?;
    Ok((a, b))
}

fn invoke_eq_callback(
    vm: &mut Vm,
    program: &Program,
    eq_func: &FunctionValue,
    a: &Value,
    b: &Value,
) -> Result<bool, VmError> {
    let mut callback_args = eq_func.captures.clone();
    callback_args.push(a.clone());
    callback_args.push(b.clone());
    let result = vm.invoke_callback(program, eq_func.function, callback_args)?;
    match result.data {
        ValueData::Bool(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "eq callback".into(),
            expected: "bool return value".into(),
        }),
    }
}

fn invoke_kv_predicate(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    key: &Value,
    val: &Value,
) -> Result<bool, VmError> {
    let mut callback_args = predicate.captures.clone();
    callback_args.push(key.clone());
    callback_args.push(val.clone());
    let result = vm.invoke_callback(program, predicate.function, callback_args)?;
    match result.data {
        ValueData::Bool(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "delete predicate".into(),
            expected: "bool return value".into(),
        }),
    }
}

fn invalid_maps_argument(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
) -> Result<VmError, VmError> {
    Ok(VmError::InvalidStringFunctionArgument {
        function: vm.current_function_name(program)?,
        builtin: builtin.into(),
        expected: expected.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Function, Program};

    fn test_program() -> Program {
        Program {
            functions: vec![Function {
                name: "main".into(),
                param_count: 0,
                register_count: 0,
                code: vec![],
            }],
            methods: vec![],
            global_count: 0,
            entry_function: 0,
        }
    }

    #[test]
    fn maps_equal_handles_large_maps_with_different_insertion_orders() {
        let mut left_entries = Vec::new();
        let mut right_entries = Vec::new();
        for index in 0..1024 {
            let key = Value::string(format!("key-{index}"));
            let value = Value::int(index);
            left_entries.push((key.clone(), value.clone()));
            right_entries.insert(0, (key, value));
        }

        let mut vm = Vm::new();
        let program = test_program();
        let result = maps_equal(
            &mut vm,
            &program,
            &[
                Value::map(left_entries, Value::int(0)),
                Value::map(right_entries, Value::int(0)),
            ],
        )
        .expect("maps.Equal should succeed");
        assert_eq!(result, Value::bool(true));
    }

    #[test]
    fn maps_copy_updates_existing_entries_and_appends_new_ones_in_order() {
        let dst_entries: Vec<(Value, Value)> = (0..1024)
            .map(|index| (Value::int(index), Value::string(format!("dst-{index}"))))
            .collect();
        let src_entries: Vec<(Value, Value)> = (512..1536)
            .map(|index| (Value::int(index), Value::string(format!("src-{index}"))))
            .collect();

        let mut vm = Vm::new();
        let program = test_program();
        let result = maps_copy(
            &mut vm,
            &program,
            &[
                Value::map(dst_entries, Value::string("")),
                Value::map(src_entries, Value::string("")),
            ],
        )
        .expect("maps.Copy should succeed");

        let ValueData::Map(map) = result.data else {
            unreachable!("maps.Copy should return a map");
        };
        let entries = map.entries_snapshot();
        assert_eq!(entries.len(), 1536);
        assert_eq!(entries[0], (Value::int(0), Value::string("dst-0")));
        assert_eq!(entries[512], (Value::int(512), Value::string("src-512")));
        assert_eq!(entries[1023], (Value::int(1023), Value::string("src-1023")));
        assert_eq!(entries[1024], (Value::int(1024), Value::string("src-1024")));
        assert_eq!(entries[1535], (Value::int(1535), Value::string("src-1535")));
    }

    #[test]
    fn maps_clone_preserves_nil_maps() {
        let mut vm = Vm::new();
        let program = test_program();
        let result = maps_clone(&mut vm, &program, &[Value::nil_map(Value::int(0))])
            .expect("maps.Clone should accept nil maps");

        let ValueData::Map(map) = result.data else {
            unreachable!("maps.Clone should return a map");
        };
        assert!(map.is_nil());
    }

    #[test]
    fn maps_copy_rejects_nil_destination_when_source_is_non_empty() {
        let mut vm = Vm::new();
        let program = test_program();
        let error = maps_copy(
            &mut vm,
            &program,
            &[
                Value::nil_map(Value::int(0)),
                Value::map(vec![(Value::string("go"), Value::int(1))], Value::int(0)),
            ],
        )
        .expect_err("maps.Copy should reject nil destinations with writes");

        assert!(matches!(error, VmError::AssignToNilMap { .. }));
    }
}
