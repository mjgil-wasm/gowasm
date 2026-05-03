use super::{
    StdlibFunctionId, IO_FS_DIR_ENTRY_INFO, IO_FS_DIR_ENTRY_IS_DIR, IO_FS_DIR_ENTRY_NAME,
    IO_FS_DIR_ENTRY_TYPE, IO_FS_FILE_CLOSE, IO_FS_FILE_INFO_IS_DIR, IO_FS_FILE_INFO_MODE,
    IO_FS_FILE_INFO_MOD_TIME, IO_FS_FILE_INFO_NAME, IO_FS_FILE_INFO_SIZE, IO_FS_FILE_INFO_SYS,
    IO_FS_FILE_INFO_TO_DIR_ENTRY, IO_FS_FILE_MODE_IS_DIR, IO_FS_FILE_MODE_IS_REGULAR,
    IO_FS_FILE_MODE_PERM, IO_FS_FILE_MODE_STRING, IO_FS_FILE_MODE_TYPE, IO_FS_FILE_READ,
    IO_FS_FILE_STAT, IO_FS_FORMAT_DIR_ENTRY, IO_FS_FORMAT_FILE_INFO, IO_FS_FS_OPEN, IO_FS_GLOB,
    IO_FS_READ_DIR, IO_FS_READ_DIR_FILE_READ_DIR, IO_FS_READ_FILE, IO_FS_STAT, IO_FS_SUB,
    IO_FS_VALID_PATH, IO_FS_WALK_DIR, JSON_MARSHAL, JSON_MARSHAL_INDENT, JSON_UNMARSHAL,
    JSON_VALID, NET_HTTP_CANONICAL_HEADER_KEY, NET_HTTP_CLIENT_DO, NET_HTTP_CLIENT_GET,
    NET_HTTP_CLIENT_HEAD, NET_HTTP_CLIENT_POST, NET_HTTP_CLIENT_POST_FORM,
    NET_HTTP_DETECT_CONTENT_TYPE, NET_HTTP_GET, NET_HTTP_HEAD, NET_HTTP_HEADER_ADD,
    NET_HTTP_HEADER_CLONE, NET_HTTP_HEADER_DEL, NET_HTTP_HEADER_GET, NET_HTTP_HEADER_SET,
    NET_HTTP_HEADER_VALUES, NET_HTTP_NEW_REQUEST, NET_HTTP_NEW_REQUEST_WITH_CONTEXT,
    NET_HTTP_PARSE_HTTP_VERSION, NET_HTTP_PARSE_TIME, NET_HTTP_POST, NET_HTTP_POST_FORM,
    NET_HTTP_REQUEST_BODY_READ, NET_HTTP_REQUEST_CLONE, NET_HTTP_REQUEST_CONTEXT,
    NET_HTTP_REQUEST_WITH_CONTEXT, NET_HTTP_RESPONSE_BODY_CLOSE, NET_HTTP_RESPONSE_BODY_READ,
    NET_HTTP_RESPONSE_LOCATION, NET_HTTP_STATUS_TEXT, NET_URL_JOIN_PATH, NET_URL_PARSE,
    NET_URL_PARSE_QUERY, NET_URL_PARSE_REQUEST_URI, NET_URL_PATH_ESCAPE, NET_URL_PATH_UNESCAPE,
    NET_URL_QUERY_ESCAPE, NET_URL_QUERY_UNESCAPE, NET_URL_URL_ESCAPED_FRAGMENT,
    NET_URL_URL_ESCAPED_PATH, NET_URL_URL_HOSTNAME, NET_URL_URL_IS_ABS, NET_URL_URL_JOIN_PATH,
    NET_URL_URL_MARSHAL_BINARY, NET_URL_URL_PARSE, NET_URL_URL_PORT, NET_URL_URL_QUERY,
    NET_URL_URL_REDACTED, NET_URL_URL_REQUEST_URI, NET_URL_URL_RESOLVE_REFERENCE,
    NET_URL_URL_STRING, NET_URL_URL_UNMARSHAL_BINARY, NET_URL_USER, NET_URL_USERINFO_PASSWORD,
    NET_URL_USERINFO_STRING, NET_URL_USERINFO_USERNAME, NET_URL_USER_PASSWORD, NET_URL_VALUES_ADD,
    NET_URL_VALUES_DEL, NET_URL_VALUES_ENCODE, NET_URL_VALUES_GET, NET_URL_VALUES_HAS,
    NET_URL_VALUES_SET, OS_CLEARENV, OS_DIR_FS, OS_ENVIRON, OS_EXECUTABLE, OS_EXIT, OS_EXPAND,
    OS_EXPAND_ENV, OS_GETEGID, OS_GETENV, OS_GETEUID, OS_GETGID, OS_GETGROUPS, OS_GETPAGESIZE,
    OS_GETPID, OS_GETPPID, OS_GETUID, OS_GETWD, OS_HOSTNAME, OS_IS_EXIST, OS_IS_NOT_EXIST,
    OS_IS_PATH_SEPARATOR, OS_IS_PERMISSION, OS_IS_TIMEOUT, OS_LOOKUP_ENV, OS_LSTAT, OS_MKDIR_ALL,
    OS_NEW_SYSCALL_ERROR, OS_READ_DIR, OS_READ_FILE, OS_REMOVE_ALL, OS_SAME_FILE, OS_SETENV,
    OS_STAT, OS_TEMP_DIR, OS_UNSETENV, OS_USER_CACHE_DIR, OS_USER_CONFIG_DIR, OS_USER_HOME_DIR,
    OS_WRITE_FILE, TIME_AFTER, TIME_DURATION_HOURS, TIME_DURATION_MICROSECONDS,
    TIME_DURATION_MILLISECONDS, TIME_DURATION_MINUTES, TIME_DURATION_NANOSECONDS,
    TIME_DURATION_SECONDS, TIME_NEW_TIMER, TIME_NOW, TIME_PARSE, TIME_SINCE, TIME_SLEEP,
    TIME_TIMER_RESET, TIME_TIMER_STOP, TIME_TIME_ADD, TIME_TIME_AFTER, TIME_TIME_BEFORE,
    TIME_TIME_COMPARE, TIME_TIME_EQUAL, TIME_TIME_FORMAT, TIME_TIME_IS_ZERO, TIME_TIME_SUB,
    TIME_TIME_UNIX, TIME_TIME_UNIX_MICRO, TIME_TIME_UNIX_MILLI, TIME_TIME_UNIX_NANO, TIME_UNIX,
    TIME_UNIX_MICRO, TIME_UNIX_MILLI, TIME_UNTIL,
};

pub(super) fn stdlib_function_param_types_host(
    function: StdlibFunctionId,
) -> Option<&'static [&'static str]> {
    match function {
        NET_HTTP_CANONICAL_HEADER_KEY => Some(&["string"]),
        NET_HTTP_DETECT_CONTENT_TYPE => Some(&["[]byte"]),
        NET_HTTP_HEADER_CLONE => Some(&["http.Header"]),
        NET_HTTP_HEADER_GET | NET_HTTP_HEADER_VALUES | NET_HTTP_HEADER_DEL => {
            Some(&["http.Header", "string"])
        }
        NET_HTTP_HEADER_SET | NET_HTTP_HEADER_ADD => Some(&["http.Header", "string", "string"]),
        NET_HTTP_GET => Some(&["string"]),
        NET_HTTP_HEAD => Some(&["string"]),
        NET_HTTP_POST => Some(&["string", "string", "io.Reader"]),
        NET_HTTP_POST_FORM => Some(&["string", "url.Values"]),
        NET_HTTP_CLIENT_DO => Some(&["*http.Client", "*http.Request"]),
        NET_HTTP_CLIENT_GET | NET_HTTP_CLIENT_HEAD => Some(&["*http.Client", "string"]),
        NET_HTTP_CLIENT_POST => Some(&["*http.Client", "string", "string", "io.Reader"]),
        NET_HTTP_CLIENT_POST_FORM => Some(&["*http.Client", "string", "url.Values"]),
        NET_HTTP_NEW_REQUEST => Some(&["string", "string", "io.Reader"]),
        NET_HTTP_NEW_REQUEST_WITH_CONTEXT => {
            Some(&["context.Context", "string", "string", "io.Reader"])
        }
        NET_HTTP_REQUEST_CLONE => Some(&["*http.Request", "context.Context"]),
        NET_HTTP_REQUEST_CONTEXT => Some(&["*http.Request"]),
        NET_HTTP_REQUEST_BODY_READ => Some(&["http.__requestBody", "[]byte"]),
        NET_HTTP_REQUEST_WITH_CONTEXT => Some(&["*http.Request", "context.Context"]),
        NET_HTTP_RESPONSE_LOCATION => Some(&["*http.Response"]),
        NET_HTTP_RESPONSE_BODY_CLOSE => Some(&["http.__responseBody"]),
        NET_HTTP_RESPONSE_BODY_READ => Some(&["http.__responseBody", "[]byte"]),
        NET_HTTP_PARSE_HTTP_VERSION => Some(&["string"]),
        NET_HTTP_PARSE_TIME => Some(&["string"]),
        NET_HTTP_STATUS_TEXT => Some(&["int"]),
        NET_URL_USER => Some(&["string"]),
        NET_URL_USER_PASSWORD => Some(&["string", "string"]),
        NET_URL_JOIN_PATH => Some(&["string"]),
        NET_URL_PARSE | NET_URL_PARSE_REQUEST_URI => Some(&["string"]),
        NET_URL_PARSE_QUERY
        | NET_URL_PATH_ESCAPE
        | NET_URL_PATH_UNESCAPE
        | NET_URL_QUERY_ESCAPE
        | NET_URL_QUERY_UNESCAPE => Some(&["string"]),
        NET_URL_USERINFO_STRING | NET_URL_USERINFO_USERNAME | NET_URL_USERINFO_PASSWORD => {
            Some(&["*url.Userinfo"])
        }
        NET_URL_URL_ESCAPED_FRAGMENT
        | NET_URL_URL_ESCAPED_PATH
        | NET_URL_URL_HOSTNAME
        | NET_URL_URL_IS_ABS
        | NET_URL_URL_MARSHAL_BINARY
        | NET_URL_URL_PORT
        | NET_URL_URL_QUERY
        | NET_URL_URL_REDACTED
        | NET_URL_URL_REQUEST_URI => Some(&["*url.URL"]),
        NET_URL_URL_JOIN_PATH => Some(&["*url.URL"]),
        NET_URL_URL_PARSE => Some(&["*url.URL", "string"]),
        NET_URL_URL_RESOLVE_REFERENCE => Some(&["*url.URL", "*url.URL"]),
        NET_URL_URL_UNMARSHAL_BINARY => Some(&["*url.URL", "[]byte"]),
        NET_URL_URL_STRING => Some(&["url.URL"]),
        NET_URL_VALUES_ENCODE => Some(&["url.Values"]),
        NET_URL_VALUES_GET => Some(&["url.Values", "string"]),
        NET_URL_VALUES_HAS => Some(&["url.Values", "string"]),
        NET_URL_VALUES_SET | NET_URL_VALUES_ADD => Some(&["url.Values", "string", "string"]),
        NET_URL_VALUES_DEL => Some(&["url.Values", "string"]),
        OS_EXIT => Some(&["int"]),
        OS_GETENV | OS_LOOKUP_ENV | OS_UNSETENV => Some(&["string"]),
        OS_SETENV => Some(&["string", "string"]),
        OS_DIR_FS => Some(&["string"]),
        OS_READ_FILE => Some(&["string"]),
        OS_WRITE_FILE => Some(&["string", "[]byte", "fs.FileMode"]),
        OS_READ_DIR => Some(&["string"]),
        OS_STAT => Some(&["string"]),
        OS_LSTAT => Some(&["string"]),
        OS_MKDIR_ALL => Some(&["string", "fs.FileMode"]),
        OS_REMOVE_ALL => Some(&["string"]),
        OS_GETWD => Some(&[]),
        OS_GETUID | OS_GETEUID | OS_GETGID | OS_GETEGID | OS_GETPID | OS_GETPPID
        | OS_GETPAGESIZE => Some(&[]),
        OS_GETGROUPS => Some(&[]),
        OS_HOSTNAME | OS_EXECUTABLE => Some(&[]),
        OS_TEMP_DIR => Some(&[]),
        OS_USER_HOME_DIR | OS_USER_CACHE_DIR | OS_USER_CONFIG_DIR => Some(&[]),
        OS_IS_EXIST => Some(&["error"]),
        OS_IS_NOT_EXIST => Some(&["error"]),
        OS_IS_PATH_SEPARATOR => Some(&["byte"]),
        OS_IS_PERMISSION => Some(&["error"]),
        OS_IS_TIMEOUT => Some(&["error"]),
        OS_NEW_SYSCALL_ERROR => Some(&["string", "error"]),
        OS_SAME_FILE => Some(&["fs.FileInfo", "fs.FileInfo"]),
        OS_ENVIRON => Some(&[]),
        OS_EXPAND_ENV => Some(&["string"]),
        OS_EXPAND => Some(&["string", "__gowasm_func__(string)->(string)"]),
        OS_CLEARENV => Some(&[]),
        IO_FS_VALID_PATH => Some(&["string"]),
        IO_FS_FS_OPEN => Some(&["fs.FS", "string"]),
        IO_FS_FILE_CLOSE => Some(&["fs.File"]),
        IO_FS_FILE_STAT => Some(&["fs.File"]),
        IO_FS_FILE_READ => Some(&["fs.File", "[]byte"]),
        IO_FS_READ_DIR_FILE_READ_DIR => Some(&["fs.ReadDirFile", "int"]),
        IO_FS_FILE_INFO_NAME => Some(&["fs.FileInfo"]),
        IO_FS_FILE_INFO_IS_DIR => Some(&["fs.FileInfo"]),
        IO_FS_FILE_INFO_SIZE => Some(&["fs.FileInfo"]),
        IO_FS_FILE_INFO_MODE => Some(&["fs.FileInfo"]),
        IO_FS_FILE_INFO_MOD_TIME => Some(&["fs.FileInfo"]),
        IO_FS_FILE_INFO_SYS => Some(&["fs.FileInfo"]),
        IO_FS_FILE_INFO_TO_DIR_ENTRY => Some(&["fs.FileInfo"]),
        IO_FS_FILE_MODE_IS_DIR => Some(&["fs.FileMode"]),
        IO_FS_FILE_MODE_IS_REGULAR => Some(&["fs.FileMode"]),
        IO_FS_FILE_MODE_PERM => Some(&["fs.FileMode"]),
        IO_FS_FILE_MODE_STRING => Some(&["fs.FileMode"]),
        IO_FS_FILE_MODE_TYPE => Some(&["fs.FileMode"]),
        IO_FS_DIR_ENTRY_NAME => Some(&["fs.DirEntry"]),
        IO_FS_DIR_ENTRY_IS_DIR => Some(&["fs.DirEntry"]),
        IO_FS_DIR_ENTRY_TYPE => Some(&["fs.DirEntry"]),
        IO_FS_DIR_ENTRY_INFO => Some(&["fs.DirEntry"]),
        IO_FS_FORMAT_DIR_ENTRY => Some(&["fs.DirEntry"]),
        IO_FS_FORMAT_FILE_INFO => Some(&["fs.FileInfo"]),
        IO_FS_READ_FILE => Some(&["fs.FS", "string"]),
        IO_FS_READ_DIR => Some(&["fs.FS", "string"]),
        IO_FS_STAT => Some(&["fs.FS", "string"]),
        IO_FS_SUB => Some(&["fs.FS", "string"]),
        IO_FS_GLOB => Some(&["fs.FS", "string"]),
        IO_FS_WALK_DIR => Some(&[
            "fs.FS",
            "string",
            "__gowasm_func__(string, fs.DirEntry, error)->(error)",
        ]),
        JSON_MARSHAL => Some(&["interface{}"]),
        JSON_MARSHAL_INDENT => Some(&["interface{}", "string", "string"]),
        JSON_UNMARSHAL => Some(&["[]byte", "interface{}"]),
        JSON_VALID => Some(&["[]byte"]),
        TIME_NOW => Some(&[]),
        TIME_SLEEP | TIME_AFTER | TIME_NEW_TIMER => Some(&["time.Duration"]),
        TIME_UNIX => Some(&["int", "int"]),
        TIME_UNIX_MILLI => Some(&["int"]),
        TIME_UNIX_MICRO => Some(&["int"]),
        TIME_PARSE => Some(&["string", "string"]),
        TIME_SINCE | TIME_UNTIL => Some(&["time.Time"]),
        TIME_TIME_UNIX | TIME_TIME_UNIX_MILLI | TIME_TIME_UNIX_MICRO | TIME_TIME_UNIX_NANO
        | TIME_TIME_IS_ZERO => Some(&["time.Time"]),
        TIME_TIME_FORMAT => Some(&["time.Time", "string"]),
        TIME_TIME_BEFORE | TIME_TIME_AFTER | TIME_TIME_EQUAL | TIME_TIME_COMPARE => {
            Some(&["time.Time", "time.Time"])
        }
        TIME_TIME_ADD => Some(&["time.Time", "time.Duration"]),
        TIME_TIME_SUB => Some(&["time.Time", "time.Time"]),
        TIME_TIMER_STOP => Some(&["*time.Timer"]),
        TIME_TIMER_RESET => Some(&["*time.Timer", "time.Duration"]),
        TIME_DURATION_NANOSECONDS
        | TIME_DURATION_MICROSECONDS
        | TIME_DURATION_MILLISECONDS
        | TIME_DURATION_SECONDS
        | TIME_DURATION_MINUTES
        | TIME_DURATION_HOURS => Some(&["time.Duration"]),
        _ => None,
    }
}

pub(super) fn stdlib_function_result_types_host(
    function: StdlibFunctionId,
) -> Option<&'static [&'static str]> {
    match function {
        NET_HTTP_CANONICAL_HEADER_KEY
        | NET_HTTP_STATUS_TEXT
        | NET_HTTP_DETECT_CONTENT_TYPE
        | NET_HTTP_HEADER_GET => Some(&["string"]),
        NET_HTTP_HEADER_CLONE => Some(&["http.Header"]),
        NET_HTTP_HEADER_VALUES => Some(&["[]string"]),
        NET_HTTP_GET | NET_HTTP_HEAD | NET_HTTP_POST => Some(&["*http.Response", "error"]),
        NET_HTTP_POST_FORM
        | NET_HTTP_CLIENT_DO
        | NET_HTTP_CLIENT_GET
        | NET_HTTP_CLIENT_HEAD
        | NET_HTTP_CLIENT_POST
        | NET_HTTP_CLIENT_POST_FORM => Some(&["*http.Response", "error"]),
        NET_HTTP_NEW_REQUEST | NET_HTTP_NEW_REQUEST_WITH_CONTEXT => {
            Some(&["*http.Request", "error"])
        }
        NET_HTTP_PARSE_HTTP_VERSION => Some(&["int", "int", "bool"]),
        NET_HTTP_PARSE_TIME => Some(&["time.Time", "error"]),
        NET_HTTP_REQUEST_BODY_READ | NET_HTTP_RESPONSE_BODY_READ => Some(&["int", "error"]),
        NET_HTTP_REQUEST_CLONE | NET_HTTP_REQUEST_WITH_CONTEXT => Some(&["*http.Request"]),
        NET_HTTP_REQUEST_CONTEXT => Some(&["context.Context"]),
        NET_HTTP_RESPONSE_LOCATION => Some(&["*url.URL", "error"]),
        NET_HTTP_RESPONSE_BODY_CLOSE => Some(&["error"]),
        NET_URL_JOIN_PATH => Some(&["string", "error"]),
        NET_URL_PARSE | NET_URL_PARSE_REQUEST_URI => Some(&["*url.URL", "error"]),
        NET_URL_PARSE_QUERY => Some(&["url.Values", "error"]),
        NET_URL_PATH_UNESCAPE | NET_URL_QUERY_UNESCAPE => Some(&["string", "error"]),
        NET_URL_PATH_ESCAPE => Some(&["string"]),
        NET_URL_USER | NET_URL_USER_PASSWORD => Some(&["*url.Userinfo"]),
        NET_URL_QUERY_ESCAPE
        | NET_URL_URL_ESCAPED_FRAGMENT
        | NET_URL_URL_ESCAPED_PATH
        | NET_URL_URL_HOSTNAME
        | NET_URL_URL_PORT
        | NET_URL_URL_REDACTED
        | NET_URL_USERINFO_STRING
        | NET_URL_USERINFO_USERNAME
        | NET_URL_URL_STRING
        | NET_URL_VALUES_ENCODE
        | NET_URL_VALUES_GET => Some(&["string"]),
        NET_URL_URL_JOIN_PATH => Some(&["*url.URL"]),
        NET_URL_URL_PARSE => Some(&["*url.URL", "error"]),
        NET_URL_URL_QUERY => Some(&["url.Values"]),
        NET_URL_URL_RESOLVE_REFERENCE => Some(&["*url.URL"]),
        NET_URL_USERINFO_PASSWORD => Some(&["string", "bool"]),
        NET_URL_VALUES_HAS | NET_URL_URL_IS_ABS => Some(&["bool"]),
        NET_URL_URL_MARSHAL_BINARY => Some(&["[]byte", "error"]),
        NET_URL_URL_UNMARSHAL_BINARY => Some(&["error"]),
        NET_URL_URL_REQUEST_URI => Some(&["string"]),
        OS_MKDIR_ALL | OS_REMOVE_ALL | OS_WRITE_FILE => Some(&["error"]),
        OS_GETENV | OS_TEMP_DIR | OS_EXPAND_ENV | OS_EXPAND => Some(&["string"]),
        OS_LOOKUP_ENV => Some(&["string", "bool"]),
        OS_DIR_FS => Some(&["fs.FS"]),
        OS_READ_FILE => Some(&["[]byte", "error"]),
        OS_READ_DIR => Some(&["[]fs.DirEntry", "error"]),
        OS_STAT | OS_LSTAT => Some(&["fs.FileInfo", "error"]),
        OS_GETWD => Some(&["string", "error"]),
        OS_GETUID | OS_GETEUID | OS_GETGID | OS_GETEGID | OS_GETPID | OS_GETPPID
        | OS_GETPAGESIZE => Some(&["int"]),
        OS_GETGROUPS => Some(&["[]int", "error"]),
        OS_USER_HOME_DIR | OS_USER_CACHE_DIR | OS_USER_CONFIG_DIR | OS_HOSTNAME | OS_EXECUTABLE => {
            Some(&["string", "error"])
        }
        OS_IS_EXIST | OS_IS_NOT_EXIST | OS_IS_PATH_SEPARATOR | OS_IS_PERMISSION | OS_IS_TIMEOUT
        | OS_SAME_FILE => Some(&["bool"]),
        OS_NEW_SYSCALL_ERROR => Some(&["error"]),
        OS_ENVIRON => Some(&["[]string"]),
        IO_FS_VALID_PATH => Some(&["bool"]),
        IO_FS_FS_OPEN => Some(&["fs.File", "error"]),
        IO_FS_FILE_CLOSE => Some(&["error"]),
        IO_FS_FILE_STAT | IO_FS_DIR_ENTRY_INFO => Some(&["fs.FileInfo", "error"]),
        IO_FS_FILE_READ => Some(&["int", "error"]),
        IO_FS_READ_DIR_FILE_READ_DIR | IO_FS_READ_DIR => Some(&["[]fs.DirEntry", "error"]),
        IO_FS_FILE_INFO_NAME
        | IO_FS_FILE_MODE_STRING
        | IO_FS_DIR_ENTRY_NAME
        | IO_FS_FORMAT_DIR_ENTRY
        | IO_FS_FORMAT_FILE_INFO => Some(&["string"]),
        IO_FS_FILE_INFO_IS_DIR
        | IO_FS_FILE_MODE_IS_DIR
        | IO_FS_FILE_MODE_IS_REGULAR
        | IO_FS_DIR_ENTRY_IS_DIR => Some(&["bool"]),
        IO_FS_FILE_INFO_SIZE => Some(&["int"]),
        IO_FS_FILE_INFO_MODE | IO_FS_FILE_MODE_PERM | IO_FS_FILE_MODE_TYPE
        | IO_FS_DIR_ENTRY_TYPE => Some(&["fs.FileMode"]),
        IO_FS_FILE_INFO_MOD_TIME => Some(&["time.Time"]),
        IO_FS_FILE_INFO_SYS => Some(&["interface{}"]),
        IO_FS_FILE_INFO_TO_DIR_ENTRY => Some(&["fs.DirEntry"]),
        IO_FS_READ_FILE => Some(&["[]byte", "error"]),
        IO_FS_STAT => Some(&["fs.FileInfo", "error"]),
        IO_FS_SUB => Some(&["fs.FS", "error"]),
        IO_FS_GLOB => Some(&["[]string", "error"]),
        IO_FS_WALK_DIR => Some(&["error"]),
        JSON_MARSHAL | JSON_MARSHAL_INDENT => Some(&["[]byte", "error"]),
        JSON_UNMARSHAL => Some(&["error"]),
        JSON_VALID => Some(&["bool"]),
        TIME_NOW | TIME_UNIX | TIME_UNIX_MILLI | TIME_UNIX_MICRO | TIME_TIME_ADD => {
            Some(&["time.Time"])
        }
        TIME_PARSE => Some(&["time.Time", "error"]),
        TIME_SLEEP => Some(&[]),
        TIME_AFTER => Some(&["<-chan time.Time"]),
        TIME_NEW_TIMER => Some(&["*time.Timer"]),
        TIME_TIMER_STOP | TIME_TIMER_RESET => Some(&["bool"]),
        TIME_TIME_FORMAT => Some(&["string"]),
        TIME_SINCE | TIME_UNTIL | TIME_TIME_SUB => Some(&["time.Duration"]),
        TIME_TIME_UNIX | TIME_TIME_UNIX_MILLI | TIME_TIME_UNIX_MICRO | TIME_TIME_UNIX_NANO => {
            Some(&["int64"])
        }
        TIME_TIME_BEFORE | TIME_TIME_AFTER | TIME_TIME_EQUAL | TIME_TIME_IS_ZERO => Some(&["bool"]),
        TIME_TIME_COMPARE => Some(&["int"]),
        TIME_DURATION_NANOSECONDS | TIME_DURATION_MICROSECONDS | TIME_DURATION_MILLISECONDS => {
            Some(&["int64"])
        }
        TIME_DURATION_SECONDS | TIME_DURATION_MINUTES | TIME_DURATION_HOURS => Some(&["float64"]),
        _ => None,
    }
}

pub(super) fn stdlib_function_variadic_param_type_host(
    function: StdlibFunctionId,
) -> Option<&'static str> {
    match function {
        NET_URL_JOIN_PATH | NET_URL_URL_JOIN_PATH => Some("string"),
        _ => None,
    }
}
