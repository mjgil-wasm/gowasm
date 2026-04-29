use super::super::{
    unsupported_multi_result_stdlib, StdlibConstant, StdlibConstantValue, StdlibFunction,
    StdlibValue, StdlibValueInit, NET_HTTP_CANONICAL_HEADER_KEY, NET_HTTP_DETECT_CONTENT_TYPE,
    NET_HTTP_GET, NET_HTTP_HEAD, NET_HTTP_NEW_REQUEST, NET_HTTP_NEW_REQUEST_WITH_CONTEXT,
    NET_HTTP_PARSE_HTTP_VERSION, NET_HTTP_PARSE_TIME, NET_HTTP_POST, NET_HTTP_POST_FORM,
    NET_HTTP_STATUS_TEXT,
};
use super::sniff::detect_content_type;
use crate::{Program, Value, ValueData, Vm, VmError};

pub(crate) const NET_HTTP_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: NET_HTTP_CANONICAL_HEADER_KEY,
        symbol: "CanonicalHeaderKey",
        returns_value: true,
        handler: canonical_header_key,
    },
    StdlibFunction {
        id: NET_HTTP_STATUS_TEXT,
        symbol: "StatusText",
        returns_value: true,
        handler: status_text,
    },
    StdlibFunction {
        id: NET_HTTP_PARSE_HTTP_VERSION,
        symbol: "ParseHTTPVersion",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_DETECT_CONTENT_TYPE,
        symbol: "DetectContentType",
        returns_value: true,
        handler: detect_content_type,
    },
    StdlibFunction {
        id: NET_HTTP_PARSE_TIME,
        symbol: "ParseTime",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_NEW_REQUEST,
        symbol: "NewRequest",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_NEW_REQUEST_WITH_CONTEXT,
        symbol: "NewRequestWithContext",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_GET,
        symbol: "Get",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_POST,
        symbol: "Post",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_POST_FORM,
        symbol: "PostForm",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: NET_HTTP_HEAD,
        symbol: "Head",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
];

pub(crate) const NET_HTTP_VALUES: &[StdlibValue] = &[
    StdlibValue {
        symbol: "ErrMissingFile",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("http: no such file")),
    },
    StdlibValue {
        symbol: "ErrNoCookie",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(
            "http: named cookie not present",
        )),
    },
    StdlibValue {
        symbol: "ErrNoLocation",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(
            "http: no Location header in response",
        )),
    },
    StdlibValue {
        symbol: "ErrUseLastResponse",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("net/http: use last response")),
    },
    StdlibValue {
        symbol: "ErrAbortHandler",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("net/http: abort Handler")),
    },
    StdlibValue {
        symbol: "ErrServerClosed",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error("http: Server closed")),
    },
    StdlibValue {
        symbol: "DefaultClient",
        typ: "*http.Client",
        value: StdlibValueInit::NewPointer("http.Client"),
    },
];

pub(crate) const NET_HTTP_CONSTANTS: &[StdlibConstant] = &[
    StdlibConstant {
        symbol: "MethodGet",
        typ: "string",
        value: StdlibConstantValue::String("GET"),
    },
    StdlibConstant {
        symbol: "MethodHead",
        typ: "string",
        value: StdlibConstantValue::String("HEAD"),
    },
    StdlibConstant {
        symbol: "MethodPost",
        typ: "string",
        value: StdlibConstantValue::String("POST"),
    },
    StdlibConstant {
        symbol: "MethodPut",
        typ: "string",
        value: StdlibConstantValue::String("PUT"),
    },
    StdlibConstant {
        symbol: "MethodPatch",
        typ: "string",
        value: StdlibConstantValue::String("PATCH"),
    },
    StdlibConstant {
        symbol: "MethodDelete",
        typ: "string",
        value: StdlibConstantValue::String("DELETE"),
    },
    StdlibConstant {
        symbol: "MethodConnect",
        typ: "string",
        value: StdlibConstantValue::String("CONNECT"),
    },
    StdlibConstant {
        symbol: "MethodOptions",
        typ: "string",
        value: StdlibConstantValue::String("OPTIONS"),
    },
    StdlibConstant {
        symbol: "MethodTrace",
        typ: "string",
        value: StdlibConstantValue::String("TRACE"),
    },
    StdlibConstant {
        symbol: "TimeFormat",
        typ: "string",
        value: StdlibConstantValue::String("Mon, 02 Jan 2006 15:04:05 GMT"),
    },
    StdlibConstant {
        symbol: "TrailerPrefix",
        typ: "string",
        value: StdlibConstantValue::String("Trailer:"),
    },
    StdlibConstant {
        symbol: "DefaultMaxHeaderBytes",
        typ: "int",
        value: StdlibConstantValue::Int(1 << 20),
    },
    StdlibConstant {
        symbol: "StatusContinue",
        typ: "int",
        value: StdlibConstantValue::Int(100),
    },
    StdlibConstant {
        symbol: "StatusSwitchingProtocols",
        typ: "int",
        value: StdlibConstantValue::Int(101),
    },
    StdlibConstant {
        symbol: "StatusProcessing",
        typ: "int",
        value: StdlibConstantValue::Int(102),
    },
    StdlibConstant {
        symbol: "StatusEarlyHints",
        typ: "int",
        value: StdlibConstantValue::Int(103),
    },
    StdlibConstant {
        symbol: "StatusOK",
        typ: "int",
        value: StdlibConstantValue::Int(200),
    },
    StdlibConstant {
        symbol: "StatusCreated",
        typ: "int",
        value: StdlibConstantValue::Int(201),
    },
    StdlibConstant {
        symbol: "StatusAccepted",
        typ: "int",
        value: StdlibConstantValue::Int(202),
    },
    StdlibConstant {
        symbol: "StatusNonAuthoritativeInfo",
        typ: "int",
        value: StdlibConstantValue::Int(203),
    },
    StdlibConstant {
        symbol: "StatusNoContent",
        typ: "int",
        value: StdlibConstantValue::Int(204),
    },
    StdlibConstant {
        symbol: "StatusResetContent",
        typ: "int",
        value: StdlibConstantValue::Int(205),
    },
    StdlibConstant {
        symbol: "StatusPartialContent",
        typ: "int",
        value: StdlibConstantValue::Int(206),
    },
    StdlibConstant {
        symbol: "StatusMultiStatus",
        typ: "int",
        value: StdlibConstantValue::Int(207),
    },
    StdlibConstant {
        symbol: "StatusAlreadyReported",
        typ: "int",
        value: StdlibConstantValue::Int(208),
    },
    StdlibConstant {
        symbol: "StatusIMUsed",
        typ: "int",
        value: StdlibConstantValue::Int(226),
    },
    StdlibConstant {
        symbol: "StatusMultipleChoices",
        typ: "int",
        value: StdlibConstantValue::Int(300),
    },
    StdlibConstant {
        symbol: "StatusMovedPermanently",
        typ: "int",
        value: StdlibConstantValue::Int(301),
    },
    StdlibConstant {
        symbol: "StatusFound",
        typ: "int",
        value: StdlibConstantValue::Int(302),
    },
    StdlibConstant {
        symbol: "StatusSeeOther",
        typ: "int",
        value: StdlibConstantValue::Int(303),
    },
    StdlibConstant {
        symbol: "StatusNotModified",
        typ: "int",
        value: StdlibConstantValue::Int(304),
    },
    StdlibConstant {
        symbol: "StatusUseProxy",
        typ: "int",
        value: StdlibConstantValue::Int(305),
    },
    StdlibConstant {
        symbol: "StatusTemporaryRedirect",
        typ: "int",
        value: StdlibConstantValue::Int(307),
    },
    StdlibConstant {
        symbol: "StatusPermanentRedirect",
        typ: "int",
        value: StdlibConstantValue::Int(308),
    },
    StdlibConstant {
        symbol: "StatusBadRequest",
        typ: "int",
        value: StdlibConstantValue::Int(400),
    },
    StdlibConstant {
        symbol: "StatusUnauthorized",
        typ: "int",
        value: StdlibConstantValue::Int(401),
    },
    StdlibConstant {
        symbol: "StatusPaymentRequired",
        typ: "int",
        value: StdlibConstantValue::Int(402),
    },
    StdlibConstant {
        symbol: "StatusForbidden",
        typ: "int",
        value: StdlibConstantValue::Int(403),
    },
    StdlibConstant {
        symbol: "StatusNotFound",
        typ: "int",
        value: StdlibConstantValue::Int(404),
    },
    StdlibConstant {
        symbol: "StatusMethodNotAllowed",
        typ: "int",
        value: StdlibConstantValue::Int(405),
    },
    StdlibConstant {
        symbol: "StatusNotAcceptable",
        typ: "int",
        value: StdlibConstantValue::Int(406),
    },
    StdlibConstant {
        symbol: "StatusProxyAuthRequired",
        typ: "int",
        value: StdlibConstantValue::Int(407),
    },
    StdlibConstant {
        symbol: "StatusRequestTimeout",
        typ: "int",
        value: StdlibConstantValue::Int(408),
    },
    StdlibConstant {
        symbol: "StatusConflict",
        typ: "int",
        value: StdlibConstantValue::Int(409),
    },
    StdlibConstant {
        symbol: "StatusGone",
        typ: "int",
        value: StdlibConstantValue::Int(410),
    },
    StdlibConstant {
        symbol: "StatusLengthRequired",
        typ: "int",
        value: StdlibConstantValue::Int(411),
    },
    StdlibConstant {
        symbol: "StatusPreconditionFailed",
        typ: "int",
        value: StdlibConstantValue::Int(412),
    },
    StdlibConstant {
        symbol: "StatusRequestEntityTooLarge",
        typ: "int",
        value: StdlibConstantValue::Int(413),
    },
    StdlibConstant {
        symbol: "StatusRequestURITooLong",
        typ: "int",
        value: StdlibConstantValue::Int(414),
    },
    StdlibConstant {
        symbol: "StatusUnsupportedMediaType",
        typ: "int",
        value: StdlibConstantValue::Int(415),
    },
    StdlibConstant {
        symbol: "StatusRequestedRangeNotSatisfiable",
        typ: "int",
        value: StdlibConstantValue::Int(416),
    },
    StdlibConstant {
        symbol: "StatusExpectationFailed",
        typ: "int",
        value: StdlibConstantValue::Int(417),
    },
    StdlibConstant {
        symbol: "StatusTeapot",
        typ: "int",
        value: StdlibConstantValue::Int(418),
    },
    StdlibConstant {
        symbol: "StatusMisdirectedRequest",
        typ: "int",
        value: StdlibConstantValue::Int(421),
    },
    StdlibConstant {
        symbol: "StatusUnprocessableEntity",
        typ: "int",
        value: StdlibConstantValue::Int(422),
    },
    StdlibConstant {
        symbol: "StatusLocked",
        typ: "int",
        value: StdlibConstantValue::Int(423),
    },
    StdlibConstant {
        symbol: "StatusFailedDependency",
        typ: "int",
        value: StdlibConstantValue::Int(424),
    },
    StdlibConstant {
        symbol: "StatusTooEarly",
        typ: "int",
        value: StdlibConstantValue::Int(425),
    },
    StdlibConstant {
        symbol: "StatusUpgradeRequired",
        typ: "int",
        value: StdlibConstantValue::Int(426),
    },
    StdlibConstant {
        symbol: "StatusPreconditionRequired",
        typ: "int",
        value: StdlibConstantValue::Int(428),
    },
    StdlibConstant {
        symbol: "StatusTooManyRequests",
        typ: "int",
        value: StdlibConstantValue::Int(429),
    },
    StdlibConstant {
        symbol: "StatusRequestHeaderFieldsTooLarge",
        typ: "int",
        value: StdlibConstantValue::Int(431),
    },
    StdlibConstant {
        symbol: "StatusUnavailableForLegalReasons",
        typ: "int",
        value: StdlibConstantValue::Int(451),
    },
    StdlibConstant {
        symbol: "StatusInternalServerError",
        typ: "int",
        value: StdlibConstantValue::Int(500),
    },
    StdlibConstant {
        symbol: "StatusNotImplemented",
        typ: "int",
        value: StdlibConstantValue::Int(501),
    },
    StdlibConstant {
        symbol: "StatusBadGateway",
        typ: "int",
        value: StdlibConstantValue::Int(502),
    },
    StdlibConstant {
        symbol: "StatusServiceUnavailable",
        typ: "int",
        value: StdlibConstantValue::Int(503),
    },
    StdlibConstant {
        symbol: "StatusGatewayTimeout",
        typ: "int",
        value: StdlibConstantValue::Int(504),
    },
    StdlibConstant {
        symbol: "StatusHTTPVersionNotSupported",
        typ: "int",
        value: StdlibConstantValue::Int(505),
    },
    StdlibConstant {
        symbol: "StatusVariantAlsoNegotiates",
        typ: "int",
        value: StdlibConstantValue::Int(506),
    },
    StdlibConstant {
        symbol: "StatusInsufficientStorage",
        typ: "int",
        value: StdlibConstantValue::Int(507),
    },
    StdlibConstant {
        symbol: "StatusLoopDetected",
        typ: "int",
        value: StdlibConstantValue::Int(508),
    },
    StdlibConstant {
        symbol: "StatusNotExtended",
        typ: "int",
        value: StdlibConstantValue::Int(510),
    },
    StdlibConstant {
        symbol: "StatusNetworkAuthenticationRequired",
        typ: "int",
        value: StdlibConstantValue::Int(511),
    },
];

pub(super) fn canonical_header_key(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(header) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "http.CanonicalHeaderKey".into(),
            expected: "a string argument".into(),
        });
    };
    Ok(Value::string(canonicalize_header_key_text(header)))
}

pub(super) fn status_text(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Int(code) = args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "http.StatusText".into(),
            expected: "an int argument".into(),
        });
    };
    Ok(Value::string(status_text_for_code(code)))
}

pub(super) fn status_text_for_code(code: i64) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Request Entity Too Large",
        414 => "Request URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Requested Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "",
    }
}

pub(crate) fn parse_http_version(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::String(version) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "http.ParseHTTPVersion".into(),
            expected: "a string argument".into(),
        });
    };

    let parsed = match version.as_str() {
        "HTTP/1.1" => Some((1, 1)),
        "HTTP/1.0" => Some((1, 0)),
        _ => {
            let bytes = version.as_bytes();
            if bytes.len() != b"HTTP/X.Y".len()
                || !bytes.starts_with(b"HTTP/")
                || bytes[6] != b'.'
                || !bytes[5].is_ascii_digit()
                || !bytes[7].is_ascii_digit()
            {
                None
            } else {
                Some((i64::from(bytes[5] - b'0'), i64::from(bytes[7] - b'0')))
            }
        }
    };

    Ok(match parsed {
        Some((major, minor)) => vec![Value::int(major), Value::int(minor), Value::bool(true)],
        None => vec![Value::int(0), Value::int(0), Value::bool(false)],
    })
}

fn valid_header_field_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'0'..=b'9'
            | b'a'..=b'z'
            | b'A'..=b'Z'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

pub(super) fn canonicalize_header_key_text(header: &str) -> String {
    if header.is_empty() {
        return String::new();
    }

    let bytes = header.as_bytes();
    if bytes.iter().any(|byte| !valid_header_field_byte(*byte)) {
        return header.to_string();
    }

    let mut upper = true;
    let mut canonicalized = Vec::with_capacity(bytes.len());
    for byte in bytes {
        let mapped = if upper && byte.is_ascii_lowercase() {
            byte.to_ascii_uppercase()
        } else if !upper && byte.is_ascii_uppercase() {
            byte.to_ascii_lowercase()
        } else {
            *byte
        };
        canonicalized.push(mapped);
        upper = mapped == b'-';
    }

    String::from_utf8(canonicalized).expect("header key bytes stay ASCII")
}
