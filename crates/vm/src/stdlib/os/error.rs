use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) fn os_is_not_exist(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    os_error_matches(vm, program, args, is_not_exist_error)
}

pub(super) fn os_is_exist(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    os_error_matches(vm, program, args, is_exist_error)
}

pub(super) fn os_is_permission(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    os_error_matches(vm, program, args, is_permission_error)
}

pub(super) fn os_is_timeout(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    os_error_matches(vm, program, args, is_timeout_error)
}

pub(super) fn os_new_syscall_error(
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

    let ValueData::String(syscall) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "os.NewSyscallError".into(),
            expected: "a string argument".into(),
        });
    };

    match &args[1].data {
        ValueData::Nil => Ok(Value::nil()),
        ValueData::Error(err) => Ok(Value::wrapped_error(
            format!("{syscall}: {}", err.message),
            args[1].clone(),
        )),
        _ => Err(VmError::InvalidErrorValue {
            function: vm.current_function_name(program)?,
        }),
    }
}

fn os_error_matches(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    predicate: fn(&str) -> bool,
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    Ok(Value::bool(error_chain_matches(&args[0], predicate)))
}

fn is_not_exist_error(message: &str) -> bool {
    message == "file does not exist" || message.ends_with(": file does not exist")
}

fn is_exist_error(message: &str) -> bool {
    message == "file already exists" || message.ends_with(": file already exists")
}

fn is_permission_error(message: &str) -> bool {
    message == "permission denied" || message.ends_with(": permission denied")
}

fn is_timeout_error(message: &str) -> bool {
    message == "i/o timeout" || message.ends_with(": i/o timeout")
}

fn error_chain_matches(value: &Value, predicate: fn(&str) -> bool) -> bool {
    let mut current = value.clone();
    loop {
        match &current.data {
            ValueData::Error(error) => {
                if predicate(&error.message) {
                    return true;
                }
                if let Some(kind) = error.kind_message.as_deref() {
                    if predicate(kind) {
                        return true;
                    }
                }
                let Some(inner) = &error.wrapped else {
                    return false;
                };
                current = inner.as_ref().clone();
            }
            _ => return false,
        }
    }
}
