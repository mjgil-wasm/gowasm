use gowasm_host_types::{
    Diagnostic, ErrorCategory, Position, RuntimeDiagnostic, RuntimeSourceLocation,
    RuntimeSourceSpan, RuntimeStackFrame, SourceExcerpt, WorkspaceFile,
};
use gowasm_vm::VmError;

use crate::diagnostic_source::{source_excerpt_for_offsets, source_span_from_runtime_location};
use crate::snippet_runner::SnippetSourceMapper;

pub(super) fn vm_error_diagnostic(
    files: &[WorkspaceFile],
    entry_path: &str,
    error: &VmError,
    source_mapper: Option<&SnippetSourceMapper>,
) -> Diagnostic {
    let mut runtime = runtime_diagnostic(error);
    if let (Some(source_mapper), Some(runtime)) = (source_mapper, runtime.as_mut()) {
        source_mapper.remap_runtime_diagnostic(runtime);
    }

    let mut diagnostic = Diagnostic::error_with_category(error.category(), error.to_string());
    if let Some(frame) = runtime
        .as_ref()
        .and_then(|runtime| runtime.stack_trace.first())
    {
        diagnostic.file_path = diagnostic_frame_path(frame).or_else(|| Some(entry_path.into()));
        diagnostic.position = frame.source_location.as_ref().map(|location| Position {
            line: location.line,
            column: location.column,
        });
        if let Some(location) = frame.source_location.as_ref() {
            diagnostic.source_span = Some(source_span_from_runtime_location(location));
            if let Some(source) = workspace_source(files, &location.path) {
                diagnostic.source_excerpt = source_excerpt_for_runtime_location(source, location);
            }
        }
    } else {
        diagnostic.file_path = Some(entry_path.into());
    }
    diagnostic.runtime = runtime;
    diagnostic.suggested_action = runtime_suggested_action(error);
    diagnostic
}

fn runtime_diagnostic(error: &VmError) -> Option<RuntimeDiagnostic> {
    if error.stack_trace().is_empty() {
        return None;
    }
    Some(RuntimeDiagnostic {
        root_message: error.root_cause().to_string(),
        category: error.root_cause().category(),
        stack_trace: error
            .stack_trace()
            .iter()
            .map(map_runtime_stack_frame)
            .collect(),
    })
}

fn map_runtime_stack_frame(frame: &gowasm_vm::FrameDebugInfo) -> RuntimeStackFrame {
    RuntimeStackFrame {
        function: frame.function.clone(),
        instruction_index: usize_to_u32(frame.instruction_index),
        source_span: frame.source_span.as_ref().map(|span| RuntimeSourceSpan {
            path: span.path.clone(),
            start: usize_to_u32(span.start),
            end: usize_to_u32(span.end),
        }),
        source_location: frame
            .source_location
            .as_ref()
            .map(|location| RuntimeSourceLocation {
                path: location.path.clone(),
                line: usize_to_u32(location.line),
                column: usize_to_u32(location.column),
                end_line: usize_to_u32(location.end_line),
                end_column: usize_to_u32(location.end_column),
            }),
    }
}

fn diagnostic_frame_path(frame: &RuntimeStackFrame) -> Option<String> {
    frame
        .source_location
        .as_ref()
        .map(|location| location.path.clone())
        .or_else(|| frame.source_span.as_ref().map(|span| span.path.clone()))
}

fn workspace_source<'a>(files: &'a [WorkspaceFile], path: &str) -> Option<&'a str> {
    files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
}

fn source_excerpt_for_runtime_location(
    source: &str,
    location: &RuntimeSourceLocation,
) -> Option<SourceExcerpt> {
    let start_offset = byte_offset_for_line_column(source, location.line, location.column)?;
    let end_offset = byte_offset_for_line_column(source, location.end_line, location.end_column)?;
    source_excerpt_for_offsets(source, start_offset, end_offset)
}

fn byte_offset_for_line_column(source: &str, line: u32, column: u32) -> Option<usize> {
    let target_line = usize::try_from(line).ok()?;
    let target_column = usize::try_from(column).ok()?;
    if target_line == 0 || target_column == 0 {
        return None;
    }

    let line_start = if target_line == 1 {
        0
    } else {
        let mut current_line = 1usize;
        let mut start = None;
        for (index, ch) in source.char_indices() {
            if ch == '\n' {
                current_line += 1;
                if current_line == target_line {
                    start = Some(index + 1);
                    break;
                }
            }
        }
        start?
    };

    if target_column == 1 {
        return Some(line_start);
    }

    let mut chars_seen = 1usize;
    for (relative, _) in source[line_start..].char_indices() {
        if chars_seen == target_column {
            return Some(line_start + relative);
        }
        chars_seen += 1;
    }
    if chars_seen == target_column {
        return Some(source.len());
    }
    None
}

fn runtime_suggested_action(error: &VmError) -> Option<String> {
    match error.category() {
        ErrorCategory::RuntimeBudgetExhaustion => {
            Some("Increase the instruction budget or reduce work per run.".into())
        }
        ErrorCategory::RuntimeDeadlock => Some(
            "Unblock the program by adding a ready goroutine, channel partner, or default path."
                .into(),
        ),
        ErrorCategory::RuntimeCancellation => {
            Some("Retry the run after resuming or restarting the cancelled work.".into())
        }
        _ => None,
    }
}

fn usize_to_u32(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
