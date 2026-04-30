use gowasm_parser::{parse_source_file_with_spans, SourceFile, SourceFileSpans};

use crate::{compile_workspace, CompileError, Program};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInput<'a> {
    pub path: &'a str,
    pub source: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedFile {
    pub(crate) path: String,
    pub(crate) file: SourceFile,
    pub(crate) spans: SourceFileSpans,
}

pub fn compile_source(source: &str) -> Result<Program, CompileError> {
    compile_workspace(
        &[SourceInput {
            path: "main.go",
            source,
        }],
        "main.go",
    )
}

pub(crate) fn is_go_source_path(path: &str) -> bool {
    path.ends_with(".go")
}

pub(crate) fn parse_workspace_files(
    sources: &[SourceInput<'_>],
) -> Result<Vec<ParsedFile>, CompileError> {
    let mut parsed = Vec::with_capacity(sources.len());
    for source in sources {
        if !is_go_source_path(source.path) {
            continue;
        }
        let (file, spans) = parse_source_file_with_spans(source.source)?;
        parsed.push(ParsedFile {
            path: source.path.to_string(),
            file,
            spans,
        });
    }
    Ok(parsed)
}
