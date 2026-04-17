#[derive(Clone, Default)]
pub(super) struct ParsedUserinfoFields {
    pub(super) username: String,
    pub(super) password: String,
    pub(super) password_set: bool,
}

#[derive(Clone, Default)]
pub(super) struct ParsedUrlFields {
    pub(super) scheme: String,
    pub(super) opaque: String,
    pub(super) user: Option<ParsedUserinfoFields>,
    pub(super) host: String,
    pub(super) path: String,
    pub(super) raw_path: String,
    pub(super) force_query: bool,
    pub(super) raw_query: String,
    pub(super) fragment: String,
    pub(super) raw_fragment: String,
}

pub(super) fn parse_url_fields(text: &str) -> Result<ParsedUrlFields, String> {
    parse_url_fields_in_mode(text, false)
}

pub(super) fn parse_request_uri_fields(text: &str) -> Result<ParsedUrlFields, String> {
    parse_url_fields_in_mode(text, true)
}

fn parse_url_fields_in_mode(text: &str, via_request: bool) -> Result<ParsedUrlFields, String> {
    if text.bytes().any(invalid_url_byte) {
        return Err("net/url: invalid control character in URL".into());
    }

    if via_request && text.is_empty() {
        return Err("empty url".into());
    }

    if text == "*" {
        return Ok(ParsedUrlFields {
            path: "*".into(),
            ..ParsedUrlFields::default()
        });
    }

    let (without_fragment, fragment) = if via_request {
        (text, "")
    } else {
        text.split_once('#').unwrap_or((text, ""))
    };

    let mut scheme = "";
    let mut remainder = without_fragment;
    if let Some(colon) = without_fragment.find(':') {
        let path_start = without_fragment.find('/').unwrap_or(without_fragment.len());
        if colon < path_start && valid_scheme(&without_fragment[..colon]) {
            scheme = &without_fragment[..colon];
            remainder = &without_fragment[colon + 1..];
        }
    }

    let (remainder, raw_query, force_query) = if remainder.ends_with('?')
        && remainder.bytes().filter(|byte| *byte == b'?').count() == 1
    {
        (&remainder[..remainder.len() - 1], "", true)
    } else {
        let (without_query, raw_query) = remainder.split_once('?').unwrap_or((remainder, ""));
        (without_query, raw_query, false)
    };

    if !remainder.starts_with('/') && !scheme.is_empty() {
        let (fragment, raw_fragment) = parsed_fragment_fields(fragment)?;
        return Ok(ParsedUrlFields {
            scheme: scheme.into(),
            opaque: remainder.into(),
            user: None,
            host: String::new(),
            path: String::new(),
            raw_path: String::new(),
            force_query,
            raw_query: raw_query.into(),
            fragment,
            raw_fragment,
        });
    }

    if via_request && !remainder.starts_with('/') {
        return Err("invalid URI for request".into());
    }

    let mut user = None;
    let mut host = "";
    let mut path = remainder;
    if (!via_request || !scheme.is_empty()) && remainder.starts_with("//") {
        let authority = remainder
            .strip_prefix("//")
            .expect("prefix was already checked");
        let host_end = authority.find('/').unwrap_or(authority.len());
        let (parsed_user, parsed_host) = parse_authority_fields(&authority[..host_end])?;
        user = parsed_user;
        host = parsed_host;
        path = &authority[host_end..];
    }

    let (path, raw_path) = parsed_path_fields(path)?;
    let (fragment, raw_fragment) = parsed_fragment_fields(fragment)?;

    Ok(ParsedUrlFields {
        scheme: scheme.into(),
        opaque: String::new(),
        user,
        host: host.into(),
        path,
        raw_path,
        force_query,
        raw_query: raw_query.into(),
        fragment,
        raw_fragment,
    })
}

pub(super) fn query_escape_component(text: &str) -> String {
    escape_component(text, ComponentEscapeMode::QueryComponent)
}

pub(super) fn path_escape_component(text: &str) -> String {
    escape_component(text, ComponentEscapeMode::PathSegment)
}

pub(super) fn userinfo_string_text(username: &str, password: Option<&str>) -> String {
    let mut rendered = escape_component(username, ComponentEscapeMode::Userinfo);
    if let Some(password) = password {
        rendered.push(':');
        rendered.push_str(&escape_component(password, ComponentEscapeMode::Userinfo));
    }
    rendered
}

pub(super) fn escaped_path_text_with_hint(path: &str, raw_path: &str) -> String {
    if raw_hint_matches_component(raw_path, path, ComponentEscapeMode::Path) {
        return raw_path.into();
    }
    escaped_path_text(path)
}

pub(super) fn escaped_fragment_text_with_hint(fragment: &str, raw_fragment: &str) -> String {
    if raw_hint_matches_component(raw_fragment, fragment, ComponentEscapeMode::Fragment) {
        return raw_fragment.into();
    }
    escaped_fragment_text(fragment)
}

pub(super) fn parsed_path_fields_from_text(text: &str) -> Result<(String, String), String> {
    parsed_path_fields(text)
}

pub(super) fn resolve_path_text(base: &str, reference: &str) -> String {
    let full = if reference.is_empty() {
        base.to_owned()
    } else if !reference.starts_with('/') {
        let index = base.rfind('/').map(|value| value + 1).unwrap_or(0);
        format!("{}{}", &base[..index], reference)
    } else {
        reference.to_owned()
    };

    if full.is_empty() {
        return String::new();
    }

    let mut output = String::from("/");
    let mut first = true;
    let mut last = "";

    for element in full.split('/') {
        last = element;
        if element == "." {
            first = false;
            continue;
        }

        if element == ".." {
            let trimmed = &output[1..];
            if let Some(index) = trimmed.rfind('/') {
                output.truncate(index + 1);
                first = false;
            } else {
                output.truncate(1);
                first = true;
            }
            continue;
        }

        if !first {
            output.push('/');
        }
        output.push_str(element);
        first = false;
    }

    if matches!(last, "." | "..") {
        output.push('/');
    }

    if output.len() > 1 && output.as_bytes()[1] == b'/' {
        output.remove(0);
    }
    output
}

pub(super) fn query_unescape_component(text: &str) -> Result<String, String> {
    unescape_component(text, ComponentEscapeMode::QueryComponent)
}

pub(super) fn path_unescape_component(text: &str) -> Result<String, String> {
    unescape_component(text, ComponentEscapeMode::PathSegment)
}

fn invalid_url_byte(byte: u8) -> bool {
    matches!(byte, 0..=0x1f | 0x7f)
}

fn valid_scheme(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ComponentEscapeMode {
    Fragment,
    Path,
    PathSegment,
    QueryComponent,
    Userinfo,
}

fn escaped_path_text(text: &str) -> String {
    if text == "*" {
        return "*".into();
    }
    escape_component(text, ComponentEscapeMode::Path)
}

fn escaped_fragment_text(text: &str) -> String {
    escape_component(text, ComponentEscapeMode::Fragment)
}

fn escape_component(text: &str, mode: ComponentEscapeMode) -> String {
    let mut escaped = String::new();
    for byte in text.bytes() {
        match byte {
            b' ' if mode == ComponentEscapeMode::QueryComponent => escaped.push('+'),
            byte if !should_escape(byte, mode) => escaped.push(byte as char),
            _ => {
                escaped.push('%');
                escaped.push(upper_hex((byte >> 4) & 0x0f));
                escaped.push(upper_hex(byte & 0x0f));
            }
        }
    }
    escaped
}

fn parsed_path_fields(text: &str) -> Result<(String, String), String> {
    let path = unescape_component(text, ComponentEscapeMode::Path)?;
    let raw_path = if text == escaped_path_text(&path) {
        String::new()
    } else {
        text.into()
    };
    Ok((path, raw_path))
}

fn parsed_fragment_fields(text: &str) -> Result<(String, String), String> {
    let fragment = unescape_component(text, ComponentEscapeMode::Fragment)?;
    let raw_fragment = if text == escaped_fragment_text(&fragment) {
        String::new()
    } else {
        text.into()
    };
    Ok((fragment, raw_fragment))
}

fn raw_hint_matches_component(raw_text: &str, decoded: &str, mode: ComponentEscapeMode) -> bool {
    !raw_text.is_empty()
        && valid_encoded_component(raw_text, mode)
        && matches!(unescape_component(raw_text, mode), Ok(text) if text == decoded)
}

fn unescape_component(text: &str, mode: ComponentEscapeMode) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' if mode == ComponentEscapeMode::QueryComponent => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(invalid_url_escape(bytes, index));
                }
                let Some(high) = hex_value(bytes[index + 1]) else {
                    return Err(invalid_url_escape(bytes, index));
                };
                let Some(low) = hex_value(bytes[index + 2]) else {
                    return Err(invalid_url_escape(bytes, index));
                };
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(decoded).map_err(|_| match mode {
        ComponentEscapeMode::Fragment => "invalid UTF-8 in fragment".into(),
        ComponentEscapeMode::Path | ComponentEscapeMode::PathSegment => {
            "invalid UTF-8 in path".into()
        }
        ComponentEscapeMode::QueryComponent => "invalid UTF-8 in query".into(),
        ComponentEscapeMode::Userinfo => "invalid UTF-8 in userinfo".into(),
    })
}

fn valid_encoded_component(text: &str, mode: ComponentEscapeMode) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len()
                    || hex_value(bytes[index + 1]).is_none()
                    || hex_value(bytes[index + 2]).is_none()
                {
                    return false;
                }
                index += 3;
            }
            byte if !should_escape(byte, mode) => index += 1,
            _ => return false,
        }
    }
    true
}

fn should_escape(byte: u8, mode: ComponentEscapeMode) -> bool {
    match byte {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => false,
        b'$' | b'&' | b'+' | b',' | b'/' | b':' | b';' | b'=' | b'?' | b'@' => match mode {
            ComponentEscapeMode::Fragment => false,
            ComponentEscapeMode::Path => byte == b'?',
            ComponentEscapeMode::PathSegment => {
                matches!(byte, b'/' | b';' | b',' | b'?')
            }
            ComponentEscapeMode::QueryComponent => true,
            ComponentEscapeMode::Userinfo => matches!(byte, b'@' | b'/' | b'?' | b':'),
        },
        b'!' | b'(' | b')' | b'*' if mode == ComponentEscapeMode::Fragment => false,
        _ => true,
    }
}

fn parse_authority_fields(authority: &str) -> Result<(Option<ParsedUserinfoFields>, &str), String> {
    let Some(index) = authority.rfind('@') else {
        return Ok((None, authority));
    };

    let userinfo = &authority[..index];
    if !valid_userinfo_text(userinfo) {
        return Err("invalid userinfo".into());
    }

    let host = &authority[index + 1..];
    if let Some((username, password)) = userinfo.split_once(':') {
        return Ok((
            Some(ParsedUserinfoFields {
                username: unescape_component(username, ComponentEscapeMode::Userinfo)?,
                password: unescape_component(password, ComponentEscapeMode::Userinfo)?,
                password_set: true,
            }),
            host,
        ));
    }

    Ok((
        Some(ParsedUserinfoFields {
            username: unescape_component(userinfo, ComponentEscapeMode::Userinfo)?,
            password: String::new(),
            password_set: false,
        }),
        host,
    ))
}

fn valid_userinfo_text(text: &str) -> bool {
    text.chars().all(|r| {
        r.is_ascii_alphanumeric()
            || matches!(
                r,
                '-' | '.'
                    | '_'
                    | ':'
                    | '~'
                    | '!'
                    | '$'
                    | '&'
                    | '\''
                    | '('
                    | ')'
                    | '*'
                    | '+'
                    | ','
                    | ';'
                    | '='
                    | '%'
                    | '@'
            )
    })
}

fn invalid_url_escape(bytes: &[u8], index: usize) -> String {
    let end = bytes.len().min(index + 3);
    format!(
        "invalid URL escape {:?}",
        String::from_utf8_lossy(&bytes[index..end])
    )
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn upper_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + (nibble - 10)) as char,
        _ => unreachable!("nibble should be in 0..=15"),
    }
}
