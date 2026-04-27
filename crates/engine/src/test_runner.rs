use gowasm_host_types::{Diagnostic, ErrorCategory, TestResultDetails, WorkspaceFile};
use gowasm_lexer::{lex, TokenKind};
use gowasm_parser::{parse_source_file_with_spans, SourceFile, SourceFileSpans};

pub(super) struct PreparedPackageTest {
    pub(super) files: Vec<WorkspaceFile>,
    pub(super) entry_path: String,
    pub(super) details: TestResultDetails,
}

const GENERATED_PACKAGE_TEST_RUNNER_PATH: &str = "__gowasm_package_test_runner__.go";
const RENAMED_PACKAGE_MAIN_PREFIX: &str = "__gowasm_saved_package_main_";

pub(super) fn prepare_package_test(
    files: &[WorkspaceFile],
    target_path: &str,
    filter: Option<&str>,
) -> Result<PreparedPackageTest, Vec<Diagnostic>> {
    if !target_path.ends_with(".go") {
        return Err(vec![test_runner_diagnostic(
            target_path,
            "package tests currently require a Go source target path".into(),
        )]);
    }

    let Some(target_file) = files.iter().find(|file| file.path == target_path) else {
        return Err(vec![test_runner_diagnostic(
            target_path,
            format!("target file `{target_path}` was not found in the workspace payload"),
        )]);
    };

    let (target_parsed, _) =
        parse_source_file_with_spans(&target_file.contents).map_err(|error| {
            vec![test_runner_diagnostic(
                target_path,
                format!(
                "cannot test package rooted at `{target_path}` until it parses cleanly: {error}"
            ),
            )]
        })?;
    if target_parsed.package_name.ends_with("_test") {
        return Err(vec![test_runner_diagnostic(
            target_path,
            "external test packages ending in `_test` are not yet supported".into(),
        )]);
    }

    let target_dir = directory_of(target_path);
    let runner_path = join_path(target_dir, GENERATED_PACKAGE_TEST_RUNNER_PATH);
    if files.iter().any(|file| file.path == runner_path) {
        return Err(vec![test_runner_diagnostic(
            &runner_path,
            format!(
                "reserved generated runner path `{runner_path}` already exists in the workspace"
            ),
        )]);
    }

    let filter = filter.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let mut test_functions = Vec::new();
    let mut saw_unsupported_test = false;
    let mut saw_filter_name = false;
    let mut next_saved_main_index = 0usize;
    let mut files = files.to_vec();
    for file in &mut files {
        if !file.path.ends_with(".go") || directory_of(&file.path) != target_dir {
            continue;
        }
        let (parsed, spans) = parse_source_file_with_spans(&file.contents).map_err(|error| {
            vec![test_runner_diagnostic(
                &file.path,
                format!(
                    "cannot test package rooted at `{target_path}` until `{}` parses cleanly: {error}",
                    file.path
                ),
            )]
        })?;
        if parsed.package_name != target_parsed.package_name {
            continue;
        }
        if let Some(rewritten_contents) = rewrite_top_level_main_functions(
            &file.contents,
            &parsed,
            &spans,
            &mut next_saved_main_index,
        )
        .map_err(|error| vec![test_runner_diagnostic(&file.path, error)])?
        {
            file.contents = rewritten_contents;
        }
        for function in parsed.functions {
            if !function.name.starts_with("Test") {
                continue;
            }
            if let Some(filter_name) = filter {
                if function.name != filter_name {
                    continue;
                }
                saw_filter_name = true;
            }
            if function.receiver.is_none()
                && function.params.is_empty()
                && function.result_types.is_empty()
            {
                test_functions.push(function.name);
            } else {
                saw_unsupported_test = true;
            }
        }
    }

    if test_functions.is_empty() {
        let message = if let Some(filter_name) = filter {
            if saw_unsupported_test && saw_filter_name {
                format!(
                    "package test filter `{filter_name}` matched no supported same-package top-level `Test*` function; matching tests must have no parameters, no return values, and no receiver in the current runner"
                )
            } else {
                format!(
                    "package test filter `{filter_name}` matched no supported same-package top-level `Test*` function"
                )
            }
        } else if saw_unsupported_test {
            "no supported same-package top-level `Test*` functions were found; test functions must have no parameters, no return values, and no receiver in the current runner"
                .into()
        } else {
            "no same-package top-level `Test*` functions were found for the current package test run"
                .into()
        };
        return Err(vec![test_runner_diagnostic(target_path, message)]);
    }

    files.push(WorkspaceFile {
        path: runner_path.clone(),
        contents: build_package_test_runner_source(&target_parsed.package_name, &test_functions),
    });
    Ok(PreparedPackageTest {
        files,
        entry_path: runner_path,
        details: TestResultDetails {
            subject_path: target_path.into(),
            planned_tests: test_functions,
            completed_tests: Vec::new(),
            active_test: None,
        },
    })
}

fn rewrite_top_level_main_functions(
    source: &str,
    parsed: &SourceFile,
    spans: &SourceFileSpans,
    next_saved_main_index: &mut usize,
) -> Result<Option<String>, String> {
    if parsed.functions.len() != spans.functions.len() {
        return Err(
            "cannot prepare package tests because function source spans were incomplete".into(),
        );
    }

    let mut replacements = Vec::new();
    for (function, function_spans) in parsed.functions.iter().zip(&spans.functions) {
        if function.receiver.is_some() || function.name != "main" {
            continue;
        }
        let function_source = &source[function_spans.span.start..function_spans.span.end];
        let name_span = top_level_function_name_span(function_source).map_err(|error| {
            format!("cannot rewrite existing `main` for the package test runner: {error}")
        })?;
        replacements.push((
            function_spans.span.start + name_span.0,
            function_spans.span.start + name_span.1,
            format!("{RENAMED_PACKAGE_MAIN_PREFIX}{next_saved_main_index}"),
        ));
        *next_saved_main_index += 1;
    }

    if replacements.is_empty() {
        return Ok(None);
    }

    let mut rewritten = source.to_string();
    for (start, end, replacement) in replacements.into_iter().rev() {
        rewritten.replace_range(start..end, &replacement);
    }
    Ok(Some(rewritten))
}

fn top_level_function_name_span(function_source: &str) -> Result<(usize, usize), String> {
    let tokens = lex(function_source).map_err(|error| error.to_string())?;
    let mut saw_func = false;
    for token in tokens {
        match token.kind {
            TokenKind::Func => saw_func = true,
            TokenKind::Ident(name) if saw_func => {
                if name != "main" {
                    return Err(format!(
                        "expected top-level function name `main`, found `{name}`"
                    ));
                }
                return Ok((token.span.start, token.span.end));
            }
            _ => {}
        }
    }
    Err("did not find a function name token".into())
}

fn build_package_test_runner_source(package_name: &str, test_functions: &[String]) -> String {
    let mut source = String::new();
    source.push_str("package ");
    source.push_str(package_name);
    source.push_str("\n\nimport \"fmt\"\n\nfunc init() {\n");
    for test_name in test_functions {
        source.push_str("\tfmt.Println(\"RUN ");
        source.push_str(test_name);
        source.push_str("\")\n");
        source.push('\t');
        source.push_str(test_name);
        source.push_str("()\n");
        source.push_str("\tfmt.Println(\"PASS ");
        source.push_str(test_name);
        source.push_str("\")\n");
    }
    source.push_str("\tfmt.Println(\"PASS\")\n}\n\nfunc main() {}\n");
    source
}

fn test_runner_diagnostic(path: &str, message: String) -> Diagnostic {
    let mut diagnostic = Diagnostic::error_with_category(ErrorCategory::Tooling, message);
    diagnostic.file_path = Some(path.into());
    diagnostic.suggested_action = Some("Fix the package test setup and try again.".into());
    diagnostic
}

fn directory_of(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

fn join_path(dir: &str, file_name: &str) -> String {
    if dir.is_empty() {
        file_name.into()
    } else {
        format!("{dir}/{file_name}")
    }
}
