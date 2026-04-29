use icu_casemap::{CaseMapper, ClosureSink};
use icu_properties::{props::WhiteSpace, CodePointSetData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ByteRune {
    pub(super) byte_index: usize,
    pub(super) byte_width: usize,
    pub(super) ch: char,
}

impl ByteRune {
    pub(super) fn end(self) -> usize {
        self.byte_index + self.byte_width
    }
}

#[derive(Default)]
struct CharClosure {
    chars: Vec<char>,
}

impl ClosureSink for CharClosure {
    fn add_char(&mut self, c: char) {
        self.chars.push(c);
    }

    fn add_string(&mut self, _string: &str) {}
}

pub(super) fn byte_runes(bytes: &[u8]) -> Vec<ByteRune> {
    let mut runes = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        if let Some(width) = valid_utf8_width(&bytes[index..]) {
            let chunk = std::str::from_utf8(&bytes[index..index + width])
                .expect("valid utf-8 width should decode");
            let ch = chunk
                .chars()
                .next()
                .expect("valid utf-8 chunk should decode");
            runes.push(ByteRune {
                byte_index: index,
                byte_width: width,
                ch,
            });
            index += width;
        } else {
            runes.push(ByteRune {
                byte_index: index,
                byte_width: 1,
                ch: '\u{FFFD}',
            });
            index += 1;
        }
    }

    runes
}

pub(super) fn split_empty_sep_bytes(bytes: &[u8], count: i64) -> Vec<Vec<u8>> {
    let runes = byte_runes(bytes);
    if count < 0 || count as usize >= runes.len() {
        return runes
            .into_iter()
            .map(|rune| bytes[rune.byte_index..rune.end()].to_vec())
            .collect();
    }
    if count == 1 {
        return vec![bytes.to_vec()];
    }

    let split_at = runes[(count - 1) as usize].byte_index;
    let mut parts = runes
        .into_iter()
        .take((count - 1) as usize)
        .map(|rune| bytes[rune.byte_index..rune.end()].to_vec())
        .collect::<Vec<_>>();
    parts.push(bytes[split_at..].to_vec());
    parts
}

pub(super) fn empty_match_boundaries(bytes: &[u8]) -> Vec<usize> {
    let mut boundaries = Vec::new();
    boundaries.push(0);
    boundaries.extend(byte_runes(bytes).into_iter().map(ByteRune::end));
    boundaries
}

pub(super) fn unicode_space(ch: char) -> bool {
    match ch {
        '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r' | ' ' | '\u{0085}' | '\u{00A0}' => true,
        _ if (ch as u32) <= 0xFF => false,
        _ => CodePointSetData::new::<WhiteSpace>().contains(ch),
    }
}

pub(super) fn equal_fold_bytes(left: &[u8], right: &[u8]) -> bool {
    let mut left_runes = byte_runes(left).into_iter();
    let mut right_runes = byte_runes(right).into_iter();
    loop {
        match (left_runes.next(), right_runes.next()) {
            (None, None) => return true,
            (Some(left), Some(right)) if chars_equal_fold(left.ch, right.ch) => {}
            _ => return false,
        }
    }
}

pub(super) fn map_runes_to_bytes<F>(bytes: &[u8], mut mapper: F) -> Vec<u8>
where
    F: FnMut(char) -> Option<char>,
{
    let mut out = Vec::new();
    for rune in byte_runes(bytes) {
        if let Some(ch) = mapper(rune.ch) {
            let mut encoded = [0u8; 4];
            out.extend_from_slice(ch.encode_utf8(&mut encoded).as_bytes());
        }
    }
    out
}

pub(super) fn titlecase_rune(ch: char) -> char {
    CaseMapper::new().simple_titlecase(ch)
}

fn chars_equal_fold(left: char, right: char) -> bool {
    if left == right {
        return true;
    }

    let mut closure = CharClosure::default();
    closure.chars.push(left);
    CaseMapper::new().add_case_closure_to(left, &mut closure);
    closure.chars.contains(&right)
}

fn valid_utf8_width(bytes: &[u8]) -> Option<usize> {
    let first = *bytes.first()?;
    match first {
        0x00..=0x7f => Some(1),
        0xc2..=0xdf => continuation(bytes, 1).then_some(2),
        0xe0 => continuation_in_range(bytes, 1, 0xa0, 0xbf)
            .filter(|_| continuation(bytes, 2))
            .map(|_| 3),
        0xe1..=0xec | 0xee..=0xef => continuation(bytes, 1)
            .then_some(())
            .filter(|_| continuation(bytes, 2))
            .map(|_| 3),
        0xed => continuation_in_range(bytes, 1, 0x80, 0x9f)
            .filter(|_| continuation(bytes, 2))
            .map(|_| 3),
        0xf0 => continuation_in_range(bytes, 1, 0x90, 0xbf)
            .filter(|_| continuation(bytes, 2))
            .filter(|_| continuation(bytes, 3))
            .map(|_| 4),
        0xf1..=0xf3 => continuation(bytes, 1)
            .then_some(())
            .filter(|_| continuation(bytes, 2))
            .filter(|_| continuation(bytes, 3))
            .map(|_| 4),
        0xf4 => continuation_in_range(bytes, 1, 0x80, 0x8f)
            .filter(|_| continuation(bytes, 2))
            .filter(|_| continuation(bytes, 3))
            .map(|_| 4),
        _ => None,
    }
}

fn continuation(bytes: &[u8], index: usize) -> bool {
    matches!(bytes.get(index), Some(value) if (0x80..=0xbf).contains(value))
}

fn continuation_in_range(bytes: &[u8], index: usize, low: u8, high: u8) -> Option<()> {
    matches!(bytes.get(index), Some(value) if (low..=high).contains(value)).then_some(())
}
