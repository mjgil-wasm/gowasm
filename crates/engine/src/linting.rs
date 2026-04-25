use std::collections::{BTreeMap, BTreeSet};

use gowasm_host_types::{Diagnostic, ErrorCategory, Severity, WorkspaceFile};
use gowasm_parser::{
    parse_source_file_with_spans, AssignTarget, Expr, FunctionDecl, SourceFile, SourceFileSpans,
    Stmt, SwitchCase, TypeDeclKind, TypeSwitchCase,
};

use crate::diagnostic_source::{
    position_for_offset, source_excerpt_for_offsets, source_span_for_offsets,
};
use crate::formatting::{format_go_source, go_parse_diagnostic};

const FORMAT_DRIFT_RULE: &str = "format-drift";
const DUPLICATE_IMPORT_RULE: &str = "duplicate-import";
const UNUSED_IMPORT_RULE: &str = "unused-import";

pub(super) fn lint_workspace_files(files: &[WorkspaceFile]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for file in files {
        if !file.path.ends_with(".go") {
            continue;
        }
        let suppressions = collect_lint_suppressions(&file.contents);
        let (parsed, spans) = match parse_source_file_with_spans(&file.contents) {
            Ok(parsed) => parsed,
            Err(error) => {
                diagnostics.push(go_parse_diagnostic(&file.path, "lint", error.to_string()));
                continue;
            }
        };
        diagnostics.extend(import_lint_warnings(
            &file.path,
            &file.contents,
            &parsed,
            &spans,
            &suppressions,
        ));

        let formatted = format_go_source(&file.contents);
        if file.contents != formatted && !suppressions.contains(FORMAT_DRIFT_RULE) {
            let difference_offset = first_difference_offset(&file.contents, &formatted);
            diagnostics.push(formatting_lint_warning(
                &file.path,
                &file.contents,
                difference_offset,
            ));
        }
    }

    diagnostics
}

fn collect_lint_suppressions(source: &str) -> BTreeSet<String> {
    let mut suppressions = BTreeSet::new();

    for line in source.lines() {
        let trimmed = line.trim();
        let Some(rule_list) = trimmed
            .strip_prefix("//gowasm:ignore")
            .or_else(|| trimmed.strip_prefix("// gowasm:ignore"))
        else {
            continue;
        };

        for rule in rule_list
            .split(',')
            .flat_map(|part| part.split_whitespace())
            .filter(|rule| !rule.is_empty())
        {
            suppressions.insert(rule.to_string());
        }
    }

    suppressions
}

fn import_lint_warnings(
    path: &str,
    source: &str,
    file: &SourceFile,
    spans: &SourceFileSpans,
    suppressions: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut import_indices = BTreeMap::<&str, Vec<usize>>::new();
    let import_selectors = file
        .imports
        .iter()
        .map(|decl| decl.selector())
        .collect::<BTreeSet<_>>();
    let mut used_selectors = BTreeSet::new();

    collect_used_import_selectors(file, &import_selectors, &mut used_selectors);

    for (index, import) in file.imports.iter().enumerate() {
        import_indices.entry(&import.path).or_default().push(index);
    }

    if !suppressions.contains(DUPLICATE_IMPORT_RULE) {
        for (import_path, indices) in &import_indices {
            for duplicate_index in indices.iter().skip(1) {
                diagnostics.push(duplicate_import_warning(
                    path,
                    source,
                    spans,
                    *duplicate_index,
                    import_path,
                ));
            }
        }
    }

    if suppressions.contains(UNUSED_IMPORT_RULE) {
        return diagnostics;
    }

    for (import_path, indices) in &import_indices {
        let selector = package_selector_name(import_path);
        if !used_selectors.contains(selector) {
            diagnostics.push(unused_import_warning(
                path,
                source,
                spans,
                indices[0],
                import_path,
                selector,
            ));
        }
    }

    diagnostics
}

fn formatting_lint_warning(
    path: &str,
    source: &str,
    difference_offset: Option<usize>,
) -> Diagnostic {
    let position = difference_offset.and_then(|offset| position_for_offset(source, offset));
    Diagnostic {
        message: format!(
            "`{path}` does not match the current gowasm formatter output; run Format to rewrite it"
        ),
        severity: Severity::Warning,
        category: ErrorCategory::Tooling,
        file_path: Some(path.into()),
        position,
        source_span: difference_offset
            .and_then(|offset| source_span_for_offsets(source, offset, offset)),
        source_excerpt: difference_offset
            .and_then(|offset| source_excerpt_for_offsets(source, offset, offset)),
        suggested_action: Some("Run Format to rewrite the file to the frozen gowasm style.".into()),
        runtime: None,
    }
}

fn duplicate_import_warning(
    path: &str,
    source: &str,
    spans: &SourceFileSpans,
    import_index: usize,
    import_path: &str,
) -> Diagnostic {
    let import_span = spans.imports.get(import_index).copied();
    Diagnostic {
        message: format!(
            "`{path}` imports `{import_path}` more than once; keep a single import entry"
        ),
        severity: Severity::Warning,
        category: ErrorCategory::Tooling,
        file_path: Some(path.into()),
        position: import_span.and_then(|span| position_for_offset(source, span.start)),
        source_span: import_span
            .and_then(|span| source_span_for_offsets(source, span.start, span.end)),
        source_excerpt: import_span
            .and_then(|span| source_excerpt_for_offsets(source, span.start, span.end)),
        suggested_action: Some(
            "Remove the duplicate import entry and keep a single import path.".into(),
        ),
        runtime: None,
    }
}

fn unused_import_warning(
    path: &str,
    source: &str,
    spans: &SourceFileSpans,
    import_index: usize,
    import_path: &str,
    selector: &str,
) -> Diagnostic {
    let import_span = spans.imports.get(import_index).copied();
    Diagnostic {
        message: format!(
            "`{path}` imports `{import_path}` but never references `{selector}`; remove the import or use it before running"
        ),
        severity: Severity::Warning,
        category: ErrorCategory::Tooling,
        file_path: Some(path.into()),
        position: import_span.and_then(|span| position_for_offset(source, span.start)),
        source_span: import_span.and_then(|span| source_span_for_offsets(source, span.start, span.end)),
        source_excerpt: import_span
            .and_then(|span| source_excerpt_for_offsets(source, span.start, span.end)),
        suggested_action: Some("Remove the unused import or reference its package selector.".into()),
        runtime: None,
    }
}

fn package_selector_name(import_path: &str) -> &str {
    import_path.rsplit('/').next().unwrap_or(import_path)
}

fn first_difference_offset(source: &str, formatted: &str) -> Option<usize> {
    let source_iter = source.char_indices().peekable();
    let mut formatted_iter = formatted.chars();

    for (offset, source_char) in source_iter {
        let Some(formatted_char) = formatted_iter.next() else {
            return Some(offset);
        };
        if source_char != formatted_char {
            return Some(offset);
        }
    }

    if formatted_iter.next().is_some() {
        return Some(source.len());
    }

    None
}

fn collect_used_import_selectors(
    file: &SourceFile,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    for type_decl in &file.types {
        for type_param in &type_decl.type_params {
            collect_import_selectors_from_type(
                &type_param.constraint,
                import_selectors,
                used_selectors,
            );
        }

        match &type_decl.kind {
            TypeDeclKind::Struct { fields } => {
                for field in fields {
                    collect_import_selectors_from_type(
                        &field.typ,
                        import_selectors,
                        used_selectors,
                    );
                }
            }
            TypeDeclKind::Interface { methods, embeds } => {
                for method in methods {
                    for parameter in &method.params {
                        collect_import_selectors_from_type(
                            &parameter.typ,
                            import_selectors,
                            used_selectors,
                        );
                    }
                    for result_type in &method.result_types {
                        collect_import_selectors_from_type(
                            result_type,
                            import_selectors,
                            used_selectors,
                        );
                    }
                }
                for embed in embeds {
                    collect_import_selectors_from_type(embed, import_selectors, used_selectors);
                }
            }
            TypeDeclKind::Alias { underlying } => {
                collect_import_selectors_from_type(underlying, import_selectors, used_selectors);
            }
        }
    }

    for const_decl in &file.consts {
        if let Some(typ) = &const_decl.typ {
            collect_import_selectors_from_type(typ, import_selectors, used_selectors);
        }
        collect_import_selectors_from_expr(&const_decl.value, import_selectors, used_selectors);
    }

    for var_decl in &file.vars {
        if let Some(typ) = &var_decl.typ {
            collect_import_selectors_from_type(typ, import_selectors, used_selectors);
        }
        if let Some(value) = &var_decl.value {
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
    }

    for function in &file.functions {
        collect_import_selectors_from_function(function, import_selectors, used_selectors);
    }
}

fn collect_import_selectors_from_function(
    function: &FunctionDecl,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    if let Some(receiver) = &function.receiver {
        collect_import_selectors_from_type(&receiver.typ, import_selectors, used_selectors);
    }

    for type_param in &function.type_params {
        collect_import_selectors_from_type(
            &type_param.constraint,
            import_selectors,
            used_selectors,
        );
    }

    for parameter in &function.params {
        collect_import_selectors_from_type(&parameter.typ, import_selectors, used_selectors);
    }

    for result_type in &function.result_types {
        collect_import_selectors_from_type(result_type, import_selectors, used_selectors);
    }

    collect_import_selectors_from_stmt_slice(&function.body, import_selectors, used_selectors);
}

fn collect_import_selectors_from_stmt_slice(
    stmts: &[Stmt],
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    for stmt in stmts {
        collect_import_selectors_from_stmt(stmt, import_selectors, used_selectors);
    }
}

fn collect_import_selectors_from_stmt(
    stmt: &Stmt,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    match stmt {
        Stmt::Expr(expr)
        | Stmt::ShortVarDecl { value: expr, .. }
        | Stmt::ShortVarDeclPair { value: expr, .. }
        | Stmt::ShortVarDeclTriple { value: expr, .. }
        | Stmt::ShortVarDeclQuad { value: expr, .. }
        | Stmt::Go { call: expr }
        | Stmt::Defer { call: expr } => {
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
        }
        Stmt::ShortVarDeclList { values, .. } => {
            for value in values {
                collect_import_selectors_from_expr(value, import_selectors, used_selectors);
            }
        }
        Stmt::VarDecl { typ, value, .. } => {
            if let Some(typ) = typ {
                collect_import_selectors_from_type(typ, import_selectors, used_selectors);
            }
            if let Some(value) = value {
                collect_import_selectors_from_expr(value, import_selectors, used_selectors);
            }
        }
        Stmt::ConstDecl { typ, value, .. } => {
            if let Some(typ) = typ {
                collect_import_selectors_from_type(typ, import_selectors, used_selectors);
            }
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
        Stmt::ConstGroup { decls } => {
            for decl in decls {
                if let Some(typ) = &decl.typ {
                    collect_import_selectors_from_type(typ, import_selectors, used_selectors);
                }
                collect_import_selectors_from_expr(&decl.value, import_selectors, used_selectors);
            }
        }
        Stmt::Assign { target, value } => {
            collect_import_selectors_from_target(target, import_selectors, used_selectors);
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
        Stmt::AssignList { targets, values } => {
            for target in targets {
                collect_import_selectors_from_target(target, import_selectors, used_selectors);
            }
            for value in values {
                collect_import_selectors_from_expr(value, import_selectors, used_selectors);
            }
        }
        Stmt::AssignPair {
            first,
            second,
            value,
        } => {
            collect_import_selectors_from_target(first, import_selectors, used_selectors);
            collect_import_selectors_from_target(second, import_selectors, used_selectors);
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
        Stmt::AssignTriple {
            first,
            second,
            third,
            value,
        } => {
            collect_import_selectors_from_target(first, import_selectors, used_selectors);
            collect_import_selectors_from_target(second, import_selectors, used_selectors);
            collect_import_selectors_from_target(third, import_selectors, used_selectors);
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
        Stmt::AssignQuad {
            first,
            second,
            third,
            fourth,
            value,
        } => {
            collect_import_selectors_from_target(first, import_selectors, used_selectors);
            collect_import_selectors_from_target(second, import_selectors, used_selectors);
            collect_import_selectors_from_target(third, import_selectors, used_selectors);
            collect_import_selectors_from_target(fourth, import_selectors, used_selectors);
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
        Stmt::Increment { .. }
        | Stmt::Decrement { .. }
        | Stmt::Break { .. }
        | Stmt::Continue { .. } => {}
        Stmt::If {
            init,
            condition,
            then_body,
            else_body,
        } => {
            if let Some(init) = init {
                collect_import_selectors_from_stmt(init, import_selectors, used_selectors);
            }
            collect_import_selectors_from_expr(condition, import_selectors, used_selectors);
            collect_import_selectors_from_stmt_slice(then_body, import_selectors, used_selectors);
            if let Some(else_body) = else_body {
                collect_import_selectors_from_stmt_slice(
                    else_body,
                    import_selectors,
                    used_selectors,
                );
            }
        }
        Stmt::For {
            init,
            condition,
            post,
            body,
        } => {
            if let Some(init) = init {
                collect_import_selectors_from_stmt(init, import_selectors, used_selectors);
            }
            if let Some(condition) = condition {
                collect_import_selectors_from_expr(condition, import_selectors, used_selectors);
            }
            if let Some(post) = post {
                collect_import_selectors_from_stmt(post, import_selectors, used_selectors);
            }
            collect_import_selectors_from_stmt_slice(body, import_selectors, used_selectors);
        }
        Stmt::RangeFor { expr, body, .. } => {
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
            collect_import_selectors_from_stmt_slice(body, import_selectors, used_selectors);
        }
        Stmt::Switch {
            init,
            expr,
            cases,
            default,
            ..
        } => {
            if let Some(init) = init {
                collect_import_selectors_from_stmt(init, import_selectors, used_selectors);
            }
            if let Some(expr) = expr {
                collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
            }
            for case in cases {
                collect_import_selectors_from_switch_case(case, import_selectors, used_selectors);
            }
            if let Some(default) = default {
                collect_import_selectors_from_stmt_slice(default, import_selectors, used_selectors);
            }
        }
        Stmt::TypeSwitch {
            expr,
            cases,
            default,
            ..
        } => {
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
            for case in cases {
                collect_import_selectors_from_type_switch_case(
                    case,
                    import_selectors,
                    used_selectors,
                );
            }
            if let Some(default) = default {
                collect_import_selectors_from_stmt_slice(default, import_selectors, used_selectors);
            }
        }
        Stmt::Select { cases, default } => {
            for case in cases {
                collect_import_selectors_from_stmt(&case.stmt, import_selectors, used_selectors);
                collect_import_selectors_from_stmt_slice(
                    &case.body,
                    import_selectors,
                    used_selectors,
                );
            }
            if let Some(default) = default {
                collect_import_selectors_from_stmt_slice(default, import_selectors, used_selectors);
            }
        }
        Stmt::Send { chan, value } => {
            collect_import_selectors_from_expr(chan, import_selectors, used_selectors);
            collect_import_selectors_from_expr(value, import_selectors, used_selectors);
        }
        Stmt::Labeled { stmt, .. } => {
            collect_import_selectors_from_stmt(stmt, import_selectors, used_selectors);
        }
        Stmt::Return(values) => {
            for value in values {
                collect_import_selectors_from_expr(value, import_selectors, used_selectors);
            }
        }
    }
}

fn collect_import_selectors_from_switch_case(
    case: &SwitchCase,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    for expr in &case.expressions {
        collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
    }
    collect_import_selectors_from_stmt_slice(&case.body, import_selectors, used_selectors);
}

fn collect_import_selectors_from_type_switch_case(
    case: &TypeSwitchCase,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    for typ in &case.types {
        collect_import_selectors_from_type(typ, import_selectors, used_selectors);
    }
    collect_import_selectors_from_stmt_slice(&case.body, import_selectors, used_selectors);
}

fn collect_import_selectors_from_target(
    target: &AssignTarget,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    match target {
        AssignTarget::Ident(_) | AssignTarget::Deref { .. } => {}
        AssignTarget::DerefSelector { .. } | AssignTarget::Selector { .. } => {}
        AssignTarget::DerefIndex { index, .. } | AssignTarget::Index { index, .. } => {
            collect_import_selectors_from_expr(index, import_selectors, used_selectors);
        }
    }
}

fn collect_import_selectors_from_expr(
    expr: &Expr,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Ident(_)
        | Expr::NilLiteral
        | Expr::BoolLiteral(_)
        | Expr::IntLiteral(_)
        | Expr::FloatLiteral(_)
        | Expr::StringLiteral(_) => {}
        Expr::Unary { expr, .. } => {
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
        }
        Expr::Binary { left, right, .. } => {
            collect_import_selectors_from_expr(left, import_selectors, used_selectors);
            collect_import_selectors_from_expr(right, import_selectors, used_selectors);
        }
        Expr::ArrayLiteral {
            element_type,
            elements,
            ..
        }
        | Expr::SliceLiteral {
            element_type,
            elements,
        } => {
            collect_import_selectors_from_type(element_type, import_selectors, used_selectors);
            for element in elements {
                collect_import_selectors_from_expr(element, import_selectors, used_selectors);
            }
        }
        Expr::SliceConversion { element_type, expr } => {
            collect_import_selectors_from_type(element_type, import_selectors, used_selectors);
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
        }
        Expr::MapLiteral {
            key_type,
            value_type,
            entries,
        } => {
            collect_import_selectors_from_type(key_type, import_selectors, used_selectors);
            collect_import_selectors_from_type(value_type, import_selectors, used_selectors);
            for entry in entries {
                collect_import_selectors_from_expr(&entry.key, import_selectors, used_selectors);
                collect_import_selectors_from_expr(&entry.value, import_selectors, used_selectors);
            }
        }
        Expr::StructLiteral { type_name, fields } => {
            collect_import_selectors_from_type(type_name, import_selectors, used_selectors);
            for field in fields {
                collect_import_selectors_from_expr(&field.value, import_selectors, used_selectors);
            }
        }
        Expr::Index { target, index } => {
            collect_import_selectors_from_expr(target, import_selectors, used_selectors);
            collect_import_selectors_from_expr(index, import_selectors, used_selectors);
        }
        Expr::SliceExpr { target, low, high } => {
            collect_import_selectors_from_expr(target, import_selectors, used_selectors);
            if let Some(low) = low {
                collect_import_selectors_from_expr(low, import_selectors, used_selectors);
            }
            if let Some(high) = high {
                collect_import_selectors_from_expr(high, import_selectors, used_selectors);
            }
        }
        Expr::Selector { receiver, .. } => {
            if let Expr::Ident(receiver_name) = receiver.as_ref() {
                if import_selectors.contains(receiver_name.as_str()) {
                    used_selectors.insert(receiver_name.clone());
                }
            }
            collect_import_selectors_from_expr(receiver, import_selectors, used_selectors);
        }
        Expr::TypeAssert {
            expr,
            asserted_type,
        } => {
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
            collect_import_selectors_from_type(asserted_type, import_selectors, used_selectors);
        }
        Expr::New { type_name } => {
            collect_import_selectors_from_type(type_name, import_selectors, used_selectors);
        }
        Expr::Make { type_name, args } => {
            collect_import_selectors_from_type(type_name, import_selectors, used_selectors);
            for arg in args {
                collect_import_selectors_from_expr(arg, import_selectors, used_selectors);
            }
        }
        Expr::FunctionLiteral {
            params,
            result_types,
            body,
        } => {
            for parameter in params {
                collect_import_selectors_from_type(
                    &parameter.typ,
                    import_selectors,
                    used_selectors,
                );
            }
            for result_type in result_types {
                collect_import_selectors_from_type(result_type, import_selectors, used_selectors);
            }
            collect_import_selectors_from_stmt_slice(body, import_selectors, used_selectors);
        }
        Expr::Call {
            callee,
            type_args,
            args,
        } => {
            collect_import_selectors_from_expr(callee, import_selectors, used_selectors);
            for type_arg in type_args {
                collect_import_selectors_from_type(type_arg, import_selectors, used_selectors);
            }
            for arg in args {
                collect_import_selectors_from_expr(arg, import_selectors, used_selectors);
            }
        }
        Expr::Spread { expr } => {
            collect_import_selectors_from_expr(expr, import_selectors, used_selectors);
        }
    }
}

fn collect_import_selectors_from_type(
    typ: &str,
    import_selectors: &BTreeSet<&str>,
    used_selectors: &mut BTreeSet<String>,
) {
    let bytes = typ.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        if !is_ident_start(bytes[index]) {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue(bytes[index]) {
            index += 1;
        }
        let end = index;

        let mut next = index;
        while next < bytes.len() && bytes[next].is_ascii_whitespace() {
            next += 1;
        }

        if next < bytes.len() && bytes[next] == b'.' {
            let selector = &typ[start..end];
            if import_selectors.contains(selector) {
                used_selectors.insert(selector.to_string());
            }
        }
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_continue(byte: u8) -> bool {
    is_ident_start(byte) || byte.is_ascii_digit()
}
