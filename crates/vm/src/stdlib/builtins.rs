use super::{
    StdlibFunction, BUILTIN_APPEND, BUILTIN_APPEND_SPREAD, BUILTIN_CAP, BUILTIN_CLEAR,
    BUILTIN_DELETE, BUILTIN_LEN, BUILTIN_MAKE_SLICE, BUILTIN_MAX, BUILTIN_MIN, BUILTIN_RANGE_KEYS,
    BUILTIN_RANGE_VALUE,
};
use crate::{
    explicit_concrete_type_for_value, ConcreteType, Program, Value, ValueData, Vm, VmError,
};

pub(super) const BUILTIN_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: BUILTIN_LEN,
        symbol: "len",
        returns_value: true,
        handler: builtin_len,
    },
    StdlibFunction {
        id: BUILTIN_APPEND,
        symbol: "append",
        returns_value: true,
        handler: builtin_append,
    },
    StdlibFunction {
        id: BUILTIN_RANGE_KEYS,
        symbol: "__range_keys",
        returns_value: true,
        handler: builtin_range_keys,
    },
    StdlibFunction {
        id: BUILTIN_CAP,
        symbol: "cap",
        returns_value: true,
        handler: builtin_cap,
    },
    StdlibFunction {
        id: BUILTIN_RANGE_VALUE,
        symbol: "__range_value",
        returns_value: true,
        handler: builtin_range_value,
    },
    StdlibFunction {
        id: BUILTIN_DELETE,
        symbol: "delete",
        returns_value: true,
        handler: builtin_delete,
    },
    StdlibFunction {
        id: BUILTIN_MAKE_SLICE,
        symbol: "__make_slice",
        returns_value: true,
        handler: builtin_make_slice,
    },
    StdlibFunction {
        id: BUILTIN_APPEND_SPREAD,
        symbol: "__append_spread",
        returns_value: true,
        handler: builtin_append_spread,
    },
    StdlibFunction {
        id: BUILTIN_MIN,
        symbol: "min",
        returns_value: true,
        handler: builtin_min,
    },
    StdlibFunction {
        id: BUILTIN_MAX,
        symbol: "max",
        returns_value: true,
        handler: builtin_max,
    },
    StdlibFunction {
        id: BUILTIN_CLEAR,
        symbol: "clear",
        returns_value: true,
        handler: builtin_clear,
    },
];

fn builtin_len(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let len = match &args[0].data {
        ValueData::String(text) => text.len(),
        ValueData::Array(array) => array.len(),
        ValueData::Slice(slice) => slice.len(),
        ValueData::Map(map) => map.len(),
        ValueData::Channel(channel) => {
            if let Some(id) = channel.id {
                vm.channel_state(program, id)?.buffer.len()
            } else {
                0
            }
        }
        _ => {
            return Err(VmError::InvalidLenArgument {
                function: vm.current_function_name(program)?,
            });
        }
    };
    Ok(Value::int(len as i64))
}

fn builtin_append(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: 0,
        });
    }

    let ValueData::Slice(existing) = &args[0].data else {
        return Err(VmError::InvalidAppendTarget {
            function: vm.current_function_name(program)?,
        });
    };

    Ok(Value {
        typ: crate::TYPE_SLICE,
        data: ValueData::Slice(existing.appended(&args[1..])),
    })
}

fn builtin_append_spread(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::Slice(existing) = &args[0].data else {
        return Err(VmError::InvalidAppendTarget {
            function: vm.current_function_name(program)?,
        });
    };

    let ValueData::Slice(source) = &args[1].data else {
        return Err(VmError::InvalidAppendTarget {
            function: vm.current_function_name(program)?,
        });
    };

    Ok(Value {
        typ: crate::TYPE_SLICE,
        data: ValueData::Slice(existing.appended(&source.values_snapshot())),
    })
}

fn builtin_range_keys(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let keys = match &args[0].data {
        ValueData::Array(array) => (0..array.len())
            .map(|index| Value::int(index as i64))
            .collect(),
        ValueData::Slice(slice) => (0..slice.len())
            .map(|index| Value::int(index as i64))
            .collect(),
        ValueData::String(text) => text
            .char_indices()
            .map(|(index, _)| Value::int(index as i64))
            .collect(),
        ValueData::Map(map) => map
            .entries_snapshot()
            .into_iter()
            .map(|(key, _)| key)
            .collect(),
        _ => {
            return Err(VmError::InvalidRangeTarget {
                function: vm.current_function_name(program)?,
            });
        }
    };
    Ok(Value::slice(keys))
}

fn builtin_cap(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let cap = match &args[0].data {
        ValueData::Array(array) => array.len(),
        ValueData::Slice(slice) => slice.cap,
        ValueData::Channel(channel) => {
            if let Some(id) = channel.id {
                vm.channel_state(program, id)?.capacity
            } else {
                0
            }
        }
        _ => {
            return Err(VmError::InvalidCapArgument {
                function: vm.current_function_name(program)?,
            });
        }
    };
    Ok(Value::int(cap as i64))
}

fn builtin_range_value(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let value = match &args[0].data {
        ValueData::Array(array) => {
            let ValueData::Int(index) = args[1].data else {
                return Err(VmError::InvalidIndexValue {
                    function: vm.current_function_name(program)?,
                });
            };
            if index < 0 {
                return Err(VmError::IndexOutOfBounds {
                    function: vm.current_function_name(program)?,
                    index,
                    len: 0,
                });
            }
            array.get(index as usize).ok_or(VmError::IndexOutOfBounds {
                function: vm.current_function_name(program)?,
                index,
                len: array.len(),
            })?
        }
        ValueData::Slice(slice) => {
            let ValueData::Int(index) = args[1].data else {
                return Err(VmError::InvalidIndexValue {
                    function: vm.current_function_name(program)?,
                });
            };
            if index < 0 {
                return Err(VmError::IndexOutOfBounds {
                    function: vm.current_function_name(program)?,
                    index,
                    len: 0,
                });
            }
            slice.get(index as usize).ok_or(VmError::IndexOutOfBounds {
                function: vm.current_function_name(program)?,
                index,
                len: slice.len(),
            })?
        }
        ValueData::Map(map) => map
            .get(&args[1])
            .unwrap_or_else(|| (*map.zero_value).clone()),
        ValueData::String(text) => {
            let ValueData::Int(index) = args[1].data else {
                return Err(VmError::InvalidIndexValue {
                    function: vm.current_function_name(program)?,
                });
            };
            if index < 0 {
                return Err(VmError::IndexOutOfBounds {
                    function: vm.current_function_name(program)?,
                    index,
                    len: 0,
                });
            }
            let index = index as usize;
            let character = text
                .get(index..)
                .and_then(|tail| tail.chars().next())
                .ok_or(VmError::IndexOutOfBounds {
                    function: vm.current_function_name(program)?,
                    index: index as i64,
                    len: text.len(),
                })?;
            Value::int(character as i64)
        }
        _ => {
            return Err(VmError::InvalidRangeTarget {
                function: vm.current_function_name(program)?,
            });
        }
    };
    Ok(value)
}

fn builtin_delete(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::Map(map) = &args[0].data else {
        return Err(VmError::InvalidDeleteTarget {
            function: vm.current_function_name(program)?,
        });
    };

    map.remove(&args[1]);
    Ok(args[0].clone())
}

fn builtin_min(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: 0,
        });
    }
    let mut result = args[0].clone();
    for arg in &args[1..] {
        if compare_ordered_values(&result, arg)? > 0 {
            result = arg.clone();
        }
    }
    Ok(result)
}

fn builtin_max(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: 0,
        });
    }
    let mut result = args[0].clone();
    for arg in &args[1..] {
        if compare_ordered_values(&result, arg)? < 0 {
            result = arg.clone();
        }
    }
    Ok(result)
}

fn builtin_clear(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    match &args[0].data {
        ValueData::Map(map) => {
            map.clear();
            Ok(args[0].clone())
        }
        ValueData::Slice(slice) => {
            let visible_values = slice.values_snapshot();
            let zero = if visible_values.is_empty() {
                Value::int(0)
            } else {
                zero_for_value(&visible_values[0])
            };
            for index in 0..slice.len() {
                let replaced = slice.set(index, zero.clone());
                debug_assert!(replaced);
            }
            Ok(args[0].clone())
        }
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "clear".into(),
            expected: "a map or slice argument".into(),
        }),
    }
}

fn compare_ordered_values(a: &Value, b: &Value) -> Result<i64, VmError> {
    match (&a.data, &b.data) {
        (ValueData::Int(x), ValueData::Int(y)) => Ok(ord_to_int(x.cmp(y))),
        (ValueData::Float(x), ValueData::Float(y)) => Ok(ord_to_int(
            x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal),
        )),
        (ValueData::String(x), ValueData::String(y)) => Ok(ord_to_int(x.cmp(y))),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: "builtin".into(),
            builtin: "min/max".into(),
            expected: "ordered type arguments".into(),
        }),
    }
}

fn ord_to_int(ord: std::cmp::Ordering) -> i64 {
    match ord {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn zero_for_value(v: &Value) -> Value {
    match &v.data {
        ValueData::Int(_) => Value::int(0),
        ValueData::Float(_) => Value::float(0.0),
        ValueData::String(_) => Value::string(""),
        ValueData::Bool(_) => Value::bool(false),
        _ => Value::nil(),
    }
}

fn builtin_make_slice(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !(2..=3).contains(&args.len()) {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::Int(len) = args[0].data else {
        return Err(VmError::InvalidMakeLength {
            function: vm.current_function_name(program)?,
        });
    };
    if len < 0 {
        return Err(VmError::NegativeMakeLength {
            function: vm.current_function_name(program)?,
            len,
        });
    }

    let (cap, zero) = if args.len() == 2 {
        (len, &args[1])
    } else {
        let ValueData::Int(cap) = args[1].data else {
            return Err(VmError::InvalidMakeCapacity {
                function: vm.current_function_name(program)?,
            });
        };
        if cap < 0 {
            return Err(VmError::NegativeMakeCapacity {
                function: vm.current_function_name(program)?,
                cap,
            });
        }
        if cap < len {
            return Err(VmError::MakeCapacityLessThanLength {
                function: vm.current_function_name(program)?,
                len,
                cap,
            });
        }
        (cap, &args[2])
    };

    let values = vec![zero.clone(); len as usize];
    let concrete_type = explicit_concrete_type_for_value(zero).map(|element| ConcreteType::Slice {
        element: Box::new(element),
    });
    Ok(match concrete_type {
        Some(concrete_type) => Value::slice_with_cap_typed(values, cap as usize, concrete_type),
        None => Value::slice_with_cap(values, cap as usize),
    })
}
