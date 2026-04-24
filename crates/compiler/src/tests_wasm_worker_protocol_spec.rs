use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("source file should exist")
}

fn extract_rust_block(source: &str, marker: &str) -> String {
    let start = source.find(marker).expect("marker should exist");
    let brace_start = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .expect("block should open");
    let mut depth = 0usize;
    for (offset, ch) in source[brace_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return source[brace_start + 1..brace_start + offset].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("unterminated block for `{marker}`");
}

fn extract_enum_variants(source: &str, marker: &str) -> Vec<String> {
    let body = extract_rust_block(source, marker);
    let mut variants = Vec::new();
    let mut depth = 0usize;
    let mut token = String::new();
    for ch in body.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                token.push(ch);
                continue;
            }
            if !token.is_empty() {
                if token
                    .chars()
                    .next()
                    .expect("token should not be empty")
                    .is_ascii_uppercase()
                {
                    variants.push(token.clone());
                }
                token.clear();
            }
        } else if !token.is_empty() {
            token.clear();
        }
    }
    variants
}

fn extract_status_codes(source: &str) -> BTreeMap<String, String> {
    let body = extract_rust_block(source, "enum WasmAbiStatus");
    let mut codes = BTreeMap::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((name, value)) = line.split_once('=') {
            let name = name.trim().trim_end_matches(',');
            let value = value.trim().trim_end_matches(',');
            if name
                .chars()
                .next()
                .expect("status name should not be empty")
                .is_ascii_uppercase()
            {
                codes.insert(name.to_string(), value.to_string());
            }
        }
    }
    codes
}

fn extract_marked_names(markdown: &str, start: &str, end: &str) -> Vec<String> {
    let section = markdown
        .split(start)
        .nth(1)
        .and_then(|tail| tail.split(end).next())
        .expect("marked section should exist");
    section
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("- `") {
                return None;
            }
            let rest = &line[3..];
            let end = rest.find('`')?;
            Some(rest[..end].to_string())
        })
        .collect()
}

fn extract_marked_status_codes(markdown: &str) -> BTreeMap<String, String> {
    let section = markdown
        .split("<!-- wasm-abi-status-codes:start -->")
        .nth(1)
        .and_then(|tail| tail.split("<!-- wasm-abi-status-codes:end -->").next())
        .expect("status code section should exist");
    let mut statuses = BTreeMap::new();
    for line in section.lines() {
        let line = line.trim();
        if !line.starts_with("| `") {
            continue;
        }
        let cells: Vec<_> = line
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect();
        if cells.len() >= 2 {
            statuses.insert(
                cells[1].trim_matches('`').to_string(),
                cells[0].trim_matches('`').to_string(),
            );
        }
    }
    statuses
}

#[test]
fn wasm_worker_protocol_spec_matches_engine_and_host_surfaces() {
    let markdown = read("docs/wasm-worker-protocol.md");
    let host_types = read("crates/host-types/src/lib.rs");
    let module_protocol = read("crates/host-types/src/module_protocol.rs");
    let engine_wasm = read("crates/engine-wasm/src/lib.rs");

    let actual_requests = extract_enum_variants(&host_types, "pub enum EngineRequest");
    let actual_responses = extract_enum_variants(&host_types, "pub enum EngineResponse");
    let actual_cap_requests = extract_enum_variants(&host_types, "pub enum CapabilityRequest");
    let actual_cap_results = extract_enum_variants(&host_types, "pub enum CapabilityResult");
    let actual_module_requests = extract_enum_variants(&module_protocol, "pub enum ModuleRequest");
    let actual_module_results = extract_enum_variants(&module_protocol, "pub enum ModuleResult");
    let actual_statuses = extract_status_codes(&engine_wasm);

    assert_eq!(
        extract_marked_names(
            &markdown,
            "<!-- engine-request-kinds:start -->",
            "<!-- engine-request-kinds:end -->"
        ),
        actual_requests
    );
    assert_eq!(
        extract_marked_names(
            &markdown,
            "<!-- engine-response-kinds:start -->",
            "<!-- engine-response-kinds:end -->"
        ),
        actual_responses
    );
    assert_eq!(
        extract_marked_names(
            &markdown,
            "<!-- capability-request-kinds:start -->",
            "<!-- capability-request-kinds:end -->"
        ),
        actual_cap_requests
    );
    assert_eq!(
        extract_marked_names(
            &markdown,
            "<!-- capability-result-kinds:start -->",
            "<!-- capability-result-kinds:end -->"
        ),
        actual_cap_results
    );
    assert_eq!(
        extract_marked_names(
            &markdown,
            "<!-- module-request-kinds:start -->",
            "<!-- module-request-kinds:end -->"
        ),
        actual_module_requests
    );
    assert_eq!(
        extract_marked_names(
            &markdown,
            "<!-- module-result-kinds:start -->",
            "<!-- module-result-kinds:end -->"
        ),
        actual_module_results
    );
    assert_eq!(extract_marked_status_codes(&markdown), actual_statuses);
}
