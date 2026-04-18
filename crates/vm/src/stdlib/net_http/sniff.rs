use super::super::bytes_impl::extract_bytes;
use crate::{Program, Value, Vm, VmError};

const SNIFF_LEN: usize = 512;

enum SniffSignature {
    Html(&'static [u8]),
    Masked {
        mask: &'static [u8],
        pat: &'static [u8],
        skip_ws: bool,
        ct: &'static str,
    },
    Exact {
        sig: &'static [u8],
        ct: &'static str,
    },
    Mp4,
    Text,
}

const SNIFF_SIGNATURES: &[SniffSignature] = &[
    SniffSignature::Html(b"<!DOCTYPE HTML"),
    SniffSignature::Html(b"<HTML"),
    SniffSignature::Html(b"<HEAD"),
    SniffSignature::Html(b"<SCRIPT"),
    SniffSignature::Html(b"<IFRAME"),
    SniffSignature::Html(b"<H1"),
    SniffSignature::Html(b"<DIV"),
    SniffSignature::Html(b"<FONT"),
    SniffSignature::Html(b"<TABLE"),
    SniffSignature::Html(b"<A"),
    SniffSignature::Html(b"<STYLE"),
    SniffSignature::Html(b"<TITLE"),
    SniffSignature::Html(b"<B"),
    SniffSignature::Html(b"<BODY"),
    SniffSignature::Html(b"<BR"),
    SniffSignature::Html(b"<P"),
    SniffSignature::Html(b"<!--"),
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\xFF",
        pat: b"<?xml",
        skip_ws: true,
        ct: "text/xml; charset=utf-8",
    },
    SniffSignature::Exact {
        sig: b"%PDF-",
        ct: "application/pdf",
    },
    SniffSignature::Exact {
        sig: b"%!PS-Adobe-",
        ct: "application/postscript",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\x00\x00",
        pat: b"\xFE\xFF\x00\x00",
        skip_ws: false,
        ct: "text/plain; charset=utf-16be",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\x00\x00",
        pat: b"\xFF\xFE\x00\x00",
        skip_ws: false,
        ct: "text/plain; charset=utf-16le",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\x00",
        pat: b"\xEF\xBB\xBF\x00",
        skip_ws: false,
        ct: "text/plain; charset=utf-8",
    },
    SniffSignature::Exact {
        sig: b"\x00\x00\x01\x00",
        ct: "image/x-icon",
    },
    SniffSignature::Exact {
        sig: b"\x00\x00\x02\x00",
        ct: "image/x-icon",
    },
    SniffSignature::Exact {
        sig: b"BM",
        ct: "image/bmp",
    },
    SniffSignature::Exact {
        sig: b"GIF87a",
        ct: "image/gif",
    },
    SniffSignature::Exact {
        sig: b"GIF89a",
        ct: "image/gif",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF",
        pat: b"RIFF\x00\x00\x00\x00WEBPVP",
        skip_ws: false,
        ct: "image/webp",
    },
    SniffSignature::Exact {
        sig: b"\x89PNG\x0D\x0A\x1A\x0A",
        ct: "image/png",
    },
    SniffSignature::Exact {
        sig: b"\xFF\xD8\xFF",
        ct: "image/jpeg",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
        pat: b"FORM\x00\x00\x00\x00AIFF",
        skip_ws: false,
        ct: "audio/aiff",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF",
        pat: b"ID3",
        skip_ws: false,
        ct: "audio/mpeg",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\xFF",
        pat: b"OggS\x00",
        skip_ws: false,
        ct: "application/ogg",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",
        pat: b"MThd\x00\x00\x00\x06",
        skip_ws: false,
        ct: "audio/midi",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
        pat: b"RIFF\x00\x00\x00\x00AVI ",
        skip_ws: false,
        ct: "video/avi",
    },
    SniffSignature::Masked {
        mask: b"\xFF\xFF\xFF\xFF\x00\x00\x00\x00\xFF\xFF\xFF\xFF",
        pat: b"RIFF\x00\x00\x00\x00WAVE",
        skip_ws: false,
        ct: "audio/wave",
    },
    SniffSignature::Mp4,
    SniffSignature::Exact {
        sig: b"\x1A\x45\xDF\xA3",
        ct: "video/webm",
    },
    SniffSignature::Masked {
        pat: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00LP",
        mask: b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF",
        skip_ws: false,
        ct: "application/vnd.ms-fontobject",
    },
    SniffSignature::Exact {
        sig: b"\x00\x01\x00\x00",
        ct: "font/ttf",
    },
    SniffSignature::Exact {
        sig: b"OTTO",
        ct: "font/otf",
    },
    SniffSignature::Exact {
        sig: b"ttcf",
        ct: "font/collection",
    },
    SniffSignature::Exact {
        sig: b"wOFF",
        ct: "font/woff",
    },
    SniffSignature::Exact {
        sig: b"wOF2",
        ct: "font/woff2",
    },
    SniffSignature::Exact {
        sig: b"\x1F\x8B\x08",
        ct: "application/x-gzip",
    },
    SniffSignature::Exact {
        sig: b"PK\x03\x04",
        ct: "application/zip",
    },
    SniffSignature::Exact {
        sig: b"Rar!\x1A\x07\x00",
        ct: "application/x-rar-compressed",
    },
    SniffSignature::Exact {
        sig: b"Rar!\x1A\x07\x01\x00",
        ct: "application/x-rar-compressed",
    },
    SniffSignature::Exact {
        sig: b"\x00\x61\x73\x6D",
        ct: "application/wasm",
    },
    SniffSignature::Text,
];

pub(super) fn detect_content_type(
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
    let data = extract_bytes(vm, program, "http.DetectContentType", &args[0])?;
    Ok(Value::string(detect_content_type_bytes(&data)))
}

fn detect_content_type_bytes(data: &[u8]) -> &'static str {
    let data = if data.len() > SNIFF_LEN {
        &data[..SNIFF_LEN]
    } else {
        data
    };

    let mut first_non_ws = 0;
    while first_non_ws < data.len() && is_ws(data[first_non_ws]) {
        first_non_ws += 1;
    }

    for signature in SNIFF_SIGNATURES {
        if let Some(content_type) = match_signature(signature, data, first_non_ws) {
            return content_type;
        }
    }

    "application/octet-stream"
}

fn match_signature(
    signature: &SniffSignature,
    data: &[u8],
    first_non_ws: usize,
) -> Option<&'static str> {
    match signature {
        SniffSignature::Html(pattern) => {
            match_html_signature(pattern, data, first_non_ws).then_some("text/html; charset=utf-8")
        }
        SniffSignature::Masked {
            mask,
            pat,
            skip_ws,
            ct,
        } => match_masked_signature(mask, pat, *skip_ws, data, first_non_ws).then_some(*ct),
        SniffSignature::Exact { sig, ct } => data.starts_with(sig).then_some(*ct),
        SniffSignature::Mp4 => match_mp4_signature(data).then_some("video/mp4"),
        SniffSignature::Text => {
            match_text_signature(data, first_non_ws).then_some("text/plain; charset=utf-8")
        }
    }
}

fn match_html_signature(pattern: &[u8], data: &[u8], first_non_ws: usize) -> bool {
    let data = &data[first_non_ws..];
    if data.len() < pattern.len() + 1 {
        return false;
    }

    for (index, pattern_byte) in pattern.iter().enumerate() {
        let data_byte = if pattern_byte.is_ascii_uppercase() {
            data[index].to_ascii_uppercase()
        } else {
            data[index]
        };
        if *pattern_byte != data_byte {
            return false;
        }
    }

    is_tt(data[pattern.len()])
}

fn match_masked_signature(
    mask: &[u8],
    pattern: &[u8],
    skip_ws: bool,
    data: &[u8],
    first_non_ws: usize,
) -> bool {
    let data = if skip_ws { &data[first_non_ws..] } else { data };
    if mask.len() != pattern.len() || data.len() < pattern.len() {
        return false;
    }

    for index in 0..pattern.len() {
        if data[index] & mask[index] != pattern[index] {
            return false;
        }
    }

    true
}

fn match_mp4_signature(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    let box_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < box_size || !box_size.is_multiple_of(4) {
        return false;
    }
    if &data[4..8] != b"ftyp" {
        return false;
    }

    for start in (8..box_size).step_by(4) {
        if start == 12 {
            continue;
        }
        if &data[start..start + 3] == b"mp4" {
            return true;
        }
    }

    false
}

fn match_text_signature(data: &[u8], first_non_ws: usize) -> bool {
    for byte in &data[first_non_ws..] {
        match *byte {
            0x00..=0x08 | 0x0B | 0x0E..=0x1A | 0x1C..=0x1F => return false,
            _ => {}
        }
    }

    true
}

fn is_ws(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | 0x0C | b'\r' | b' ')
}

fn is_tt(byte: u8) -> bool {
    matches!(byte, b' ' | b'>')
}
