use super::{
    StdlibFunction, LOG_FATAL, LOG_FATALF, LOG_FLAGS, LOG_PREFIX, LOG_PRINT, LOG_PRINTF,
    LOG_PRINTLN, LOG_SET_FLAGS, LOG_SET_PREFIX,
};
use crate::{Program, Value, Vm, VmError};

pub(super) const LOG_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: LOG_SET_FLAGS,
        symbol: "SetFlags",
        returns_value: false,
        handler: log_set_flags,
    },
    StdlibFunction {
        id: LOG_FLAGS,
        symbol: "Flags",
        returns_value: true,
        handler: log_flags,
    },
    StdlibFunction {
        id: LOG_SET_PREFIX,
        symbol: "SetPrefix",
        returns_value: false,
        handler: log_set_prefix,
    },
    StdlibFunction {
        id: LOG_PREFIX,
        symbol: "Prefix",
        returns_value: true,
        handler: log_prefix,
    },
    StdlibFunction {
        id: LOG_PRINTLN,
        symbol: "Println",
        returns_value: false,
        handler: log_println,
    },
    StdlibFunction {
        id: LOG_PRINTF,
        symbol: "Printf",
        returns_value: false,
        handler: log_printf,
    },
    StdlibFunction {
        id: LOG_PRINT,
        symbol: "Print",
        returns_value: false,
        handler: log_print,
    },
    StdlibFunction {
        id: LOG_FATAL,
        symbol: "Fatal",
        returns_value: false,
        handler: log_fatal,
    },
    StdlibFunction {
        id: LOG_FATALF,
        symbol: "Fatalf",
        returns_value: false,
        handler: log_fatalf,
    },
];

fn log_set_flags(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let Some(flags) = args.first() else {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    };
    let crate::ValueData::Int(flags) = flags.data else {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "log: flags must be an int".into(),
        });
    };
    if flags != 0 {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "log: non-zero flags are outside the supported slice".into(),
        });
    }
    vm.log_flags = flags;
    Ok(Value::nil())
}

fn log_flags(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(Value::int(vm.log_flags))
}

fn log_set_prefix(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let Some(prefix) = args.first() else {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    };
    let crate::ValueData::String(prefix) = &prefix.data else {
        return Err(VmError::UnhandledPanic {
            function: vm.current_function_name(program)?,
            value: "log: prefix must be a string".into(),
        });
    };
    vm.log_prefix = prefix.clone();
    Ok(Value::nil())
}

fn log_prefix(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(Value::string(vm.log_prefix.clone()))
}

fn log_println(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let message = super::fmt_impl::sprintln_impl(vm, program, args)?;
    write_log_entry(vm, &message);
    Ok(Value::nil())
}

fn log_printf(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let result = super::fmt_impl::sprintf_impl(vm, program, args)?;
    write_log_entry(vm, &result);
    Ok(Value::nil())
}

fn log_print(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let message = super::fmt_impl::sprint_impl(vm, program, args)?;
    write_log_entry(vm, &message);
    Ok(Value::nil())
}

fn log_fatal(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let message = super::fmt_impl::sprint_impl(vm, program, args)?;
    write_log_entry(vm, &message);
    Err(VmError::ProgramExit { code: 1 })
}

fn log_fatalf(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let result = super::fmt_impl::sprintf_impl(vm, program, args)?;
    write_log_entry(vm, &result);
    Err(VmError::ProgramExit { code: 1 })
}

fn write_log_entry(vm: &mut Vm, message: &str) {
    vm.stdout.push_str(&vm.log_prefix);
    vm.stdout.push_str(message);
    if !message.ends_with('\n') {
        vm.stdout.push('\n');
    }
}
