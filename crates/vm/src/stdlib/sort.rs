use super::{
    StdlibFunction, SORT_FLOAT64S, SORT_FLOAT64S_ARE_SORTED, SORT_INTS, SORT_INTS_ARE_SORTED,
    SORT_SEARCH, SORT_SEARCH_FLOAT64S, SORT_SEARCH_INTS, SORT_SEARCH_STRINGS, SORT_SLICE,
    SORT_SLICE_IS_SORTED, SORT_SLICE_STABLE, SORT_STRINGS, SORT_STRINGS_ARE_SORTED,
};
use crate::{Float64, FunctionValue, Program, Value, ValueData, Vm, VmError, TYPE_ANY, TYPE_SLICE};

pub(super) const SORT_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: SORT_INTS_ARE_SORTED,
        symbol: "IntsAreSorted",
        returns_value: true,
        handler: sort_ints_are_sorted,
    },
    StdlibFunction {
        id: SORT_STRINGS_ARE_SORTED,
        symbol: "StringsAreSorted",
        returns_value: true,
        handler: sort_strings_are_sorted,
    },
    StdlibFunction {
        id: SORT_SEARCH_INTS,
        symbol: "SearchInts",
        returns_value: true,
        handler: sort_search_ints,
    },
    StdlibFunction {
        id: SORT_SEARCH_STRINGS,
        symbol: "SearchStrings",
        returns_value: true,
        handler: sort_search_strings,
    },
    StdlibFunction {
        id: SORT_INTS,
        symbol: "Ints",
        returns_value: true,
        handler: sort_ints,
    },
    StdlibFunction {
        id: SORT_STRINGS,
        symbol: "Strings",
        returns_value: true,
        handler: sort_strings,
    },
    StdlibFunction {
        id: SORT_FLOAT64S,
        symbol: "Float64s",
        returns_value: true,
        handler: sort_float64s,
    },
    StdlibFunction {
        id: SORT_FLOAT64S_ARE_SORTED,
        symbol: "Float64sAreSorted",
        returns_value: true,
        handler: sort_float64s_are_sorted,
    },
    StdlibFunction {
        id: SORT_SEARCH_FLOAT64S,
        symbol: "SearchFloat64s",
        returns_value: true,
        handler: sort_search_float64s,
    },
    StdlibFunction {
        id: SORT_SLICE,
        symbol: "Slice",
        returns_value: true,
        handler: sort_slice,
    },
    StdlibFunction {
        id: SORT_SLICE_IS_SORTED,
        symbol: "SliceIsSorted",
        returns_value: true,
        handler: sort_slice_is_sorted,
    },
    StdlibFunction {
        id: SORT_SLICE_STABLE,
        symbol: "SliceStable",
        returns_value: true,
        handler: sort_slice_stable,
    },
    StdlibFunction {
        id: SORT_SEARCH,
        symbol: "Search",
        returns_value: true,
        handler: sort_search,
    },
];

pub(super) fn sort_ints_are_sorted(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let values = int_slice_arg(vm, program, "sort.IntsAreSorted", args)?;
    let sorted = values.windows(2).all(|pair| {
        int_slice_value(vm, program, "sort.IntsAreSorted", &pair[0]).unwrap()
            <= int_slice_value(vm, program, "sort.IntsAreSorted", &pair[1]).unwrap()
    });
    Ok(Value::bool(sorted))
}

pub(super) fn sort_strings_are_sorted(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let values = string_slice_arg(vm, program, "sort.StringsAreSorted", args)?;
    let sorted = values.windows(2).all(|pair| {
        string_slice_value(vm, program, "sort.StringsAreSorted", &pair[0]).unwrap()
            <= string_slice_value(vm, program, "sort.StringsAreSorted", &pair[1]).unwrap()
    });
    Ok(Value::bool(sorted))
}

pub(super) fn sort_search_ints(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, target) = int_slice_and_int_arg(vm, program, "sort.SearchInts", args)?;
    let mut low = 0usize;
    let mut high = values.len();
    while low < high {
        let mid = low + (high - low) / 2;
        let value = int_slice_value(vm, program, "sort.SearchInts", &values[mid])?;
        if value < target {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    Ok(Value::int(low as i64))
}

pub(super) fn sort_search_strings(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, target) = string_slice_and_string_arg(vm, program, "sort.SearchStrings", args)?;
    let mut low = 0usize;
    let mut high = values.len();
    while low < high {
        let mid = low + (high - low) / 2;
        let value = string_slice_value(vm, program, "sort.SearchStrings", &values[mid])?;
        if value < target.as_str() {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    Ok(Value::int(low as i64))
}

pub(super) fn sort_ints(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let values = int_slice_arg(vm, program, "sort.Ints", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            "sort.Ints",
            "a []int argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut sorted: Vec<Value> = values.to_vec();
    sorted.sort_by(|a, b| {
        let ValueData::Int(a) = a.data else {
            unreachable!()
        };
        let ValueData::Int(b) = b.data else {
            unreachable!()
        };
        a.cmp(&b)
    });
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

pub(super) fn sort_strings(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let values = string_slice_arg(vm, program, "sort.Strings", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            "sort.Strings",
            "a []string argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut sorted: Vec<Value> = values.to_vec();
    sorted.sort_by(|a, b| {
        let ValueData::String(a) = &a.data else {
            unreachable!()
        };
        let ValueData::String(b) = &b.data else {
            unreachable!()
        };
        a.cmp(b)
    });
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

pub(super) fn sort_float64s(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let values = float_slice_arg(vm, program, "sort.Float64s", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            "sort.Float64s",
            "a []float64 argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut sorted: Vec<Value> = values.to_vec();
    sorted.sort_by(|a, b| {
        let ValueData::Float(Float64(a)) = a.data else {
            unreachable!()
        };
        let ValueData::Float(Float64(b)) = b.data else {
            unreachable!()
        };
        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    });
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

fn sort_float64s_are_sorted(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let values = float_slice_arg(vm, program, "sort.Float64sAreSorted", args)?;
    let sorted = values.windows(2).all(|pair| {
        let ValueData::Float(Float64(a)) = pair[0].data else {
            unreachable!()
        };
        let ValueData::Float(Float64(b)) = pair[1].data else {
            unreachable!()
        };
        a <= b
    });
    Ok(Value::bool(sorted))
}

fn sort_search_float64s(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, target) = float_slice_and_float_args(vm, program, "sort.SearchFloat64s", args)?;
    let position = values
        .iter()
        .position(|v| {
            let ValueData::Float(Float64(f)) = v.data else {
                unreachable!()
            };
            f >= target
        })
        .unwrap_or(values.len());
    Ok(Value::int(position as i64))
}

fn float_slice_and_float_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<Value>, f64), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []float64 and float64 argument",
        )?);
    };
    let ValueData::Float(Float64(target)) = &args[1].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []float64 and float64 argument",
        )?);
    };
    Ok((slice.values_snapshot(), *target))
}

fn float_slice_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []float64 argument",
        )?);
    };
    let values = slice.values_snapshot();
    for value in &values {
        if !matches!(value.data, ValueData::Float(_)) {
            return Err(invalid_sort_argument(
                vm,
                program,
                builtin,
                "a []float64 argument",
            )?);
        }
    }
    Ok(values)
}

fn int_slice_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []int argument",
        )?);
    };
    let values = slice.values_snapshot();
    for value in &values {
        int_slice_value(vm, program, builtin, value)?;
    }
    Ok(values)
}

fn string_slice_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []string argument",
        )?);
    };
    let values = slice.values_snapshot();
    for value in &values {
        string_slice_value(vm, program, builtin, value)?;
    }
    Ok(values)
}

fn int_slice_and_int_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<Value>, i64), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let values = int_slice_arg(vm, program, builtin, &args[..1])?;
    let ValueData::Int(target) = &args[1].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []int and int argument",
        )?);
    };
    Ok((values, *target))
}

fn string_slice_and_string_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(Vec<Value>, String), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let values = string_slice_arg(vm, program, builtin, &args[..1])?;
    let ValueData::String(target) = &args[1].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []string and string argument",
        )?);
    };
    Ok((values, target.clone()))
}

fn int_slice_value(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &Value,
) -> Result<i64, VmError> {
    match &value.data {
        ValueData::Int(number) => Ok(*number),
        _ => Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []int argument",
        )?),
    }
}

fn string_slice_value<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    value: &'a Value,
) -> Result<&'a str, VmError> {
    let ValueData::String(text) = &value.data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a []string argument",
        )?);
    };
    Ok(text)
}

fn invalid_sort_argument(
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

fn sort_slice(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, less) = slice_and_less_args(vm, program, "sort.Slice", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            "sort.Slice",
            "a slice argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut indices: Vec<usize> = (0..values.len()).collect();
    insertion_sort_indices(vm, program, &less, &mut indices)?;
    let sorted: Vec<Value> = indices.iter().map(|&i| values[i].clone()).collect();
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

fn sort_slice_is_sorted(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, less) = slice_and_less_args(vm, program, "sort.SliceIsSorted", args)?;
    for i in 1..values.len() {
        if invoke_less_callback(vm, program, &less, i, i - 1)? {
            return Ok(Value::bool(false));
        }
    }
    Ok(Value::bool(true))
}

fn sort_slice_stable(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (values, _, less) = slice_and_less_args(vm, program, "sort.SliceStable", args)?;
    let ValueData::Slice(slice) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            "sort.SliceStable",
            "a slice argument",
        )?);
    };
    if slice.is_nil || values.len() < 2 {
        return Ok(args[0].clone());
    }
    let mut indices: Vec<usize> = (0..values.len()).collect();
    merge_sort_indices(vm, program, &less, &mut indices)?;
    let sorted: Vec<Value> = indices.iter().map(|&i| values[i].clone()).collect();
    overwrite_slice_window(slice, &sorted);
    Ok(clone_slice_result(&args[0], slice, slice.len))
}

fn sort_search(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Int(n) = &args[0].data else {
        return Err(invalid_sort_argument(
            vm,
            program,
            "sort.Search",
            "an int argument",
        )?);
    };
    let n = *n as usize;
    let ValueData::Function(f) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[1]),
        });
    };
    let f = f.clone();
    let mut low = 0usize;
    let mut high = n;
    while low < high {
        let mid = low + (high - low) / 2;
        let mut callback_args = f.captures.clone();
        callback_args.push(Value::int(mid as i64));
        let result = vm.invoke_callback(program, f.function, callback_args)?;
        match result.data {
            ValueData::Bool(true) => {
                high = mid;
            }
            ValueData::Bool(false) => {
                low = mid + 1;
            }
            _ => {
                return Err(VmError::InvalidStringFunctionArgument {
                    function: vm.current_function_name(program)?,
                    builtin: "sort.Search callback".into(),
                    expected: "bool return value".into(),
                });
            }
        }
    }
    Ok(Value::int(low as i64))
}

fn slice_and_less_args(
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
        return Err(invalid_sort_argument(
            vm,
            program,
            builtin,
            "a slice argument",
        )?);
    };
    let cap = slice.cap;
    let values = slice.values_snapshot();
    let ValueData::Function(less) = &args[1].data else {
        return Err(VmError::InvalidFunctionValue {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[1]),
        });
    };
    Ok((values, cap, less.clone()))
}

fn insertion_sort_indices(
    vm: &mut Vm,
    program: &Program,
    less: &FunctionValue,
    indices: &mut [usize],
) -> Result<(), VmError> {
    for i in 1..indices.len() {
        let mut j = i;
        while j > 0 {
            if invoke_less_callback(vm, program, less, indices[j], indices[j - 1])? {
                indices.swap(j, j - 1);
                j -= 1;
            } else {
                break;
            }
        }
    }
    Ok(())
}

fn merge_sort_indices(
    vm: &mut Vm,
    program: &Program,
    less: &FunctionValue,
    indices: &mut [usize],
) -> Result<(), VmError> {
    let len = indices.len();
    if len <= 1 {
        return Ok(());
    }
    if len <= 16 {
        for i in 1..len {
            let mut j = i;
            while j > 0 {
                if invoke_less_callback(vm, program, less, indices[j], indices[j - 1])? {
                    indices.swap(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }
        return Ok(());
    }
    let mid = len / 2;
    merge_sort_indices(vm, program, less, &mut indices[..mid])?;
    merge_sort_indices(vm, program, less, &mut indices[mid..])?;
    let mut merged = Vec::with_capacity(len);
    let (mut i, mut j) = (0, mid);
    while i < mid && j < len {
        if invoke_less_callback(vm, program, less, indices[j], indices[i])? {
            merged.push(indices[j]);
            j += 1;
        } else {
            merged.push(indices[i]);
            i += 1;
        }
    }
    merged.extend_from_slice(&indices[i..mid]);
    merged.extend_from_slice(&indices[j..len]);
    indices.copy_from_slice(&merged);
    Ok(())
}

fn invoke_less_callback(
    vm: &mut Vm,
    program: &Program,
    less: &FunctionValue,
    i: usize,
    j: usize,
) -> Result<bool, VmError> {
    let mut callback_args = less.captures.clone();
    callback_args.push(Value::int(i as i64));
    callback_args.push(Value::int(j as i64));
    let result = vm.invoke_callback(program, less.function, callback_args)?;
    match result.data {
        ValueData::Bool(v) => Ok(v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "sort.Slice callback".into(),
            expected: "bool return value".into(),
        }),
    }
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
