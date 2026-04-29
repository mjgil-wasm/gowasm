use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn instruction_enum_names() -> BTreeSet<String> {
    let source = fs::read_to_string(repo_root().join("crates/vm/src/instruction.rs"))
        .expect("instruction source should exist");
    let body = extract_rust_block(&source, "pub enum Instruction");
    let mut names = BTreeSet::new();
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
                    names.insert(token.clone());
                }
                token.clear();
            }
        } else if !token.is_empty() {
            token.clear();
        }
    }
    names
}

fn documented_instruction_names(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("| `") {
                return None;
            }
            let rest = &line[3..];
            let end = rest.find('`')?;
            Some(rest[..end].to_string())
        })
        .collect()
}

fn validate_instruction_spec(
    markdown: &str,
    actual_names: &BTreeSet<String>,
) -> Result<(), String> {
    let documented = documented_instruction_names(markdown);
    let documented_set: BTreeSet<_> = documented.into_iter().collect();

    let unknown: Vec<_> = documented_set.difference(actual_names).cloned().collect();
    if !unknown.is_empty() {
        return Err(format!(
            "unknown instructions in spec: {}",
            unknown.join(", ")
        ));
    }

    let missing: Vec<_> = actual_names.difference(&documented_set).cloned().collect();
    if !missing.is_empty() {
        return Err(format!(
            "missing instructions from spec: {}",
            missing.join(", ")
        ));
    }

    Ok(())
}

#[test]
fn bytecode_vm_spec_matches_instruction_enum_surface() {
    let markdown = fs::read_to_string(repo_root().join("docs/bytecode-vm-spec.md"))
        .expect("bytecode vm spec should exist");
    let actual = instruction_enum_names();
    validate_instruction_spec(&markdown, &actual)
        .expect("bytecode vm spec should match the instruction enum surface");
}

#[test]
fn bytecode_vm_spec_validation_rejects_unknown_instruction_names() {
    let actual = BTreeSet::from(["LoadInt".to_string(), "Return".to_string()]);
    let markdown = "\
| Instruction | Operands | Purpose |\n\
| --- | --- | --- |\n\
| `LoadInt` | `dst`, `value` | ok |\n\
| `BogusInstruction` | `dst` | bad |\n\
| `Return` | `src` | ok |\n";

    let error = validate_instruction_spec(markdown, &actual)
        .expect_err("validation should reject unknown instruction names");
    assert!(error.contains("BogusInstruction"));
}

#[test]
fn bytecode_vm_spec_validation_rejects_missing_instruction_names() {
    let actual = BTreeSet::from(["LoadInt".to_string(), "Return".to_string()]);
    let markdown = "\
| Instruction | Operands | Purpose |\n\
| --- | --- | --- |\n\
| `LoadInt` | `dst`, `value` | ok |\n";

    let error = validate_instruction_spec(markdown, &actual)
        .expect_err("validation should reject missing instruction names");
    assert!(error.contains("Return"));
}
