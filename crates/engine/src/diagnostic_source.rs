use gowasm_host_types::{Position, RuntimeSourceLocation, SourceExcerpt, SourceSpan};

pub(super) fn position_for_offset(source: &str, offset: usize) -> Option<Position> {
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

    let column = source[line_start..offset].chars().count() + 1;
    Some(Position {
        line: usize_to_u32(line),
        column: usize_to_u32(column),
    })
}

pub(super) fn source_span_for_offsets(
    source: &str,
    start: usize,
    end: usize,
) -> Option<SourceSpan> {
    let start_position = position_for_offset(source, start)?;
    let end_position = position_for_offset(source, end.max(start))?;
    Some(SourceSpan {
        start: start_position,
        end: end_position,
    })
}

pub(super) fn source_excerpt_for_offsets(
    source: &str,
    start: usize,
    end: usize,
) -> Option<SourceExcerpt> {
    let (line, column, line_text) = line_details(source, start)?;
    let highlight_width = underline_width(source, start, end).max(1);
    Some(SourceExcerpt {
        line: usize_to_u32(line),
        text: line_text.to_string(),
        highlight_start_column: usize_to_u32(column),
        highlight_end_column: usize_to_u32(column + highlight_width - 1),
    })
}

pub(super) fn source_span_from_runtime_location(location: &RuntimeSourceLocation) -> SourceSpan {
    SourceSpan {
        start: Position {
            line: location.line,
            column: location.column,
        },
        end: Position {
            line: location.end_line,
            column: location.end_column,
        },
    }
}

pub(super) fn render_source_excerpt(excerpt: &SourceExcerpt) -> String {
    let gutter_width = excerpt.line.to_string().len();
    let marker_padding = " ".repeat(
        excerpt
            .highlight_start_column
            .saturating_sub(1)
            .try_into()
            .unwrap_or(usize::MAX),
    );
    let marker_width = excerpt
        .highlight_end_column
        .saturating_sub(excerpt.highlight_start_column)
        .saturating_add(1)
        .try_into()
        .unwrap_or(usize::MAX);
    let marker = "^".repeat(marker_width.max(1));

    format!(
        "{blank:>width$} |\n{line_no:>width$} | {line_text}\n{blank:>width$} | {marker_padding}{marker}",
        blank = "",
        width = gutter_width,
        line_no = excerpt.line,
        line_text = excerpt.text,
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

fn underline_width(source: &str, start: usize, end: usize) -> usize {
    let line_end = source[start..]
        .find('\n')
        .map(|relative| start + relative)
        .unwrap_or(source.len());
    let highlight_end = end.min(line_end).max(start);
    source[start..highlight_end].chars().count().max(1)
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
