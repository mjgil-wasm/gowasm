use super::*;

pub(super) const STDLIB_PACKAGES: &[StdlibPackage] = &[
    StdlibPackage {
        name: "builtin",
        functions: builtins_impl::BUILTIN_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "bytes",
        functions: bytes_impl::BYTES_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "cmp",
        functions: cmp_impl::CMP_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "context",
        functions: context_impl::CONTEXT_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "errors",
        functions: errors_impl::ERRORS_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "fmt",
        functions: fmt_impl::FMT_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "log",
        functions: log_impl::LOG_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "math",
        functions: math_impl::MATH_FUNCTIONS,
        constants: math_impl::MATH_CONSTANTS,
    },
    StdlibPackage {
        name: "os",
        functions: os_impl::OS_FUNCTIONS,
        constants: os_impl::OS_CONSTANTS,
    },
    StdlibPackage {
        name: "path",
        functions: path_impl::PATH_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "sort",
        functions: sort_impl::SORT_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "maps",
        functions: maps_impl::MAPS_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "slices",
        functions: slices_impl::SLICES_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "strings",
        functions: STRINGS_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "sync",
        functions: &[],
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "testing",
        functions: testing_impl::TESTING_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "strconv",
        functions: strconv_impl::STRCONV_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "unicode",
        functions: unicode_impl::UNICODE_FUNCTIONS,
        constants: unicode_impl::UNICODE_CONSTANTS,
    },
    StdlibPackage {
        name: "math/rand",
        functions: rand_impl::RAND_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "math/bits",
        functions: math_bits_impl::MATH_BITS_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "unicode/utf8",
        functions: unicode_utf8_impl::UTF8_FUNCTIONS,
        constants: unicode_utf8_impl::UTF8_CONSTANTS,
    },
    StdlibPackage {
        name: "regexp",
        functions: regexp_impl::REGEXP_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "encoding/hex",
        functions: hex_impl::HEX_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "encoding/json",
        functions: json_impl::JSON_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "crypto/sha256",
        functions: sha256_impl::SHA256_FUNCTIONS,
        constants: &[
            StdlibConstant {
                symbol: "BlockSize",
                typ: "int",
                value: StdlibConstantValue::Int(sha256_impl::SHA256_BLOCK_SIZE),
            },
            StdlibConstant {
                symbol: "Size",
                typ: "int",
                value: StdlibConstantValue::Int(sha256_impl::SHA256_SIZE),
            },
        ],
    },
    StdlibPackage {
        name: "crypto/md5",
        functions: md5_impl::MD5_FUNCTIONS,
        constants: &[
            StdlibConstant {
                symbol: "BlockSize",
                typ: "int",
                value: StdlibConstantValue::Int(md5_impl::MD5_BLOCK_SIZE),
            },
            StdlibConstant {
                symbol: "Size",
                typ: "int",
                value: StdlibConstantValue::Int(md5_impl::MD5_SIZE),
            },
        ],
    },
    StdlibPackage {
        name: "crypto/sha1",
        functions: sha1_impl::SHA1_FUNCTIONS,
        constants: &[
            StdlibConstant {
                symbol: "BlockSize",
                typ: "int",
                value: StdlibConstantValue::Int(sha1_impl::SHA1_BLOCK_SIZE),
            },
            StdlibConstant {
                symbol: "Size",
                typ: "int",
                value: StdlibConstantValue::Int(sha1_impl::SHA1_SIZE),
            },
        ],
    },
    StdlibPackage {
        name: "crypto/sha512",
        functions: sha512_impl::SHA512_FUNCTIONS,
        constants: &[
            StdlibConstant {
                symbol: "BlockSize",
                typ: "int",
                value: StdlibConstantValue::Int(sha512_impl::SHA512_BLOCK_SIZE),
            },
            StdlibConstant {
                symbol: "Size",
                typ: "int",
                value: StdlibConstantValue::Int(sha512_impl::SHA512_SIZE),
            },
        ],
    },
    StdlibPackage {
        name: "encoding/base64",
        functions: base64_impl::BASE64_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "time",
        functions: time_impl::TIME_FUNCTIONS,
        constants: time_impl::TIME_CONSTANTS,
    },
    StdlibPackage {
        name: "io/fs",
        functions: io_fs_registry_impl::IO_FS_FUNCTIONS,
        constants: io_fs_registry_impl::IO_FS_CONSTANTS,
    },
    StdlibPackage {
        name: "path/filepath",
        functions: filepath_impl::FILEPATH_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "net/http",
        functions: net_http_impl::NET_HTTP_FUNCTIONS,
        constants: net_http_impl::NET_HTTP_CONSTANTS,
    },
    StdlibPackage {
        name: "net/url",
        functions: net_url_impl::NET_URL_FUNCTIONS,
        constants: EMPTY_STDLIB_CONSTANTS,
    },
    StdlibPackage {
        name: "reflect",
        functions: reflect_impl::REFLECT_FUNCTIONS,
        constants: reflect_impl::REFLECT_CONSTANTS,
    },
];

pub(super) fn lookup_stdlib_function(
    function: StdlibFunctionId,
) -> Option<&'static StdlibFunction> {
    STDLIB_PACKAGES
        .iter()
        .flat_map(|package| package.functions.iter())
        .chain(base64_impl::BASE64_METHOD_FUNCTIONS.iter())
        .chain(context_impl::CONTEXT_METHOD_FUNCTIONS.iter())
        .chain(io_fs_registry_impl::IO_FS_METHOD_FUNCTIONS.iter())
        .chain(net_http_impl::NET_HTTP_METHOD_FUNCTIONS.iter())
        .chain(net_http_impl::NET_HTTP_REQUEST_METHOD_FUNCTIONS.iter())
        .chain(net_http_impl::NET_HTTP_REQUEST_BODY_METHOD_FUNCTIONS.iter())
        .chain(net_http_impl::NET_HTTP_RESPONSE_METHOD_FUNCTIONS.iter())
        .chain(net_http_impl::NET_HTTP_TRANSPORT_METHOD_FUNCTIONS.iter())
        .chain(net_url_impl::NET_URL_METHOD_FUNCTIONS.iter())
        .chain(reflect_impl::REFLECT_METHOD_FUNCTIONS.iter())
        .chain(regexp_impl::REGEXP_METHOD_FUNCTIONS.iter())
        .chain(strings_replacer_impl::STRINGS_REPLACER_METHOD_FUNCTIONS.iter())
        .chain(sync_impl::SYNC_METHOD_FUNCTIONS.iter())
        .chain(testing_impl::TESTING_METHOD_FUNCTIONS.iter())
        .chain(time_impl::TIME_METHOD_FUNCTIONS.iter())
        .find(|entry| entry.id == function)
}

pub(super) fn lookup_stdlib_function_by_name(
    package: &str,
    symbol: &str,
) -> Option<&'static StdlibFunction> {
    STDLIB_PACKAGES
        .iter()
        .find(|entry| entry.name == package)?
        .functions
        .iter()
        .find(|function| function.symbol == symbol)
}

pub(super) fn lookup_stdlib_constant_by_name(
    package: &str,
    symbol: &str,
) -> Option<&'static StdlibConstant> {
    STDLIB_PACKAGES
        .iter()
        .find(|entry| entry.name == package)?
        .constants
        .iter()
        .find(|constant| constant.symbol == symbol)
}
