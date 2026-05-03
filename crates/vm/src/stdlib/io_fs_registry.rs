use super::{
    io_fs_impl, io_fs_metadata_impl, io_fs_walk_impl, StdlibConstant, StdlibConstantValue,
    StdlibFunction, StdlibMethod, StdlibValue, StdlibValueInit, IO_FS_DIR_ENTRY_INFO,
    IO_FS_DIR_ENTRY_IS_DIR, IO_FS_DIR_ENTRY_NAME, IO_FS_DIR_ENTRY_TYPE, IO_FS_FILE_CLOSE,
    IO_FS_FILE_INFO_IS_DIR, IO_FS_FILE_INFO_MODE, IO_FS_FILE_INFO_MOD_TIME, IO_FS_FILE_INFO_NAME,
    IO_FS_FILE_INFO_SIZE, IO_FS_FILE_INFO_SYS, IO_FS_FILE_INFO_TO_DIR_ENTRY,
    IO_FS_FILE_MODE_IS_DIR, IO_FS_FILE_MODE_IS_REGULAR, IO_FS_FILE_MODE_PERM,
    IO_FS_FILE_MODE_STRING, IO_FS_FILE_MODE_TYPE, IO_FS_FILE_READ, IO_FS_FILE_STAT,
    IO_FS_FORMAT_DIR_ENTRY, IO_FS_FORMAT_FILE_INFO, IO_FS_FS_OPEN, IO_FS_GLOB, IO_FS_READ_DIR,
    IO_FS_READ_DIR_FILE_READ_DIR, IO_FS_READ_FILE, IO_FS_STAT, IO_FS_SUB, IO_FS_VALID_PATH,
    IO_FS_WALK_DIR,
};

pub(super) const MODE_DIR: i64 = 1 << 31;
pub(super) const MODE_APPEND: i64 = 1 << 30;
pub(super) const MODE_EXCLUSIVE: i64 = 1 << 29;
pub(super) const MODE_TEMPORARY: i64 = 1 << 28;
pub(super) const MODE_SYMLINK: i64 = 1 << 27;
pub(super) const MODE_DEVICE: i64 = 1 << 26;
pub(super) const MODE_NAMED_PIPE: i64 = 1 << 25;
pub(super) const MODE_SOCKET: i64 = 1 << 24;
pub(super) const MODE_SETUID: i64 = 1 << 23;
pub(super) const MODE_SETGID: i64 = 1 << 22;
pub(super) const MODE_CHAR_DEVICE: i64 = 1 << 21;
pub(super) const MODE_STICKY: i64 = 1 << 20;
pub(super) const MODE_IRREGULAR: i64 = 1 << 19;
pub(super) const MODE_TYPE: i64 = MODE_DIR
    | MODE_SYMLINK
    | MODE_NAMED_PIPE
    | MODE_SOCKET
    | MODE_DEVICE
    | MODE_CHAR_DEVICE
    | MODE_IRREGULAR;
pub(super) const MODE_PERM: i64 = 0o777;
pub(super) const SKIP_DIR: &str = "skip this directory";
pub(super) const SKIP_ALL: &str = "skip everything and stop the walk";

pub(super) const IO_FS_CONSTANTS: &[StdlibConstant] = &[
    StdlibConstant {
        symbol: "ModeDir",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_DIR),
    },
    StdlibConstant {
        symbol: "ModeAppend",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_APPEND),
    },
    StdlibConstant {
        symbol: "ModeExclusive",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_EXCLUSIVE),
    },
    StdlibConstant {
        symbol: "ModeTemporary",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_TEMPORARY),
    },
    StdlibConstant {
        symbol: "ModeSymlink",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_SYMLINK),
    },
    StdlibConstant {
        symbol: "ModeDevice",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_DEVICE),
    },
    StdlibConstant {
        symbol: "ModeNamedPipe",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_NAMED_PIPE),
    },
    StdlibConstant {
        symbol: "ModeSocket",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_SOCKET),
    },
    StdlibConstant {
        symbol: "ModeSetuid",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_SETUID),
    },
    StdlibConstant {
        symbol: "ModeSetgid",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_SETGID),
    },
    StdlibConstant {
        symbol: "ModeCharDevice",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_CHAR_DEVICE),
    },
    StdlibConstant {
        symbol: "ModeSticky",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_STICKY),
    },
    StdlibConstant {
        symbol: "ModeIrregular",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_IRREGULAR),
    },
    StdlibConstant {
        symbol: "ModeType",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_TYPE),
    },
    StdlibConstant {
        symbol: "ModePerm",
        typ: "fs.FileMode",
        value: StdlibConstantValue::Int(MODE_PERM),
    },
];

pub(super) const IO_FS_VALUES: &[StdlibValue] = &[
    StdlibValue {
        symbol: "SkipDir",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(SKIP_DIR)),
    },
    StdlibValue {
        symbol: "SkipAll",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(SKIP_ALL)),
    },
];

pub(super) const IO_FS_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "fs.FS",
        method: "Open",
        function: IO_FS_FS_OPEN,
    },
    StdlibMethod {
        receiver_type: "fs.File",
        method: "Close",
        function: IO_FS_FILE_CLOSE,
    },
    StdlibMethod {
        receiver_type: "fs.File",
        method: "Stat",
        function: IO_FS_FILE_STAT,
    },
    StdlibMethod {
        receiver_type: "fs.File",
        method: "Read",
        function: IO_FS_FILE_READ,
    },
    StdlibMethod {
        receiver_type: "fs.ReadDirFile",
        method: "ReadDir",
        function: IO_FS_READ_DIR_FILE_READ_DIR,
    },
    StdlibMethod {
        receiver_type: "fs.FileInfo",
        method: "Name",
        function: IO_FS_FILE_INFO_NAME,
    },
    StdlibMethod {
        receiver_type: "fs.FileInfo",
        method: "IsDir",
        function: IO_FS_FILE_INFO_IS_DIR,
    },
    StdlibMethod {
        receiver_type: "fs.FileInfo",
        method: "Size",
        function: IO_FS_FILE_INFO_SIZE,
    },
    StdlibMethod {
        receiver_type: "fs.FileInfo",
        method: "Mode",
        function: IO_FS_FILE_INFO_MODE,
    },
    StdlibMethod {
        receiver_type: "fs.FileInfo",
        method: "ModTime",
        function: IO_FS_FILE_INFO_MOD_TIME,
    },
    StdlibMethod {
        receiver_type: "fs.FileInfo",
        method: "Sys",
        function: IO_FS_FILE_INFO_SYS,
    },
    StdlibMethod {
        receiver_type: "fs.FileMode",
        method: "IsDir",
        function: IO_FS_FILE_MODE_IS_DIR,
    },
    StdlibMethod {
        receiver_type: "fs.FileMode",
        method: "IsRegular",
        function: IO_FS_FILE_MODE_IS_REGULAR,
    },
    StdlibMethod {
        receiver_type: "fs.FileMode",
        method: "Type",
        function: IO_FS_FILE_MODE_TYPE,
    },
    StdlibMethod {
        receiver_type: "fs.FileMode",
        method: "String",
        function: IO_FS_FILE_MODE_STRING,
    },
    StdlibMethod {
        receiver_type: "fs.FileMode",
        method: "Perm",
        function: IO_FS_FILE_MODE_PERM,
    },
    StdlibMethod {
        receiver_type: "fs.DirEntry",
        method: "Name",
        function: IO_FS_DIR_ENTRY_NAME,
    },
    StdlibMethod {
        receiver_type: "fs.DirEntry",
        method: "IsDir",
        function: IO_FS_DIR_ENTRY_IS_DIR,
    },
    StdlibMethod {
        receiver_type: "fs.DirEntry",
        method: "Type",
        function: IO_FS_DIR_ENTRY_TYPE,
    },
    StdlibMethod {
        receiver_type: "fs.DirEntry",
        method: "Info",
        function: IO_FS_DIR_ENTRY_INFO,
    },
];

pub(super) const IO_FS_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: IO_FS_VALID_PATH,
        symbol: "ValidPath",
        returns_value: true,
        handler: io_fs_impl::io_fs_valid_path,
    },
    StdlibFunction {
        id: IO_FS_READ_FILE,
        symbol: "ReadFile",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_STAT,
        symbol: "Stat",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_SUB,
        symbol: "Sub",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_GLOB,
        symbol: "Glob",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_READ_DIR,
        symbol: "ReadDir",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_WALK_DIR,
        symbol: "WalkDir",
        returns_value: true,
        handler: io_fs_impl::io_fs_walk_dir,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_TO_DIR_ENTRY,
        symbol: "FileInfoToDirEntry",
        returns_value: true,
        handler: io_fs_walk_impl::io_fs_file_info_to_dir_entry,
    },
    StdlibFunction {
        id: IO_FS_FORMAT_DIR_ENTRY,
        symbol: "FormatDirEntry",
        returns_value: true,
        handler: io_fs_walk_impl::io_fs_format_dir_entry,
    },
    StdlibFunction {
        id: IO_FS_FORMAT_FILE_INFO,
        symbol: "FormatFileInfo",
        returns_value: true,
        handler: io_fs_walk_impl::io_fs_format_file_info,
    },
];

pub(super) const IO_FS_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: IO_FS_FS_OPEN,
        symbol: "Open",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_FILE_CLOSE,
        symbol: "Close",
        returns_value: true,
        handler: io_fs_impl::io_fs_file_close,
    },
    StdlibFunction {
        id: IO_FS_FILE_STAT,
        symbol: "Stat",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_FILE_READ,
        symbol: "Read",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_READ_DIR_FILE_READ_DIR,
        symbol: "ReadDir",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_NAME,
        symbol: "Name",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_info_name,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_IS_DIR,
        symbol: "IsDir",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_info_is_dir,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_SIZE,
        symbol: "Size",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_info_size,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_MODE,
        symbol: "Mode",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_info_mode,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_MOD_TIME,
        symbol: "ModTime",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_info_mod_time,
    },
    StdlibFunction {
        id: IO_FS_FILE_INFO_SYS,
        symbol: "Sys",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_info_sys,
    },
    StdlibFunction {
        id: IO_FS_FILE_MODE_IS_DIR,
        symbol: "IsDir",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_mode_is_dir,
    },
    StdlibFunction {
        id: IO_FS_FILE_MODE_IS_REGULAR,
        symbol: "IsRegular",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_mode_is_regular,
    },
    StdlibFunction {
        id: IO_FS_FILE_MODE_TYPE,
        symbol: "Type",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_mode_type,
    },
    StdlibFunction {
        id: IO_FS_FILE_MODE_STRING,
        symbol: "String",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_mode_string,
    },
    StdlibFunction {
        id: IO_FS_FILE_MODE_PERM,
        symbol: "Perm",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_file_mode_perm,
    },
    StdlibFunction {
        id: IO_FS_DIR_ENTRY_NAME,
        symbol: "Name",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_dir_entry_name,
    },
    StdlibFunction {
        id: IO_FS_DIR_ENTRY_IS_DIR,
        symbol: "IsDir",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_dir_entry_is_dir,
    },
    StdlibFunction {
        id: IO_FS_DIR_ENTRY_TYPE,
        symbol: "Type",
        returns_value: true,
        handler: io_fs_metadata_impl::io_fs_dir_entry_type,
    },
    StdlibFunction {
        id: IO_FS_DIR_ENTRY_INFO,
        symbol: "Info",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];
