use gowasm_host_types::{Diagnostic, ErrorCategory, WorkspaceFile};
use gowasm_parser::parse_source_file;

pub(super) fn format_workspace_files(
    files: &[WorkspaceFile],
) -> (Vec<WorkspaceFile>, Vec<Diagnostic>) {
    let mut formatted = Vec::with_capacity(files.len());
    let mut diagnostics = Vec::new();

    for file in files {
        if !file.path.ends_with(".go") {
            formatted.push(file.clone());
            continue;
        }

        if let Err(error) = parse_source_file(&file.contents) {
            diagnostics.push(go_parse_diagnostic(&file.path, "format", error.to_string()));
            formatted.push(file.clone());
            continue;
        }

        formatted.push(WorkspaceFile {
            path: file.path.clone(),
            contents: format_go_source(&file.contents),
        });
    }

    (formatted, diagnostics)
}

pub(super) fn go_parse_diagnostic(path: &str, action: &str, message: String) -> Diagnostic {
    let mut diagnostic = Diagnostic::error_with_category(
        ErrorCategory::Tooling,
        format!("cannot {action} `{path}` until it parses cleanly: {message}"),
    );
    diagnostic.file_path = Some(path.into());
    diagnostic.suggested_action = Some(format!("Fix the parse error before running {action}."));
    diagnostic
}

pub(super) fn format_go_source(source: &str) -> String {
    let normalized = normalize_line_endings(source);
    let mut formatted = String::new();
    let mut indent_depth = 0usize;
    let mut line_state = LineState::Code;

    for line in normalized.lines() {
        let analysis = analyze_line(line, line_state);

        if let LineState::BlockComment { indent_level } = line_state {
            let trimmed = line.trim_start_matches([' ', '\t']);
            if trimmed.is_empty() {
                formatted.push('\n');
            } else {
                for _ in 0..indent_level {
                    formatted.push('\t');
                }
                formatted.push_str(trimmed);
                formatted.push('\n');
            }
            indent_depth = apply_indent_delta(indent_depth, analysis.delta);
            line_state = next_line_state(line_state, indent_depth, analysis, indent_level);
            continue;
        }

        if analysis.preserve_line {
            formatted.push_str(line);
            formatted.push('\n');
            indent_depth = apply_indent_delta(indent_depth, analysis.delta);
            line_state = next_line_state(line_state, indent_depth, analysis, 0);
            continue;
        }

        let trimmed_start = line.trim_start_matches([' ', '\t']);
        let trimmed = if analysis.preserve_trailing_whitespace {
            trimmed_start
        } else {
            trimmed_start.trim_end_matches([' ', '\t'])
        };

        if trimmed.is_empty() {
            formatted.push('\n');
            indent_depth = apply_indent_delta(indent_depth, analysis.delta);
            line_state = next_line_state(line_state, indent_depth, analysis, 0);
            continue;
        }

        let leading_closers = count_leading_closing_delimiters(trimmed);
        let mut indent_level = indent_depth.saturating_sub(leading_closers);
        if is_case_or_default_clause(trimmed) {
            indent_level = indent_level.saturating_sub(1);
        }

        for _ in 0..indent_level {
            formatted.push('\t');
        }
        formatted.push_str(trimmed);
        formatted.push('\n');

        let current_indent_level = indent_level;
        indent_depth = apply_indent_delta(indent_depth, analysis.delta);
        line_state = next_line_state(line_state, indent_depth, analysis, current_indent_level);
    }

    if formatted.is_empty() || !formatted.ends_with('\n') {
        formatted.push('\n');
    }

    formatted
}

fn normalize_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn count_leading_closing_delimiters(line: &str) -> usize {
    line.chars()
        .take_while(|ch| matches!(ch, '}' | ')' | ']'))
        .count()
}

fn is_case_or_default_clause(line: &str) -> bool {
    starts_with_keyword(line, "case") || starts_with_keyword(line, "default")
}

fn starts_with_keyword(line: &str, keyword: &str) -> bool {
    let Some(rest) = line.strip_prefix(keyword) else {
        return false;
    };
    rest.chars()
        .next()
        .map(|ch| !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
        .unwrap_or(true)
}

fn apply_indent_delta(indent_depth: usize, delta: isize) -> usize {
    if delta >= 0 {
        indent_depth.saturating_add(delta as usize)
    } else {
        indent_depth.saturating_sub(delta.unsigned_abs())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineState {
    Code,
    RawString,
    BlockComment { indent_level: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LineAnalysis {
    delta: isize,
    ends_in_raw_string: bool,
    ends_in_block_comment: bool,
    preserve_line: bool,
    preserve_trailing_whitespace: bool,
}

fn next_line_state(
    start_state: LineState,
    indent_depth: usize,
    analysis: LineAnalysis,
    current_indent_level: usize,
) -> LineState {
    if analysis.ends_in_raw_string {
        return LineState::RawString;
    }
    if analysis.ends_in_block_comment {
        let indent_level = match start_state {
            LineState::BlockComment { indent_level } => indent_level,
            _ => current_indent_level.min(indent_depth),
        };
        return LineState::BlockComment { indent_level };
    }
    LineState::Code
}

fn analyze_line(line: &str, start_state: LineState) -> LineAnalysis {
    let mut delta = 0isize;
    let mut chars = line.chars().peekable();
    let mut in_raw_string = matches!(start_state, LineState::RawString);
    let mut in_block_comment = matches!(start_state, LineState::BlockComment { .. });
    let mut preserve_trailing_whitespace = !matches!(start_state, LineState::Code);

    while let Some(ch) = chars.next() {
        if in_raw_string {
            preserve_trailing_whitespace = true;
            if ch == '`' {
                in_raw_string = false;
            }
            continue;
        }

        if in_block_comment {
            preserve_trailing_whitespace = true;
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        {
            if ch == '/' {
                match chars.peek() {
                    Some('/') => break,
                    Some('*') => {
                        preserve_trailing_whitespace = true;
                        chars.next();
                        in_block_comment = true;
                        continue;
                    }
                    _ => {}
                }
            }

            match ch {
                '"' => {
                    let mut escaped = false;
                    for next in chars.by_ref() {
                        if escaped {
                            escaped = false;
                            continue;
                        }
                        match next {
                            '\\' => escaped = true,
                            '"' => break,
                            _ => {}
                        }
                    }
                }
                '\'' => {
                    let mut escaped = false;
                    for next in chars.by_ref() {
                        if escaped {
                            escaped = false;
                            continue;
                        }
                        match next {
                            '\\' => escaped = true,
                            '\'' => break,
                            _ => {}
                        }
                    }
                }
                '`' => {
                    preserve_trailing_whitespace = true;
                    in_raw_string = true;
                }
                '{' | '(' | '[' => delta += 1,
                '}' | ')' | ']' => delta -= 1,
                _ => {}
            }
        }
    }

    LineAnalysis {
        delta,
        ends_in_raw_string: in_raw_string,
        ends_in_block_comment: in_block_comment,
        preserve_line: matches!(start_state, LineState::RawString),
        preserve_trailing_whitespace,
    }
}
