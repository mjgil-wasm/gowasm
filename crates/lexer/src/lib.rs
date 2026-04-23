use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Package,
    Import,
    Type,
    Const,
    Struct,
    Interface,
    Map,
    Chan,
    Func,
    Var,
    If,
    Else,
    For,
    Range,
    Switch,
    Select,
    Case,
    Default,
    Break,
    Continue,
    Fallthrough,
    Return,
    Go,
    Defer,
    True,
    False,
    Nil,
    Ident(String),
    Int(i64),
    Float(u64),
    String(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Dot,
    Ellipsis,
    Comma,
    Colon,
    PlusPlus,
    PlusEqual,
    Plus,
    MinusMinus,
    MinusEqual,
    Minus,
    StarEqual,
    Star,
    SlashEqual,
    Slash,
    PercentEqual,
    Percent,
    ColonEqual,
    Bang,
    Equal,
    EqualEqual,
    BangEqual,
    CaretEqual,
    Caret,
    AndAnd,
    BitAndEqual,
    BitAnd,
    BitClear,
    BitOrEqual,
    BitOr,
    Arrow,
    OrOr,
    ShiftLeftEqual,
    ShiftLeft,
    ShiftRightEqual,
    ShiftRight,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Semicolon,
    Eof,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LexError {
    #[error("unexpected character `{character}` at byte {offset}")]
    UnexpectedCharacter { character: char, offset: usize },
    #[error("unterminated string literal starting at byte {offset}")]
    UnterminatedString { offset: usize },
    #[error("unsupported escape sequence `\\{character}` at byte {offset}")]
    UnsupportedEscape { character: char, offset: usize },
    #[error("integer literal at byte {offset} does not fit in i64")]
    InvalidInteger { offset: usize },
    #[error("invalid float literal at byte {offset}")]
    InvalidFloat { offset: usize },
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    let mut tokens = Vec::new();

    while cursor < bytes.len() {
        let mut saw_newline = false;
        loop {
            let Some(byte) = bytes.get(cursor).copied() else {
                break;
            };

            if is_whitespace(byte) {
                if byte == b'\n' {
                    saw_newline = true;
                }
                cursor += 1;
                continue;
            }

            if byte == b'/' && peek_byte(bytes, cursor + 1) == Some(b'/') {
                cursor = skip_line_comment(bytes, cursor + 2);
                continue;
            }

            break;
        }

        if saw_newline
            && tokens
                .last()
                .map(|token: &Token| token_allows_implicit_semicolon(&token.kind))
                .unwrap_or(false)
        {
            tokens.push(Token {
                kind: TokenKind::Semicolon,
                span: Span {
                    start: cursor,
                    end: cursor,
                },
            });
        }

        if cursor >= bytes.len() {
            break;
        }

        let byte = bytes[cursor];
        let start = cursor;
        let kind = match byte {
            b'(' => {
                cursor += 1;
                TokenKind::LParen
            }
            b')' => {
                cursor += 1;
                TokenKind::RParen
            }
            b'[' => {
                cursor += 1;
                TokenKind::LBracket
            }
            b']' => {
                cursor += 1;
                TokenKind::RBracket
            }
            b'{' => {
                cursor += 1;
                TokenKind::LBrace
            }
            b'}' => {
                cursor += 1;
                TokenKind::RBrace
            }
            b'.' => {
                if bytes.get(cursor + 1..cursor + 3) == Some(b"..") {
                    cursor += 3;
                    TokenKind::Ellipsis
                } else {
                    cursor += 1;
                    TokenKind::Dot
                }
            }
            b',' => {
                cursor += 1;
                TokenKind::Comma
            }
            b':' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::ColonEqual
            }
            b':' => {
                cursor += 1;
                TokenKind::Colon
            }
            b'+' if peek_byte(bytes, cursor + 1) == Some(b'+') => {
                cursor += 2;
                TokenKind::PlusPlus
            }
            b'+' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::PlusEqual
            }
            b'+' => {
                cursor += 1;
                TokenKind::Plus
            }
            b'-' if peek_byte(bytes, cursor + 1) == Some(b'-') => {
                cursor += 2;
                TokenKind::MinusMinus
            }
            b'-' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::MinusEqual
            }
            b'-' => {
                cursor += 1;
                TokenKind::Minus
            }
            b'*' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::StarEqual
            }
            b'*' => {
                cursor += 1;
                TokenKind::Star
            }
            b'/' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::SlashEqual
            }
            b'/' => {
                cursor += 1;
                TokenKind::Slash
            }
            b'%' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::PercentEqual
            }
            b'%' => {
                cursor += 1;
                TokenKind::Percent
            }
            b'!' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::BangEqual
            }
            b'!' => {
                cursor += 1;
                TokenKind::Bang
            }
            b'=' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::EqualEqual
            }
            b'=' => {
                cursor += 1;
                TokenKind::Equal
            }
            b'^' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::CaretEqual
            }
            b'^' => {
                cursor += 1;
                TokenKind::Caret
            }
            b'&' if peek_byte(bytes, cursor + 1) == Some(b'^') => {
                cursor += 2;
                TokenKind::BitClear
            }
            b'&' if peek_byte(bytes, cursor + 1) == Some(b'&') => {
                cursor += 2;
                TokenKind::AndAnd
            }
            b'&' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::BitAndEqual
            }
            b'&' => {
                cursor += 1;
                TokenKind::BitAnd
            }
            b'|' if peek_byte(bytes, cursor + 1) == Some(b'|') => {
                cursor += 2;
                TokenKind::OrOr
            }
            b'|' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::BitOrEqual
            }
            b'|' => {
                cursor += 1;
                TokenKind::BitOr
            }
            b'<' if peek_byte(bytes, cursor + 1) == Some(b'<')
                && peek_byte(bytes, cursor + 2) == Some(b'=') =>
            {
                cursor += 3;
                TokenKind::ShiftLeftEqual
            }
            b'<' if peek_byte(bytes, cursor + 1) == Some(b'<') => {
                cursor += 2;
                TokenKind::ShiftLeft
            }
            b'>' if peek_byte(bytes, cursor + 1) == Some(b'>')
                && peek_byte(bytes, cursor + 2) == Some(b'=') =>
            {
                cursor += 3;
                TokenKind::ShiftRightEqual
            }
            b'>' if peek_byte(bytes, cursor + 1) == Some(b'>') => {
                cursor += 2;
                TokenKind::ShiftRight
            }
            b'<' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::LessEqual
            }
            b'<' if peek_byte(bytes, cursor + 1) == Some(b'-') => {
                cursor += 2;
                TokenKind::Arrow
            }
            b'<' => {
                cursor += 1;
                TokenKind::Less
            }
            b'>' if peek_byte(bytes, cursor + 1) == Some(b'=') => {
                cursor += 2;
                TokenKind::GreaterEqual
            }
            b'>' => {
                cursor += 1;
                TokenKind::Greater
            }
            b';' => {
                cursor += 1;
                TokenKind::Semicolon
            }
            b'"' => {
                let (value, next) = read_string(source, cursor)?;
                cursor = next;
                TokenKind::String(value)
            }
            b'`' => {
                let (value, next) = read_raw_string(source, cursor)?;
                cursor = next;
                TokenKind::String(value)
            }
            b'\'' => {
                let (value, next) = read_rune(source, cursor)?;
                cursor = next;
                TokenKind::Int(value)
            }
            b'0'..=b'9' => {
                let (kind, next) = read_number(source, cursor)?;
                cursor = next;
                kind
            }
            _ if is_ident_start(byte) => {
                let (value, next) = read_identifier(source, cursor);
                cursor = next;
                match value.as_str() {
                    "package" => TokenKind::Package,
                    "import" => TokenKind::Import,
                    "type" => TokenKind::Type,
                    "const" => TokenKind::Const,
                    "struct" => TokenKind::Struct,
                    "interface" => TokenKind::Interface,
                    "map" => TokenKind::Map,
                    "chan" => TokenKind::Chan,
                    "func" => TokenKind::Func,
                    "var" => TokenKind::Var,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "for" => TokenKind::For,
                    "range" => TokenKind::Range,
                    "switch" => TokenKind::Switch,
                    "select" => TokenKind::Select,
                    "case" => TokenKind::Case,
                    "default" => TokenKind::Default,
                    "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue,
                    "fallthrough" => TokenKind::Fallthrough,
                    "return" => TokenKind::Return,
                    "go" => TokenKind::Go,
                    "defer" => TokenKind::Defer,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "nil" => TokenKind::Nil,
                    _ => TokenKind::Ident(value),
                }
            }
            _ => {
                return Err(LexError::UnexpectedCharacter {
                    character: source[cursor..].chars().next().unwrap_or('\0'),
                    offset: cursor,
                });
            }
        };

        tokens.push(Token {
            kind,
            span: Span { start, end: cursor },
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            start: source.len(),
            end: source.len(),
        },
    });
    Ok(tokens)
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

fn token_allows_implicit_semicolon(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident(_)
            | TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::String(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Nil
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Return
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
    )
}

fn is_ident_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_ident_continue(byte: u8) -> bool {
    is_ident_start(byte) || byte.is_ascii_digit()
}

fn peek_byte(bytes: &[u8], index: usize) -> Option<u8> {
    bytes.get(index).copied()
}

fn skip_line_comment(bytes: &[u8], mut cursor: usize) -> usize {
    while let Some(byte) = peek_byte(bytes, cursor) {
        cursor += 1;
        if byte == b'\n' {
            break;
        }
    }
    cursor
}

fn read_identifier(source: &str, mut cursor: usize) -> (String, usize) {
    let start = cursor;
    let bytes = source.as_bytes();
    cursor += 1;
    while let Some(byte) = peek_byte(bytes, cursor) {
        if !is_ident_continue(byte) {
            break;
        }
        cursor += 1;
    }
    (source[start..cursor].to_string(), cursor)
}

fn read_number(source: &str, mut cursor: usize) -> Result<(TokenKind, usize), LexError> {
    let start = cursor;
    let bytes = source.as_bytes();

    if bytes[cursor] == b'0' && matches!(peek_byte(bytes, cursor + 1), Some(b'x' | b'X')) {
        cursor += 2;
        let hex_start = cursor;
        while let Some(byte) = peek_byte(bytes, cursor) {
            if !byte.is_ascii_hexdigit() {
                break;
            }
            cursor += 1;
        }
        if cursor == hex_start {
            return Err(LexError::InvalidInteger { offset: start });
        }
        let value = i64::from_str_radix(&source[hex_start..cursor], 16)
            .map_err(|_| LexError::InvalidInteger { offset: start })?;
        return Ok((TokenKind::Int(value), cursor));
    }

    cursor += 1;
    while let Some(byte) = peek_byte(bytes, cursor) {
        if !byte.is_ascii_digit() {
            break;
        }
        cursor += 1;
    }

    let has_dot = peek_byte(bytes, cursor) == Some(b'.')
        && peek_byte(bytes, cursor + 1)
            .map(|b| b.is_ascii_digit())
            .unwrap_or(false);

    if !has_dot {
        let has_exp = matches!(peek_byte(bytes, cursor), Some(b'e' | b'E'));
        if !has_exp {
            let value = source[start..cursor]
                .parse::<i64>()
                .map_err(|_| LexError::InvalidInteger { offset: start })?;
            return Ok((TokenKind::Int(value), cursor));
        }
    }

    if has_dot {
        cursor += 1;
        while let Some(byte) = peek_byte(bytes, cursor) {
            if !byte.is_ascii_digit() {
                break;
            }
            cursor += 1;
        }
    }

    if matches!(peek_byte(bytes, cursor), Some(b'e' | b'E')) {
        cursor += 1;
        if matches!(peek_byte(bytes, cursor), Some(b'+' | b'-')) {
            cursor += 1;
        }
        let exp_start = cursor;
        while let Some(byte) = peek_byte(bytes, cursor) {
            if !byte.is_ascii_digit() {
                break;
            }
            cursor += 1;
        }
        if cursor == exp_start {
            return Err(LexError::InvalidFloat { offset: start });
        }
    }

    let value: f64 = source[start..cursor]
        .parse()
        .map_err(|_| LexError::InvalidFloat { offset: start })?;
    Ok((TokenKind::Float(value.to_bits()), cursor))
}

fn read_string(source: &str, mut cursor: usize) -> Result<(String, usize), LexError> {
    let start = cursor;
    let bytes = source.as_bytes();
    cursor += 1;

    let mut value = String::new();
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        match byte {
            b'"' => return Ok((value, cursor + 1)),
            b'\\' => {
                let escape_offset = cursor;
                cursor += 1;
                let escaped = peek_byte(bytes, cursor)
                    .ok_or(LexError::UnterminatedString { offset: start })?;
                let escaped = match escaped {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'n' => '\n',
                    b't' => '\t',
                    other => {
                        return Err(LexError::UnsupportedEscape {
                            character: other as char,
                            offset: escape_offset,
                        });
                    }
                };
                value.push(escaped);
                cursor += 1;
            }
            _ => {
                let remaining = &source[cursor..];
                let character = remaining
                    .chars()
                    .next()
                    .ok_or(LexError::UnterminatedString { offset: start })?;
                value.push(character);
                cursor += character.len_utf8();
            }
        }
    }

    Err(LexError::UnterminatedString { offset: start })
}

fn read_raw_string(source: &str, cursor: usize) -> Result<(String, usize), LexError> {
    let start = cursor;
    let end = source[cursor + 1..]
        .find('`')
        .map(|pos| cursor + 1 + pos)
        .ok_or(LexError::UnterminatedString { offset: start })?;
    let value = source[cursor + 1..end].to_string();
    Ok((value, end + 1))
}

fn read_rune(source: &str, cursor: usize) -> Result<(i64, usize), LexError> {
    let start = cursor;
    let bytes = source.as_bytes();
    let mut pos = cursor + 1;
    let ch = if peek_byte(bytes, pos) == Some(b'\\') {
        pos += 1;
        let escaped =
            peek_byte(bytes, pos).ok_or(LexError::UnterminatedString { offset: start })?;
        pos += 1;
        match escaped {
            b'n' => '\n',
            b't' => '\t',
            b'r' => '\r',
            b'\\' => '\\',
            b'\'' => '\'',
            b'0' => '\0',
            other => {
                return Err(LexError::UnsupportedEscape {
                    character: other as char,
                    offset: start + 1,
                });
            }
        }
    } else {
        let remaining = &source[pos..];
        let c = remaining
            .chars()
            .next()
            .ok_or(LexError::UnterminatedString { offset: start })?;
        pos += c.len_utf8();
        c
    };
    if peek_byte(bytes, pos) != Some(b'\'') {
        return Err(LexError::UnterminatedString { offset: start });
    }
    Ok((ch as i64, pos + 1))
}

#[cfg(test)]
mod tests;
