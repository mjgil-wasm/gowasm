use super::{
    fmt_impl, StdlibFunction, StdlibMethod, TESTING_NEW_T, TESTING_T_ERRORF, TESTING_T_FAILED,
    TESTING_T_RUN,
};
use crate::{
    format_value, FunctionValue, Program, Value, ValueData, Vm, VmError, TYPE_TESTING_T,
    TYPE_TESTING_T_PTR,
};

const TESTING_NAME_FIELD: &str = "__testing_name";
const TESTING_FAILED_FIELD: &str = "__testing_failed";

pub(super) const TESTING_FUNCTIONS: &[StdlibFunction] = &[StdlibFunction {
    id: TESTING_NEW_T,
    symbol: "__NewT",
    returns_value: true,
    handler: testing_new_t,
}];

pub(super) const TESTING_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "*testing.T",
        method: "Errorf",
        function: TESTING_T_ERRORF,
    },
    StdlibMethod {
        receiver_type: "*testing.T",
        method: "Run",
        function: TESTING_T_RUN,
    },
    StdlibMethod {
        receiver_type: "*testing.T",
        method: "Failed",
        function: TESTING_T_FAILED,
    },
];

pub(super) const TESTING_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: TESTING_T_ERRORF,
        symbol: "Errorf",
        returns_value: false,
        handler: testing_t_errorf,
    },
    StdlibFunction {
        id: TESTING_T_RUN,
        symbol: "Run",
        returns_value: true,
        handler: testing_t_run,
    },
    StdlibFunction {
        id: TESTING_T_FAILED,
        symbol: "Failed",
        returns_value: true,
        handler: testing_t_failed,
    },
];

fn testing_new_t(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let name = string_arg(vm, program, "testing.__NewT", &args[0])?;
    Ok(new_testing_t(vm, name))
}

fn testing_t_errorf(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() < 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let name = testing_name(vm, program, &args[0])?;
    testing_set_failed(vm, program, &args[0], true)?;
    let message = fmt_impl::sprintf_impl(vm, program, &args[1..])?;
    vm.stdout.push_str("FAIL ");
    vm.stdout.push_str(&name);
    vm.stdout.push_str(": ");
    vm.stdout.push_str(&message);
    vm.stdout.push('\n');
    Ok(Value::nil())
}

fn testing_t_run(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let parent_name = testing_name(vm, program, &args[0])?;
    let subtest_name = string_arg(vm, program, "(*testing.T).Run", &args[1])?;
    let callback = function_arg(vm, program, "(*testing.T).Run", &args[2])?;
    let full_name = if parent_name.is_empty() {
        subtest_name.to_string()
    } else {
        format!("{parent_name}/{subtest_name}")
    };
    vm.stdout.push_str("RUN ");
    vm.stdout.push_str(&full_name);
    vm.stdout.push('\n');

    let child = new_testing_t(vm, &full_name);
    let mut callback_args = callback.captures.clone();
    callback_args.push(child.clone());
    match vm.invoke_callback_no_result_or_panic(program, callback.function, callback_args)? {
        Ok(()) => {}
        Err(value) => {
            return Err(VmError::UnhandledPanic {
                function: vm.current_function_name(program)?,
                value: format_value(&value),
            });
        }
    }

    let failed = testing_failed(vm, program, &child)?;
    if failed {
        testing_set_failed(vm, program, &args[0], true)?;
        vm.stdout.push_str("FAIL ");
        vm.stdout.push_str(&full_name);
        vm.stdout.push('\n');
        Ok(Value::bool(false))
    } else {
        vm.stdout.push_str("PASS ");
        vm.stdout.push_str(&full_name);
        vm.stdout.push('\n');
        Ok(Value::bool(true))
    }
}

fn testing_t_failed(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::bool(testing_failed(vm, program, &args[0])?))
}

fn new_testing_t(vm: &mut Vm, name: &str) -> Value {
    vm.box_heap_value(
        Value {
            typ: TYPE_TESTING_T,
            data: ValueData::Struct(vec![
                (TESTING_NAME_FIELD.into(), Value::string(name)),
                (TESTING_FAILED_FIELD.into(), Value::bool(false)),
            ]),
        },
        TYPE_TESTING_T_PTR,
    )
}

fn testing_name(vm: &Vm, program: &Program, receiver: &Value) -> Result<String, VmError> {
    let value = vm.deref_pointer(program, receiver)?;
    let ValueData::Struct(fields) = value.data else {
        return Err(VmError::InvalidFieldTarget {
            function: vm.current_function_name(program)?,
            target: "testing.T".into(),
        });
    };
    fields
        .into_iter()
        .find_map(|(name, value)| {
            (name == TESTING_NAME_FIELD).then_some(match value.data {
                ValueData::String(text) => Ok(text),
                _ => Err(VmError::InvalidFieldTarget {
                    function: vm
                        .current_function_name(program)
                        .unwrap_or_else(|_| "<vm>".into()),
                    target: "testing.T".into(),
                }),
            })
        })
        .unwrap_or_else(|| Ok(String::new()))
}

fn testing_failed(vm: &Vm, program: &Program, receiver: &Value) -> Result<bool, VmError> {
    let value = vm.deref_pointer(program, receiver)?;
    let ValueData::Struct(fields) = value.data else {
        return Err(VmError::InvalidFieldTarget {
            function: vm.current_function_name(program)?,
            target: "testing.T".into(),
        });
    };
    Ok(fields
        .into_iter()
        .find_map(|(name, value)| {
            (name == TESTING_FAILED_FIELD).then_some(matches!(value.data, ValueData::Bool(true)))
        })
        .unwrap_or(false))
}

fn testing_set_failed(
    vm: &mut Vm,
    program: &Program,
    receiver: &Value,
    failed: bool,
) -> Result<(), VmError> {
    let mut value = vm.deref_pointer(program, receiver)?;
    let ValueData::Struct(fields) = &mut value.data else {
        return Err(VmError::InvalidFieldTarget {
            function: vm.current_function_name(program)?,
            target: "testing.T".into(),
        });
    };
    if let Some((_, field)) = fields
        .iter_mut()
        .find(|(name, _)| name == TESTING_FAILED_FIELD)
    {
        *field = Value::bool(failed);
    } else {
        fields.push((TESTING_FAILED_FIELD.into(), Value::bool(failed)));
    }
    vm.store_indirect(program, receiver, value)
}

fn string_arg<'a>(
    vm: &Vm,
    program: &Program,
    function: &str,
    value: &'a Value,
) -> Result<&'a str, VmError> {
    let ValueData::String(text) = &value.data else {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: format!("{function} expected string"),
        });
    };
    Ok(text)
}

fn function_arg(
    vm: &Vm,
    program: &Program,
    function: &str,
    value: &Value,
) -> Result<FunctionValue, VmError> {
    let ValueData::Function(callback) = &value.data else {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: format!("{function} expected callback"),
        });
    };
    Ok(callback.clone())
}
