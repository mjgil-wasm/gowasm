use super::{Program, Value, Vm, VmError};

#[path = "stdlib/base64.rs"]
mod base64_impl;
#[path = "stdlib/builtins.rs"]
mod builtins_impl;
#[path = "stdlib/bytes_callback.rs"]
mod bytes_callback_impl;
#[path = "stdlib/bytes.rs"]
mod bytes_impl;
#[path = "stdlib/bytes_more.rs"]
mod bytes_more_impl;
#[path = "stdlib/bytes_split.rs"]
mod bytes_split_impl;
#[path = "stdlib/bytes_utf8.rs"]
mod bytes_utf8_impl;
#[path = "stdlib/cmp.rs"]
mod cmp_impl;
#[path = "stdlib/context.rs"]
mod context_impl;
#[path = "stdlib/errors.rs"]
mod errors_impl;
#[path = "stdlib/filepath/mod.rs"]
mod filepath_impl;
#[path = "stdlib/fmt.rs"]
mod fmt_impl;
#[path = "stdlib/hex.rs"]
mod hex_impl;
#[path = "stdlib/io_fs_fallback.rs"]
mod io_fs_fallback_impl;
#[path = "stdlib/io_fs.rs"]
mod io_fs_impl;
#[path = "stdlib/io_fs_metadata.rs"]
mod io_fs_metadata_impl;
#[path = "stdlib/io_fs_registry.rs"]
mod io_fs_registry_impl;
#[path = "stdlib/io_fs_walk.rs"]
mod io_fs_walk_impl;
#[path = "stdlib/json_decode.rs"]
mod json_decode_impl;
#[path = "stdlib/json.rs"]
mod json_impl;
#[path = "stdlib/json_tags.rs"]
mod json_tags_impl;
#[path = "stdlib/log.rs"]
mod log_impl;
#[path = "stdlib/maps.rs"]
mod maps_impl;
#[path = "stdlib/math_bits.rs"]
mod math_bits_impl;
#[path = "stdlib/math.rs"]
mod math_impl;
#[path = "stdlib/md5.rs"]
mod md5_impl;
#[path = "stdlib/net_http/mod.rs"]
mod net_http_impl;
#[path = "stdlib/net_url.rs"]
mod net_url_impl;
#[path = "stdlib/os/mod.rs"]
mod os_impl;
#[path = "stdlib/package_registry.rs"]
mod package_registry_impl;
#[path = "stdlib/package_values.rs"]
mod package_values_impl;
#[path = "stdlib/path.rs"]
mod path_impl;
#[path = "stdlib/rand.rs"]
mod rand_impl;
#[path = "stdlib/reflect.rs"]
mod reflect_impl;
#[path = "stdlib/regexp.rs"]
mod regexp_impl;
#[path = "stdlib/runtime_methods.rs"]
mod runtime_methods_impl;
#[path = "stdlib/sha1.rs"]
mod sha1_impl;
#[path = "stdlib/sha256.rs"]
mod sha256_impl;
#[path = "stdlib/sha512.rs"]
mod sha512_impl;
#[path = "stdlib/signatures.rs"]
mod signatures_impl;
#[path = "stdlib/slices.rs"]
mod slices_impl;
#[path = "stdlib/sort.rs"]
mod sort_impl;
#[path = "stdlib/strconv_helpers.rs"]
mod strconv_helpers_impl;
#[path = "stdlib/strconv.rs"]
mod strconv_impl;
#[path = "stdlib/strings.rs"]
mod strings_impl;
#[path = "stdlib/strings_replacer.rs"]
mod strings_replacer_impl;
#[path = "stdlib/sync.rs"]
mod sync_impl;
#[path = "stdlib/testing.rs"]
mod testing_impl;
#[cfg(test)]
#[path = "stdlib/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "stdlib/tests_base64.rs"]
mod tests_base64;
#[cfg(test)]
#[path = "stdlib/tests_context.rs"]
mod tests_context;
#[cfg(test)]
#[path = "stdlib/tests_io_fs.rs"]
mod tests_io_fs;
#[cfg(test)]
#[path = "stdlib/tests_json.rs"]
mod tests_json;
#[cfg(test)]
#[path = "stdlib/tests_net_http.rs"]
mod tests_net_http;
#[cfg(test)]
#[path = "stdlib/tests_net_url.rs"]
mod tests_net_url;
#[cfg(test)]
#[path = "stdlib/tests_net_url_runtime.rs"]
mod tests_net_url_runtime;
#[cfg(test)]
#[path = "stdlib/tests_net_url_userinfo.rs"]
mod tests_net_url_userinfo;
#[cfg(test)]
#[path = "stdlib/tests_os_errors.rs"]
mod tests_os_errors;
#[cfg(test)]
#[path = "stdlib/tests_os_fs.rs"]
mod tests_os_fs;
#[cfg(test)]
#[path = "stdlib/tests_reflect.rs"]
mod tests_reflect;
#[cfg(test)]
#[path = "stdlib/tests_regexp_methods.rs"]
mod tests_regexp_methods;
#[cfg(test)]
#[path = "stdlib/tests_registry.rs"]
mod tests_registry;
#[cfg(test)]
#[path = "stdlib/tests_strings_replacer.rs"]
mod tests_strings_replacer;
#[cfg(test)]
#[path = "stdlib/tests_sync_methods.rs"]
mod tests_sync_methods;
#[cfg(test)]
#[path = "stdlib/tests_time.rs"]
mod tests_time;
#[path = "stdlib/time_duration.rs"]
mod time_duration_impl;
#[path = "stdlib/time_format.rs"]
mod time_format_impl;
#[path = "stdlib/time/mod.rs"]
mod time_impl;
#[path = "stdlib/time_timer.rs"]
mod time_timer_impl;
#[path = "stdlib/unicode.rs"]
mod unicode_impl;
#[path = "stdlib/unicode_utf8.rs"]
mod unicode_utf8_impl;
#[path = "stdlib/workspace_fs.rs"]
mod workspace_fs_impl;
#[path = "stdlib/workspace_fs_metadata.rs"]
mod workspace_fs_metadata_impl;

pub(crate) use net_http_impl::{request_body_id, request_body_read_into, response_body_read_into};
pub use package_values_impl::resolve_stdlib_value;
pub use runtime_methods_impl::{resolve_stdlib_method, resolve_stdlib_runtime_method};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StdlibFunctionId(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct StdlibFunction {
    pub id: StdlibFunctionId,
    pub symbol: &'static str,
    pub returns_value: bool,
    handler: StdlibHandler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibConstantValue {
    Int(i64),
    Float(super::Float64),
    Bool(bool),
    String(&'static str),
    Error(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdlibConstant {
    pub symbol: &'static str,
    pub typ: &'static str,
    pub value: StdlibConstantValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdlibValue {
    pub symbol: &'static str,
    pub typ: &'static str,
    pub value: StdlibValueInit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibValueInit {
    Constant(StdlibConstantValue),
    NewPointer(&'static str),
    NewPointerWithIntField {
        type_name: &'static str,
        field: &'static str,
        value: i64,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct StdlibPackage {
    pub name: &'static str,
    pub functions: &'static [StdlibFunction],
    pub constants: &'static [StdlibConstant],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StdlibMethod {
    pub receiver_type: &'static str,
    pub method: &'static str,
    pub function: StdlibFunctionId,
}

type StdlibHandler = fn(&mut Vm, &Program, &[Value]) -> Result<Value, VmError>;
const EMPTY_STDLIB_CONSTANTS: &[StdlibConstant] = &[];

include!("stdlib/function_ids.rs");

const STRINGS_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: STRINGS_CONTAINS,
        symbol: "Contains",
        returns_value: true,
        handler: strings_impl::strings_contains,
    },
    StdlibFunction {
        id: STRINGS_HAS_PREFIX,
        symbol: "HasPrefix",
        returns_value: true,
        handler: strings_impl::strings_has_prefix,
    },
    StdlibFunction {
        id: STRINGS_HAS_SUFFIX,
        symbol: "HasSuffix",
        returns_value: true,
        handler: strings_impl::strings_has_suffix,
    },
    StdlibFunction {
        id: STRINGS_TRIM_SPACE,
        symbol: "TrimSpace",
        returns_value: true,
        handler: strings_impl::strings_trim_space,
    },
    StdlibFunction {
        id: STRINGS_TO_UPPER,
        symbol: "ToUpper",
        returns_value: true,
        handler: strings_impl::strings_to_upper,
    },
    StdlibFunction {
        id: STRINGS_TO_LOWER,
        symbol: "ToLower",
        returns_value: true,
        handler: strings_impl::strings_to_lower,
    },
    StdlibFunction {
        id: STRINGS_TO_TITLE,
        symbol: "ToTitle",
        returns_value: true,
        handler: strings_impl::strings_to_title,
    },
    StdlibFunction {
        id: STRINGS_COUNT,
        symbol: "Count",
        returns_value: true,
        handler: strings_impl::strings_count,
    },
    StdlibFunction {
        id: STRINGS_REPEAT,
        symbol: "Repeat",
        returns_value: true,
        handler: strings_impl::strings_repeat,
    },
    StdlibFunction {
        id: STRINGS_SPLIT,
        symbol: "Split",
        returns_value: true,
        handler: strings_impl::strings_split,
    },
    StdlibFunction {
        id: STRINGS_JOIN,
        symbol: "Join",
        returns_value: true,
        handler: strings_impl::strings_join,
    },
    StdlibFunction {
        id: STRINGS_NEW_REPLACER,
        symbol: "NewReplacer",
        returns_value: true,
        handler: strings_replacer_impl::strings_new_replacer,
    },
    StdlibFunction {
        id: STRINGS_REPLACE_ALL,
        symbol: "ReplaceAll",
        returns_value: true,
        handler: strings_impl::strings_replace_all,
    },
    StdlibFunction {
        id: STRINGS_FIELDS,
        symbol: "Fields",
        returns_value: true,
        handler: strings_impl::strings_fields,
    },
    StdlibFunction {
        id: STRINGS_INDEX,
        symbol: "Index",
        returns_value: true,
        handler: strings_impl::strings_index,
    },
    StdlibFunction {
        id: STRINGS_TRIM_PREFIX,
        symbol: "TrimPrefix",
        returns_value: true,
        handler: strings_impl::strings_trim_prefix,
    },
    StdlibFunction {
        id: STRINGS_TRIM_SUFFIX,
        symbol: "TrimSuffix",
        returns_value: true,
        handler: strings_impl::strings_trim_suffix,
    },
    StdlibFunction {
        id: STRINGS_LAST_INDEX,
        symbol: "LastIndex",
        returns_value: true,
        handler: strings_impl::strings_last_index,
    },
    StdlibFunction {
        id: STRINGS_TRIM_LEFT,
        symbol: "TrimLeft",
        returns_value: true,
        handler: strings_impl::strings_trim_left,
    },
    StdlibFunction {
        id: STRINGS_TRIM_RIGHT,
        symbol: "TrimRight",
        returns_value: true,
        handler: strings_impl::strings_trim_right,
    },
    StdlibFunction {
        id: STRINGS_TRIM,
        symbol: "Trim",
        returns_value: true,
        handler: strings_impl::strings_trim,
    },
    StdlibFunction {
        id: STRINGS_CONTAINS_ANY,
        symbol: "ContainsAny",
        returns_value: true,
        handler: strings_impl::strings_contains_any,
    },
    StdlibFunction {
        id: STRINGS_INDEX_ANY,
        symbol: "IndexAny",
        returns_value: true,
        handler: strings_impl::strings_index_any,
    },
    StdlibFunction {
        id: STRINGS_LAST_INDEX_ANY,
        symbol: "LastIndexAny",
        returns_value: true,
        handler: strings_impl::strings_last_index_any,
    },
    StdlibFunction {
        id: STRINGS_CLONE,
        symbol: "Clone",
        returns_value: true,
        handler: strings_impl::strings_clone,
    },
    StdlibFunction {
        id: STRINGS_CONTAINS_RUNE,
        symbol: "ContainsRune",
        returns_value: true,
        handler: strings_impl::strings_contains_rune,
    },
    StdlibFunction {
        id: STRINGS_INDEX_RUNE,
        symbol: "IndexRune",
        returns_value: true,
        handler: strings_impl::strings_index_rune,
    },
    StdlibFunction {
        id: STRINGS_COMPARE,
        symbol: "Compare",
        returns_value: true,
        handler: strings_impl::strings_compare,
    },
    StdlibFunction {
        id: STRINGS_REPLACE,
        symbol: "Replace",
        returns_value: true,
        handler: strings_impl::strings_replace,
    },
    StdlibFunction {
        id: STRINGS_INDEX_BYTE,
        symbol: "IndexByte",
        returns_value: true,
        handler: strings_impl::strings_index_byte,
    },
    StdlibFunction {
        id: STRINGS_LAST_INDEX_BYTE,
        symbol: "LastIndexByte",
        returns_value: true,
        handler: strings_impl::strings_last_index_byte,
    },
    StdlibFunction {
        id: STRINGS_CUT_PREFIX,
        symbol: "CutPrefix",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: STRINGS_CUT_SUFFIX,
        symbol: "CutSuffix",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: STRINGS_CUT,
        symbol: "Cut",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: STRINGS_EQUAL_FOLD,
        symbol: "EqualFold",
        returns_value: true,
        handler: strings_impl::strings_equal_fold,
    },
    StdlibFunction {
        id: STRINGS_SPLIT_N,
        symbol: "SplitN",
        returns_value: true,
        handler: strings_impl::strings_split_n,
    },
    StdlibFunction {
        id: STRINGS_SPLIT_AFTER_N,
        symbol: "SplitAfterN",
        returns_value: true,
        handler: strings_impl::strings_split_after_n,
    },
    StdlibFunction {
        id: STRINGS_SPLIT_AFTER,
        symbol: "SplitAfter",
        returns_value: true,
        handler: strings_impl::strings_split_after,
    },
    StdlibFunction {
        id: STRINGS_MAP,
        symbol: "Map",
        returns_value: true,
        handler: strings_impl::strings_map,
    },
    StdlibFunction {
        id: STRINGS_INDEX_FUNC,
        symbol: "IndexFunc",
        returns_value: true,
        handler: strings_impl::strings_index_func,
    },
    StdlibFunction {
        id: STRINGS_LAST_INDEX_FUNC,
        symbol: "LastIndexFunc",
        returns_value: true,
        handler: strings_impl::strings_last_index_func,
    },
    StdlibFunction {
        id: STRINGS_TRIM_FUNC,
        symbol: "TrimFunc",
        returns_value: true,
        handler: strings_impl::strings_trim_func,
    },
    StdlibFunction {
        id: STRINGS_TRIM_LEFT_FUNC,
        symbol: "TrimLeftFunc",
        returns_value: true,
        handler: strings_impl::strings_trim_left_func,
    },
    StdlibFunction {
        id: STRINGS_TRIM_RIGHT_FUNC,
        symbol: "TrimRightFunc",
        returns_value: true,
        handler: strings_impl::strings_trim_right_func,
    },
    StdlibFunction {
        id: STRINGS_FIELDS_FUNC,
        symbol: "FieldsFunc",
        returns_value: true,
        handler: strings_impl::strings_fields_func,
    },
];

pub fn stdlib_packages() -> &'static [StdlibPackage] {
    package_registry_impl::STDLIB_PACKAGES
}

pub fn resolve_stdlib_function(package: &str, symbol: &str) -> Option<StdlibFunctionId> {
    package_registry_impl::lookup_stdlib_function_by_name(package, symbol)
        .map(|function| function.id)
}

pub fn resolve_stdlib_constant(package: &str, symbol: &str) -> Option<&'static StdlibConstant> {
    package_registry_impl::lookup_stdlib_constant_by_name(package, symbol)
}

pub fn stdlib_function_returns_value(function: StdlibFunctionId) -> bool {
    package_registry_impl::lookup_stdlib_function(function)
        .map(|function| function.returns_value)
        .unwrap_or(false)
}

pub fn stdlib_function_mutates_first_arg(function: StdlibFunctionId) -> bool {
    matches!(
        function,
        SORT_INTS
            | SORT_STRINGS
            | SORT_FLOAT64S
            | SORT_SLICE
            | SORT_SLICE_STABLE
            | SLICES_SORT_FUNC
            | SLICES_SORT_STABLE_FUNC
            | SLICES_REVERSE
            | MAPS_COPY
            | MAPS_DELETE_FUNC
            | NET_HTTP_HEADER_SET
            | NET_HTTP_HEADER_ADD
            | NET_HTTP_HEADER_DEL
            | NET_URL_VALUES_SET
            | NET_URL_VALUES_ADD
            | NET_URL_VALUES_DEL
            | NET_HTTP_REQUEST_BODY_READ
            | NET_HTTP_RESPONSE_BODY_READ
    )
}

pub fn stdlib_function_result_count(function: StdlibFunctionId) -> usize {
    stdlib_function_result_types(function)
        .map(|result_types| result_types.len())
        .unwrap_or_else(|| usize::from(stdlib_function_returns_value(function)))
}

pub fn stdlib_function_result_types(function: StdlibFunctionId) -> Option<&'static [&'static str]> {
    signatures_impl::stdlib_function_result_types(function)
}

pub fn stdlib_function_param_types(function: StdlibFunctionId) -> Option<&'static [&'static str]> {
    signatures_impl::stdlib_function_param_types(function)
}

pub fn stdlib_function_variadic_param_type(function: StdlibFunctionId) -> Option<&'static str> {
    signatures_impl::stdlib_function_variadic_param_type(function)
}

impl Vm {
    pub(super) fn execute_stdlib(
        &mut self,
        program: &Program,
        function: StdlibFunctionId,
        args: &[Value],
    ) -> Result<Value, VmError> {
        let function = package_registry_impl::lookup_stdlib_function(function).ok_or(
            VmError::UnknownStdlibFunction {
                function: function.0,
            },
        )?;
        (function.handler)(self, program, args)
    }

    pub(super) fn execute_stdlib_multi(
        &mut self,
        program: &Program,
        function: StdlibFunctionId,
        args: &[Value],
    ) -> Result<Vec<Value>, VmError> {
        match function {
            CONTEXT_WITH_CANCEL => context_impl::context_with_cancel(self, program, args),
            CONTEXT_WITH_DEADLINE => context_impl::context_with_deadline(self, program, args),
            CONTEXT_WITH_TIMEOUT => context_impl::context_with_timeout(self, program, args),
            CONTEXT_DEADLINE => context_impl::context_deadline(self, program, args),
            REFLECT_STRUCT_TAG_LOOKUP => {
                reflect_impl::reflect_struct_tag_lookup(self, program, args)
            }
            PATH_MATCH => path_impl::path_match(self, program, args),
            PATH_SPLIT => path_impl::path_split(self, program, args),
            FILEPATH_GLOB => filepath_impl::filepath_glob(self, program, args),
            FILEPATH_ABS => filepath_impl::filepath_abs(self, program, args),
            FILEPATH_MATCH => filepath_impl::filepath_match(self, program, args),
            FILEPATH_REL => filepath_impl::filepath_rel(self, program, args),
            FILEPATH_LOCALIZE => filepath_impl::filepath_localize(self, program, args),
            FILEPATH_SPLIT => filepath_impl::filepath_split(self, program, args),
            NET_HTTP_PARSE_HTTP_VERSION => net_http_impl::parse_http_version(self, program, args),
            NET_HTTP_PARSE_TIME => net_http_impl::parse_time(self, program, args),
            NET_HTTP_GET => net_http_impl::get(self, program, args),
            NET_HTTP_HEAD => net_http_impl::head(self, program, args),
            NET_HTTP_POST => net_http_impl::post(self, program, args),
            NET_HTTP_POST_FORM => net_http_impl::post_form(self, program, args),
            NET_HTTP_CLIENT_DO => net_http_impl::client_do(self, program, args),
            NET_HTTP_CLIENT_GET => net_http_impl::client_get(self, program, args),
            NET_HTTP_CLIENT_HEAD => net_http_impl::client_head(self, program, args),
            NET_HTTP_CLIENT_POST => net_http_impl::client_post(self, program, args),
            NET_HTTP_CLIENT_POST_FORM => net_http_impl::client_post_form(self, program, args),
            NET_HTTP_RESPONSE_BODY_READ => net_http_impl::response_body_read(self, program, args),
            NET_HTTP_RESPONSE_LOCATION => net_http_impl::response_location(self, program, args),
            NET_HTTP_NEW_REQUEST => net_http_impl::new_request(self, program, args),
            NET_HTTP_NEW_REQUEST_WITH_CONTEXT => {
                net_http_impl::new_request_with_context(self, program, args)
            }
            NET_URL_JOIN_PATH => net_url_impl::url_join_path(self, program, args),
            NET_URL_PARSE => net_url_impl::url_parse(self, program, args),
            NET_URL_PARSE_QUERY => net_url_impl::url_parse_query(self, program, args),
            NET_URL_PARSE_REQUEST_URI => net_url_impl::url_parse_request_uri(self, program, args),
            NET_URL_PATH_UNESCAPE => net_url_impl::url_path_unescape(self, program, args),
            NET_URL_QUERY_UNESCAPE => net_url_impl::url_query_unescape(self, program, args),
            NET_URL_USERINFO_PASSWORD => net_url_impl::url_userinfo_password(self, program, args),
            NET_URL_URL_MARSHAL_BINARY => net_url_impl::url_url_marshal_binary(self, program, args),
            NET_URL_URL_PARSE => net_url_impl::url_url_parse(self, program, args),
            TIME_PARSE => time_impl::time_parse(self, program, args),
            STRCONV_ATOI => strconv_impl::strconv_atoi(self, program, args),
            STRCONV_PARSE_BOOL => strconv_impl::strconv_parse_bool(self, program, args),
            STRCONV_PARSE_INT => strconv_impl::strconv_parse_int(self, program, args),
            STRCONV_UNQUOTE => strconv_impl::strconv_unquote(self, program, args),
            STRCONV_UNQUOTE_CHAR => strconv_impl::strconv_unquote_char(self, program, args),
            BYTES_CUT => bytes_more_impl::bytes_cut(self, program, args),
            BYTES_CUT_PREFIX => bytes_more_impl::bytes_cut_prefix(self, program, args),
            BYTES_CUT_SUFFIX => bytes_more_impl::bytes_cut_suffix(self, program, args),
            STRINGS_CUT => strings_impl::strings_cut(self, program, args),
            STRINGS_CUT_PREFIX => strings_impl::strings_cut_prefix(self, program, args),
            STRINGS_CUT_SUFFIX => strings_impl::strings_cut_suffix(self, program, args),
            MATH_FREXP => math_impl::math_frexp(self, program, args),
            MATH_MODF => math_impl::math_modf(self, program, args),
            UTF8_DECODE_RUNE_IN_STRING => {
                unicode_utf8_impl::utf8_decode_rune_in_string(self, program, args)
            }
            OS_LOOKUP_ENV => os_impl::os_lookup_env(self, program, args),
            OS_READ_FILE => os_impl::os_read_file(self, program, args),
            OS_READ_DIR => os_impl::os_read_dir(self, program, args),
            OS_STAT => os_impl::os_stat(self, program, args),
            OS_LSTAT => os_impl::os_lstat(self, program, args),
            OS_GETWD => os_impl::os_getwd(self, program, args),
            OS_USER_HOME_DIR => os_impl::os_user_home_dir(self, program, args),
            OS_USER_CACHE_DIR => os_impl::os_user_cache_dir(self, program, args),
            OS_USER_CONFIG_DIR => os_impl::os_user_config_dir(self, program, args),
            OS_HOSTNAME => os_impl::os_hostname(self, program, args),
            OS_EXECUTABLE => os_impl::os_executable(self, program, args),
            OS_GETGROUPS => os_impl::os_getgroups(self, program, args),
            REGEXP_MATCH_STRING => regexp_impl::regexp_match_string(self, program, args),
            REGEXP_COMPILE => regexp_impl::regexp_compile(self, program, args),
            HEX_DECODE_STRING => hex_impl::hex_decode_string(self, program, args),
            BASE64_ENCODING_DECODE_STRING => {
                base64_impl::base64_encoding_decode_string(self, program, args)
            }
            BASE64_STD_ENCODING_DECODE_STRING => {
                base64_impl::base64_std_encoding_decode_string(self, program, args)
            }
            BASE64_URL_ENCODING_DECODE_STRING => {
                base64_impl::base64_url_encoding_decode_string(self, program, args)
            }
            BASE64_RAW_STD_ENCODING_DECODE_STRING => {
                base64_impl::base64_raw_std_encoding_decode_string(self, program, args)
            }
            BASE64_RAW_URL_ENCODING_DECODE_STRING => {
                base64_impl::base64_raw_url_encoding_decode_string(self, program, args)
            }
            JSON_MARSHAL => json_impl::json_marshal(self, program, args),
            JSON_MARSHAL_INDENT => json_impl::json_marshal_indent(self, program, args),
            IO_FS_FS_OPEN => io_fs_impl::io_fs_fs_open(self, program, args),
            IO_FS_FILE_STAT => io_fs_impl::io_fs_file_stat(self, program, args),
            IO_FS_FILE_READ => io_fs_impl::io_fs_file_read(self, program, args),
            IO_FS_READ_DIR_FILE_READ_DIR => {
                io_fs_impl::io_fs_read_dir_file_read_dir(self, program, args)
            }
            IO_FS_DIR_ENTRY_INFO => io_fs_impl::io_fs_dir_entry_info(self, program, args),
            IO_FS_READ_FILE => io_fs_impl::io_fs_read_file(self, program, args),
            IO_FS_READ_DIR => io_fs_impl::io_fs_read_dir(self, program, args),
            IO_FS_STAT => io_fs_impl::io_fs_stat(self, program, args),
            IO_FS_SUB => io_fs_impl::io_fs_sub(self, program, args),
            IO_FS_GLOB => io_fs_impl::io_fs_glob(self, program, args),
            STRCONV_PARSE_FLOAT => strconv_impl::strconv_parse_float(self, program, args),
            STRCONV_PARSE_UINT => strconv_impl::strconv_parse_uint(self, program, args),
            _ => Err(VmError::UnknownStdlibFunction {
                function: function.0,
            }),
        }
    }
}

fn unsupported_multi_result_stdlib(
    _vm: &mut Vm,
    _program: &Program,
    _args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::nil())
}
