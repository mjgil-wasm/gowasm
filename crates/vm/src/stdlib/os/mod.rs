use super::{
    StdlibConstant, StdlibConstantValue, StdlibFunction, StdlibValue, StdlibValueInit, OS_CLEARENV,
    OS_DIR_FS, OS_ENVIRON, OS_EXECUTABLE, OS_EXIT, OS_EXPAND, OS_EXPAND_ENV, OS_GETEGID, OS_GETENV,
    OS_GETEUID, OS_GETGID, OS_GETGROUPS, OS_GETPAGESIZE, OS_GETPID, OS_GETPPID, OS_GETUID,
    OS_GETWD, OS_HOSTNAME, OS_IS_EXIST, OS_IS_NOT_EXIST, OS_IS_PATH_SEPARATOR, OS_IS_PERMISSION,
    OS_IS_TIMEOUT, OS_LOOKUP_ENV, OS_LSTAT, OS_MKDIR_ALL, OS_NEW_SYSCALL_ERROR, OS_READ_DIR,
    OS_READ_FILE, OS_REMOVE_ALL, OS_SAME_FILE, OS_SETENV, OS_STAT, OS_TEMP_DIR, OS_UNSETENV,
    OS_USER_CACHE_DIR, OS_USER_CONFIG_DIR, OS_USER_HOME_DIR, OS_WRITE_FILE,
};
use crate::{Program, Value, ValueData, Vm, VmError};

mod env;
mod error;
mod fs;

pub(super) const OS_CONSTANTS: &[StdlibConstant] = &[
    StdlibConstant {
        symbol: "PathSeparator",
        typ: "byte",
        value: StdlibConstantValue::Int(b'/' as i64),
    },
    StdlibConstant {
        symbol: "PathListSeparator",
        typ: "byte",
        value: StdlibConstantValue::Int(b':' as i64),
    },
    StdlibConstant {
        symbol: "DevNull",
        typ: "string",
        value: StdlibConstantValue::String("/dev/null"),
    },
];

pub(super) const OS_VALUES: &[StdlibValue] = &[
    StdlibValue {
        symbol: "ErrInvalid",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("invalid argument")),
    },
    StdlibValue {
        symbol: "ErrPermission",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("permission denied")),
    },
    StdlibValue {
        symbol: "ErrExist",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("file already exists")),
    },
    StdlibValue {
        symbol: "ErrNotExist",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("file does not exist")),
    },
    StdlibValue {
        symbol: "ErrClosed",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("file already closed")),
    },
    StdlibValue {
        symbol: "ErrDeadlineExceeded",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("i/o timeout")),
    },
    StdlibValue {
        symbol: "ErrNoDeadline",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(
            "file type does not support deadline",
        )),
    },
];

pub(super) const OS_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: OS_EXIT,
        symbol: "Exit",
        returns_value: false,
        handler: os_exit,
    },
    StdlibFunction {
        id: OS_GETENV,
        symbol: "Getenv",
        returns_value: true,
        handler: env::os_getenv,
    },
    StdlibFunction {
        id: OS_SETENV,
        symbol: "Setenv",
        returns_value: false,
        handler: env::os_setenv,
    },
    StdlibFunction {
        id: OS_UNSETENV,
        symbol: "Unsetenv",
        returns_value: false,
        handler: env::os_unsetenv,
    },
    StdlibFunction {
        id: OS_CLEARENV,
        symbol: "Clearenv",
        returns_value: false,
        handler: env::os_clearenv,
    },
    StdlibFunction {
        id: OS_LOOKUP_ENV,
        symbol: "LookupEnv",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_DIR_FS,
        symbol: "DirFS",
        returns_value: true,
        handler: fs::os_dir_fs,
    },
    StdlibFunction {
        id: OS_READ_FILE,
        symbol: "ReadFile",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_WRITE_FILE,
        symbol: "WriteFile",
        returns_value: true,
        handler: fs::os_write_file,
    },
    StdlibFunction {
        id: OS_ENVIRON,
        symbol: "Environ",
        returns_value: true,
        handler: env::os_environ,
    },
    StdlibFunction {
        id: OS_EXPAND_ENV,
        symbol: "ExpandEnv",
        returns_value: true,
        handler: env::os_expand_env,
    },
    StdlibFunction {
        id: OS_EXPAND,
        symbol: "Expand",
        returns_value: true,
        handler: env::os_expand,
    },
    StdlibFunction {
        id: OS_READ_DIR,
        symbol: "ReadDir",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_STAT,
        symbol: "Stat",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_LSTAT,
        symbol: "Lstat",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_MKDIR_ALL,
        symbol: "MkdirAll",
        returns_value: true,
        handler: fs::os_mkdir_all,
    },
    StdlibFunction {
        id: OS_REMOVE_ALL,
        symbol: "RemoveAll",
        returns_value: true,
        handler: fs::os_remove_all,
    },
    StdlibFunction {
        id: OS_GETWD,
        symbol: "Getwd",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_IS_EXIST,
        symbol: "IsExist",
        returns_value: true,
        handler: error::os_is_exist,
    },
    StdlibFunction {
        id: OS_IS_NOT_EXIST,
        symbol: "IsNotExist",
        returns_value: true,
        handler: error::os_is_not_exist,
    },
    StdlibFunction {
        id: OS_IS_PATH_SEPARATOR,
        symbol: "IsPathSeparator",
        returns_value: true,
        handler: fs::os_is_path_separator,
    },
    StdlibFunction {
        id: OS_IS_PERMISSION,
        symbol: "IsPermission",
        returns_value: true,
        handler: error::os_is_permission,
    },
    StdlibFunction {
        id: OS_SAME_FILE,
        symbol: "SameFile",
        returns_value: true,
        handler: fs::os_same_file,
    },
    StdlibFunction {
        id: OS_IS_TIMEOUT,
        symbol: "IsTimeout",
        returns_value: true,
        handler: error::os_is_timeout,
    },
    StdlibFunction {
        id: OS_NEW_SYSCALL_ERROR,
        symbol: "NewSyscallError",
        returns_value: true,
        handler: error::os_new_syscall_error,
    },
    StdlibFunction {
        id: OS_TEMP_DIR,
        symbol: "TempDir",
        returns_value: true,
        handler: env::os_temp_dir,
    },
    StdlibFunction {
        id: OS_USER_HOME_DIR,
        symbol: "UserHomeDir",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_USER_CACHE_DIR,
        symbol: "UserCacheDir",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_USER_CONFIG_DIR,
        symbol: "UserConfigDir",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_HOSTNAME,
        symbol: "Hostname",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_EXECUTABLE,
        symbol: "Executable",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: OS_GETUID,
        symbol: "Getuid",
        returns_value: true,
        handler: env::os_getuid,
    },
    StdlibFunction {
        id: OS_GETEUID,
        symbol: "Geteuid",
        returns_value: true,
        handler: env::os_geteuid,
    },
    StdlibFunction {
        id: OS_GETGID,
        symbol: "Getgid",
        returns_value: true,
        handler: env::os_getgid,
    },
    StdlibFunction {
        id: OS_GETEGID,
        symbol: "Getegid",
        returns_value: true,
        handler: env::os_getegid,
    },
    StdlibFunction {
        id: OS_GETPID,
        symbol: "Getpid",
        returns_value: true,
        handler: env::os_getpid,
    },
    StdlibFunction {
        id: OS_GETPPID,
        symbol: "Getppid",
        returns_value: true,
        handler: env::os_getppid,
    },
    StdlibFunction {
        id: OS_GETPAGESIZE,
        symbol: "Getpagesize",
        returns_value: true,
        handler: env::os_getpagesize,
    },
    StdlibFunction {
        id: OS_GETGROUPS,
        symbol: "Getgroups",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];

fn os_exit(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let code = args
        .first()
        .and_then(|v| match &v.data {
            ValueData::Int(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(0);
    Err(VmError::ProgramExit { code })
}

pub(super) fn os_lookup_env(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_lookup_env(vm, program, args)
}

pub(super) fn os_read_file(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    fs::os_read_file(vm, program, args)
}

pub(super) fn os_read_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    fs::os_read_dir(vm, program, args)
}

pub(super) fn os_stat(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    fs::os_stat(vm, program, args)
}

pub(super) fn os_lstat(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    fs::os_lstat(vm, program, args)
}

pub(super) fn os_getwd(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    fs::os_getwd(vm, program, args)
}

pub(super) fn os_getgroups(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_getgroups(vm, program, args)
}

pub(super) fn os_user_home_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_user_home_dir(vm, program, args)
}

pub(super) fn os_user_cache_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_user_cache_dir(vm, program, args)
}

pub(super) fn os_user_config_dir(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_user_config_dir(vm, program, args)
}

pub(super) fn os_hostname(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_hostname(vm, program, args)
}

pub(super) fn os_executable(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    env::os_executable(vm, program, args)
}
