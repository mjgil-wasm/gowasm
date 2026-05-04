use super::{
    StdlibFunction, SLICES_COMPACT, SLICES_COMPACT_FUNC, SLICES_CONTAINS, SLICES_CONTAINS_FUNC,
    SLICES_EQUAL, SLICES_INDEX, SLICES_INDEX_FUNC, SLICES_REVERSE, SLICES_SORT_FUNC,
    SLICES_SORT_STABLE_FUNC,
};
use crate::{FunctionValue, Program, Value, ValueData, Vm, VmError, TYPE_ANY, TYPE_SLICE};

pub(super) const SLICES_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: SLICES_CONTAINS,
        symbol: "Contains",
        returns_value: true,
        handler: slices_contains,
    },
    StdlibFunction {
        id: SLICES_CONTAINS_FUNC,
        symbol: "ContainsFunc",
        returns_value: true,
        handler: slices_contains_func,
    },
    StdlibFunction {
        id: SLICES_INDEX,
        symbol: "Index",
        returns_value: true,
        handler: slices_index,
    },
    StdlibFunction {
        id: SLICES_INDEX_FUNC,
        symbol: "IndexFunc",
        returns_value: true,
        handler: slices_index_func,
    },
    StdlibFunction {
        id: SLICES_SORT_FUNC,
        symbol: "SortFunc",
        returns_value: true,
        handler: slices_sort_func,
    },
    StdlibFunction {
        id: SLICES_SORT_STABLE_FUNC,
        symbol: "SortStableFunc",
        returns_value: true,
        handler: slices_sort_stable_func,
    },
    StdlibFunction {
        id: SLICES_COMPACT,
        symbol: "Compact",
        returns_value: true,
        handler: slices_compact,
    },
    StdlibFunction {
        id: SLICES_COMPACT_FUNC,
        symbol: "CompactFunc",
        returns_value: true,
        handler: slices_compact_func,
    },
    StdlibFunction {
        id: SLICES_REVERSE,
        symbol: "Reverse",
        returns_value: true,
        handler: slices_reverse,
    },
    StdlibFunction {
        id: SLICES_EQUAL,
        symbol: "Equal",
        returns_value: true,
        handler: slices_equal,
    },
];

fn slices_contains(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _) = extract_slice(vm, program, "slices.Contains", args, 2)?;
    let target = &args[1];
    Ok(Value::bool(values.iter().any(|v| v == target)))
}

fn slices_contains_func(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, predicate) = extract_slice_and_func(vm, program, "slices.ContainsFunc", args)?;
    for value in &values {
        if invoke_element_predicate(vm, program, &predicate, value)? {
            return Ok(Value::bool(true));
        }
    }
    Ok(Value::bool(false))
}

fn slices_index(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _) = extract_slice(vm, program, "slices.Index", args, 2)?;
    let target = &args[1];
    let idx = values.iter().position(|v| v == target);
    Ok(Value::int(idx.map(|i| i as i64).unwrap_or(-1)))
}

fn slices_index_func(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, predicate) = extract_slice_and_func(vm, program, "slices.IndexFunc", args)?;
    for (i, value) in values.iter().enumerate() {
        if invoke_element_predicate(vm, program, &predicate, value)? {
            return Ok(Value::int(i as i64));
        }
    }
    Ok(Value::int(-1))
}

fn slices_sort_func(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, cmp) = extract_slice_and_func(vm, program, "slices.SortFunc", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.SortFunc",
            "a slice argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut indexed: Vec<(usize, Value)> = values.into_iter().enumerate().collect();
    insertion_sort_by_cmp(vm, program, &cmp, &mut indexed)?;
    let sorted: Vec<Value> = indexed.into_iter().map(|(_, v)| v).collect();
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

fn slices_sort_stable_func(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, _, cmp) = extract_slice_and_func(vm, program, "slices.SortStableFunc", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.SortStableFunc",
            "a slice argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut indexed: Vec<(usize, Value)> = values.into_iter().enumerate().collect();
    merge_sort_by_cmp(vm, program, &cmp, &mut indexed)?;
    let sorted: Vec<Value> = indexed.into_iter().map(|(_, v)| v).collect();
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

fn slices_compact(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.Compact",
            "a slice argument",
        )?);
    };
    if slice.is_nil {
        return Ok(args[0].clone());
    }
    let values = slice.values_snapshot();
    if values.is_empty() {
        return Ok(args[0].clone());
    }
    let mut result = vec![values[0].clone()];
    for v in &values[1..] {
        if result.last() != Some(v) {
            result.push(v.clone());
        }
    }
    overwrite_slice_window(slice, &result);
    Ok(clone_slice_result(&args[0], slice, result.len()))
}

fn slices_compact_func(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, eq_func) = extract_slice_and_func(vm, program, "slices.CompactFunc", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.CompactFunc",
            "a slice argument",
        )?);
    };
    if slice.is_nil {
        return Ok(args[0].clone());
    }
    if values.is_empty() {
        return Ok(args[0].clone());
    }
    let mut result = vec![values[0].clone()];
    for v in &values[1..] {
        let last = result.last().unwrap();
        if !invoke_cmp_eq(vm, program, &eq_func, last, v)? {
            result.push(v.clone());
        }
    }
    overwrite_slice_window(slice, &result);
    Ok(clone_slice_result(&args[0], slice, result.len()))
}

fn slices_reverse(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.Reverse",
            "a slice argument",
        )?);
    };
    if slice.is_nil {
        return Ok(args[0].clone());
    }
    let mut values = slice.values_snapshot();
    values.reverse();
    overwrite_slice_window(slice, &values);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

fn slices_equal(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Slice(a) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.Equal",
            "a slice argument",
        )?);
    };
    let ValueData::Slice(b) = &args[1].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            "slices.Equal",
            "a slice argument",
        )?);
    };
    Ok(Value::bool(a.values_snapshot() == b.values_snapshot()))
}

fn extract_slice(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
    expected_args: usize,
) -> Result<(Vec<Value>, usize), VmError> {
    if args.len() != expected_args {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: expected_args,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            builtin,
            "a slice argument",
        )?);
    };
    Ok((slice.values_snapshot(), slice.cap))
}

fn extract_slice_and_func(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<Value>, usize, FunctionValue), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_slices_argument(
            vm,
            program,
            builtin,
            "a slice argument",
        )?);
    };
    let cap = slice.cap;
    let values = slice.values_snapshot();
    let ValueData::Function(f) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[1]),
        });
    };
    Ok((values, cap, f.clone()))
}

fn invoke_element_predicate(
    vm: &mut Vm,
    program: &Program,
    predicate: &FunctionValue,
    value: &Value,
) -> Result<bool, VmError> {
    let mut callback_args = predicate.captures.clone();
    callback_args.push(value.clone());
    let result = vm.invoke_callback(program, predicate.function, callback_args)?;
    match result.data {
        ValueData::Bool(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "callback".into(),
            expected: "bool return value".into(),
        }),
    }
}

fn invoke_cmp_callback(
    vm: &mut Vm,
    program: &Program,
    cmp: &FunctionValue,
    a: &Value,
    b: &Value,
) -> Result<i64, VmError> {
    let mut callback_args = cmp.captures.clone();
    callback_args.push(a.clone());
    callback_args.push(b.clone());
    let result = vm.invoke_callback(program, cmp.function, callback_args)?;
    match result.data {
        ValueData::Int(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "cmp callback".into(),
            expected: "int return value".into(),
        }),
    }
}

fn invoke_cmp_eq(
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

fn insertion_sort_by_cmp(
    vm: &mut Vm,
    program: &Program,
    cmp: &FunctionValue,
    items: &mut [(usize, Value)],
) -> Result<(), VmError> {
    for i in 1..items.len() {
        let mut j = i;
        while j > 0 {
            let ord = invoke_cmp_callback(vm, program, cmp, &items[j].1, &items[j - 1].1)?;
            if ord < 0 {
                items.swap(j, j - 1);
                j -= 1;
            } else {
                break;
            }
        }
    }
    Ok(())
}

fn merge_sort_by_cmp(
    vm: &mut Vm,
    program: &Program,
    cmp: &FunctionValue,
    items: &mut [(usize, Value)],
) -> Result<(), VmError> {
    let len = items.len();
    if len <= 1 {
        return Ok(());
    }
    if len <= 16 {
        return insertion_sort_by_cmp(vm, program, cmp, items);
    }
    let mid = len / 2;
    merge_sort_by_cmp(vm, program, cmp, &mut items[..mid])?;
    merge_sort_by_cmp(vm, program, cmp, &mut items[mid..])?;
    let mut merged: Vec<(usize, Value)> = Vec::with_capacity(len);
    let (mut i, mut j) = (0, mid);
    while i < mid && j < len {
        let ord = invoke_cmp_callback(vm, program, cmp, &items[j].1, &items[i].1)?;
        if ord < 0 {
            merged.push(items[j].clone());
            j += 1;
        } else {
            merged.push(items[i].clone());
            i += 1;
        }
    }
    merged.extend_from_slice(&items[i..mid]);
    merged.extend_from_slice(&items[j..len]);
    items.clone_from_slice(&merged);
    Ok(())
}

fn invalid_slices_argument(
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

fn overwrite_slice_window(slice: &crate::SliceValue, values: &[Value]) {
    let mut storage = slice.values.borrow_mut();
    for (offset, value) in values.iter().enumerate() {
        storage[slice.start + offset] = value.clone();
    }
}

fn clone_slice_result(receiver: &Value, slice: &crate::SliceValue, len: usize) -> Value {
    let mut result_slice = slice.clone();
    result_slice.len = len;
    result_slice.cap = slice.cap;
    result_slice.start = slice.start;
    result_slice.is_nil = slice.is_nil && len == 0;
    result_slice.concrete_type = slice.concrete_type.clone();
    Value {
        typ: if receiver.typ == TYPE_ANY {
            TYPE_SLICE
        } else {
            receiver.typ
        },
        data: ValueData::Slice(result_slice),
    }
}
