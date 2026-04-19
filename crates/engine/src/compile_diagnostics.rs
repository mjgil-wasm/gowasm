use gowasm_compiler::{take_last_compile_error_context, CompileError};
use gowasm_host_types::{
    Diagnostic, ErrorCategory, Position, SourceExcerpt, SourceSpan, WorkspaceFile,
};

use super::diagnostic_source::{
    position_for_offset, render_source_excerpt, source_excerpt_for_offsets, source_span_for_offsets,
};
use super::snippet_runner::SnippetSourceMapper;

pub(super) fn compile_error_diagnostic(
    files: &[WorkspaceFile],
    entry_path: &str,
    error: &CompileError,
    source_mapper: Option<&SnippetSourceMapper>,
) -> Diagnostic {
    let mut diagnostic =
        Diagnostic::error_with_category(ErrorCategory::CompileError, error.to_string());
    diagnostic.suggested_action = Some("Fix the source error and compile again.".into());
    let Some(context) = take_last_compile_error_context() else {
        diagnostic.file_path = Some(entry_path.into());
        return diagnostic;
    };

    diagnostic.file_path = Some(context.file_path.clone());
    let Some(source) = workspace_source(files, &context.file_path) else {
        return diagnostic;
    };
    if let Some(mapped) = map_wrapped_snippet_diagnostic(
        source_mapper,
        &context.file_path,
        source,
        context.span_start,
        context.span_end,
        &diagnostic.message,
    ) {
        diagnostic.position = Some(mapped.position.clone());
        diagnostic.source_span = Some(mapped.source_span);
        diagnostic.source_excerpt = Some(mapped.source_excerpt);
        diagnostic.message = mapped.message;
        return diagnostic;
    }
    let Some(position) = position_for_offset(source, context.span_start) else {
        return diagnostic;
    };
    let source_span = source_span_for_offsets(source, context.span_start, context.span_end);
    let source_excerpt = source_excerpt_for_offsets(source, context.span_start, context.span_end);
    diagnostic.position = Some(position.clone());
    diagnostic.source_span = source_span;
    diagnostic.source_excerpt = source_excerpt.clone();
    diagnostic.message = render_compile_message(
        &diagnostic.message,
        &context.file_path,
        source_excerpt.as_ref(),
        &position,
    );
    diagnostic
}

struct MappedSnippetDiagnostic {
    position: Position,
    source_span: SourceSpan,
    source_excerpt: SourceExcerpt,
    message: String,
}

fn workspace_source<'a>(files: &'a [WorkspaceFile], path: &str) -> Option<&'a str> {
    files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
}

fn map_wrapped_snippet_diagnostic(
    source_mapper: Option<&SnippetSourceMapper>,
    path: &str,
    wrapped_source: &str,
    span_start: usize,
    span_end: usize,
    base: &str,
) -> Option<MappedSnippetDiagnostic> {
    let source_mapper = source_mapper?;
    if !source_mapper.applies_to(path) {
        return None;
    }

    let (wrapped_line, column, _) = line_details(wrapped_source, span_start)?;
    let original_line = source_mapper.original_line_for_wrapped_line(usize_to_u32(wrapped_line))?;
    let original_line_text = line_text_for_number(source_mapper.original_source(), original_line)?;
    let position = Position {
        line: original_line,
        column: usize_to_u32(column),
    };
    let underline_width = underline_width(wrapped_source, span_start, span_end).max(1);
    let source_span = SourceSpan {
        start: position.clone(),
        end: Position {
            line: position.line,
            column: position
                .column
                .saturating_add(usize_to_u32(underline_width.saturating_sub(1))),
        },
    };
    let source_excerpt = SourceExcerpt {
        line: original_line,
        text: original_line_text.to_string(),
        highlight_start_column: position.column,
        highlight_end_column: source_span.end.column,
    };

    Some(MappedSnippetDiagnostic {
        position: position.clone(),
        source_span,
        source_excerpt: source_excerpt.clone(),
        message: format!(
            "{base}\n--> {path}:{}:{}\n{}",
            position.line,
            position.column,
            render_source_excerpt(&source_excerpt),
        ),
    })
}

fn render_compile_message(
    base: &str,
    path: &str,
    source_excerpt: Option<&SourceExcerpt>,
    position: &Position,
) -> String {
    let Some(source_excerpt) = source_excerpt else {
        return base.to_string();
    };

    format!(
        "{base}\n--> {path}:{}:{}\n{}",
        position.line,
        position.column,
        render_source_excerpt(source_excerpt),
    )
}

fn line_details(source: &str, offset: usize) -> Option<(usize, usize, &str)> {
    if offset > source.len() {
        return None;
    }

    let mut line = 1usize;
    let mut line_start = 0usize;
    for (index, ch) in source.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = index + 1;
        }
    }

    let line_end = source[line_start..]
        .find('\n')
        .map(|relative| line_start + relative)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    let column = source[line_start..offset].chars().count() + 1;
    Some((line, column, line_text))
}

fn underline_width(source: &str, span_start: usize, span_end: usize) -> usize {
    let line_end = source[span_start..]
        .find('\n')
        .map(|relative| span_start + relative)
        .unwrap_or(source.len());
    let highlight_end = span_end.min(line_end).max(span_start);
    source[span_start..highlight_end].chars().count().max(1)
}

fn line_text_for_number(source: &str, line_number: u32) -> Option<&str> {
    let line_number = usize::try_from(line_number).ok()?;
    if line_number == 0 {
        return None;
    }
    source.lines().nth(line_number - 1)
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
