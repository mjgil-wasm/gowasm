use gowasm_host_types::{
    Diagnostic, ErrorCategory, RuntimeDiagnostic, RuntimeSourceLocation, TestResultDetails,
    WorkspaceFile,
};

pub(super) struct PreparedSnippetTest {
    pub(super) compile_files: Vec<WorkspaceFile>,
    pub(super) entry_path: String,
    pub(super) details: TestResultDetails,
    pub(super) source_mapper: Option<SnippetSourceMapper>,
}

#[derive(Clone)]
pub(super) struct SnippetSourceMapper {
    entry_path: String,
    original_source: String,
    wrapped_line_map: Vec<Option<u32>>,
}

pub(super) fn prepare_snippet_test(
    files: &[WorkspaceFile],
    entry_path: &str,
) -> Result<PreparedSnippetTest, Vec<Diagnostic>> {
    if !entry_path.ends_with(".go") {
        return Err(vec![snippet_runner_diagnostic(
            entry_path,
            "snippet tests currently require a Go source entry path".into(),
        )]);
    }

    let Some(entry_file) = files.iter().find(|file| file.path == entry_path) else {
        return Err(vec![snippet_runner_diagnostic(
            entry_path,
            format!("entry file `{entry_path}` was not found in the workspace payload"),
        )]);
    };

    if source_starts_with_package_clause(&entry_file.contents) {
        return Ok(PreparedSnippetTest {
            compile_files: files.to_vec(),
            entry_path: entry_path.into(),
            details: default_snippet_details(entry_path),
            source_mapper: None,
        });
    }

    let wrapped = wrap_snippet_source(&entry_file.contents);
    let mut compile_files = files.to_vec();
    if let Some(file) = compile_files
        .iter_mut()
        .find(|file| file.path == entry_path)
    {
        file.contents = wrapped.source;
    }

    Ok(PreparedSnippetTest {
        compile_files,
        entry_path: entry_path.into(),
        details: default_snippet_details(entry_path),
        source_mapper: Some(SnippetSourceMapper::new(
            entry_path,
            entry_file.contents.clone(),
            wrapped.line_map,
        )),
    })
}

impl SnippetSourceMapper {
    fn new(entry_path: &str, original_source: String, wrapped_line_map: Vec<Option<u32>>) -> Self {
        Self {
            entry_path: entry_path.into(),
            original_source,
            wrapped_line_map,
        }
    }

    pub(super) fn applies_to(&self, path: &str) -> bool {
        self.entry_path == path
    }

    pub(super) fn original_source(&self) -> &str {
        &self.original_source
    }

    pub(super) fn original_line_for_wrapped_line(&self, wrapped_line: u32) -> Option<u32> {
        let index = usize::try_from(wrapped_line.saturating_sub(1)).ok()?;
        self.wrapped_line_map.get(index).copied().flatten()
    }

    pub(super) fn remap_runtime_diagnostic(&self, runtime: &mut RuntimeDiagnostic) {
        for frame in &mut runtime.stack_trace {
            if let Some(location) = frame.source_location.as_mut() {
                self.remap_runtime_source_location(location);
            }
        }
    }

    fn remap_runtime_source_location(&self, location: &mut RuntimeSourceLocation) {
        if !self.applies_to(&location.path) {
            return;
        }
        let Some(line) = self.original_line_for_wrapped_line(location.line) else {
            return;
        };
        let line_delta = location.end_line.saturating_sub(location.line);
        location.line = line;
        location.end_line = line.saturating_add(line_delta);
    }
}

struct WrappedSnippetSource {
    source: String,
    line_map: Vec<Option<u32>>,
}

fn default_snippet_details(entry_path: &str) -> TestResultDetails {
    TestResultDetails {
        subject_path: entry_path.into(),
        planned_tests: vec![entry_path.into()],
        completed_tests: Vec::new(),
        active_test: None,
    }
}

fn wrap_snippet_source(source: &str) -> WrappedSnippetSource {
    let original_lines = logical_lines(source);
    let import_line_indexes = collect_leading_import_lines(&original_lines);

    let mut wrapped_lines = Vec::new();
    let mut line_map = Vec::new();

    push_wrapped_line(&mut wrapped_lines, &mut line_map, "package main", None);
    push_wrapped_line(&mut wrapped_lines, &mut line_map, "", None);

    if !import_line_indexes.is_empty() {
        for (index, line) in original_lines.iter().enumerate() {
            if import_line_indexes.contains(&index) {
                push_wrapped_line(
                    &mut wrapped_lines,
                    &mut line_map,
                    line,
                    Some(usize_to_u32(index + 1)),
                );
            }
        }
        push_wrapped_line(&mut wrapped_lines, &mut line_map, "", None);
    }

    push_wrapped_line(&mut wrapped_lines, &mut line_map, "func main() {", None);
    for (index, line) in original_lines.iter().enumerate() {
        if !import_line_indexes.contains(&index) {
            push_wrapped_line(
                &mut wrapped_lines,
                &mut line_map,
                line,
                Some(usize_to_u32(index + 1)),
            );
        }
    }
    push_wrapped_line(&mut wrapped_lines, &mut line_map, "}", None);

    let mut wrapped_source = wrapped_lines.join("\n");
    wrapped_source.push('\n');
    WrappedSnippetSource {
        source: wrapped_source,
        line_map,
    }
}

fn push_wrapped_line(
    wrapped_lines: &mut Vec<String>,
    line_map: &mut Vec<Option<u32>>,
    line: &str,
    original_line: Option<u32>,
) {
    wrapped_lines.push(line.to_string());
    line_map.push(original_line);
}

fn logical_lines(source: &str) -> Vec<&str> {
    let mut lines = source.split('\n').collect::<Vec<_>>();
    if source.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn collect_leading_import_lines(lines: &[&str]) -> Vec<usize> {
    let mut import_lines = Vec::new();
    let mut cursor = 0usize;
    let mut in_block_comment = false;

    skip_leading_noncode_lines(lines, &mut cursor, &mut in_block_comment);

    loop {
        if cursor >= lines.len() {
            break;
        }
        let trimmed = lines[cursor].trim_start();
        if !starts_import_decl(trimmed) {
            break;
        }

        import_lines.push(cursor);
        if starts_import_block(trimmed) {
            cursor += 1;
            while cursor < lines.len() {
                import_lines.push(cursor);
                let trimmed_line = lines[cursor].trim();
                cursor += 1;
                if trimmed_line == ")" {
                    break;
                }
            }
        } else {
            cursor += 1;
        }

        skip_blank_lines(lines, &mut cursor);
    }

    import_lines
}

fn skip_leading_noncode_lines(lines: &[&str], cursor: &mut usize, in_block_comment: &mut bool) {
    while *cursor < lines.len() {
        let trimmed = lines[*cursor].trim_start();
        if *in_block_comment {
            *in_block_comment = !trimmed.contains("*/");
            *cursor += 1;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            *cursor += 1;
            continue;
        }
        if trimmed.starts_with("/*") {
            *in_block_comment = !trimmed.contains("*/");
            *cursor += 1;
            continue;
        }
        break;
    }
}

fn skip_blank_lines(lines: &[&str], cursor: &mut usize) {
    while *cursor < lines.len() && lines[*cursor].trim().is_empty() {
        *cursor += 1;
    }
}

fn starts_import_decl(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("import") else {
        return false;
    };
    let rest = rest.trim_start();
    rest.starts_with('(')
        || rest.starts_with('"')
        || rest.starts_with('`')
        || rest.starts_with('.')
        || rest.starts_with('_')
        || rest
            .chars()
            .next()
            .is_some_and(|character| character.is_alphabetic())
}

fn starts_import_block(trimmed: &str) -> bool {
    trimmed
        .strip_prefix("import")
        .map(str::trim_start)
        .is_some_and(|rest| rest.starts_with('('))
}

fn source_starts_with_package_clause(source: &str) -> bool {
    let mut in_block_comment = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if in_block_comment {
            in_block_comment = !trimmed.contains("*/");
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("/*") {
            in_block_comment = !trimmed.contains("*/");
            continue;
        }
        return trimmed.starts_with("package ");
    }
    false
}

fn snippet_runner_diagnostic(path: &str, message: String) -> Diagnostic {
    let mut diagnostic = Diagnostic::error_with_category(ErrorCategory::Tooling, message);
    diagnostic.file_path = Some(path.into());
    diagnostic.suggested_action = Some("Fix the snippet or test entry setup and try again.".into());
    diagnostic
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
