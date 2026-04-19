use std::collections::HashMap;

use gowasm_lexer::Span;
use serde::{Deserialize, Serialize};

use crate::{AssignTarget, Expr, FunctionDecl, Stmt};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSourceSpans {
    pub span: Span,
    pub stmt_spans: Vec<Span>,
}

impl FunctionSourceSpans {
    pub fn empty() -> Self {
        Self {
            span: Span { start: 0, end: 0 },
            stmt_spans: Vec::new(),
        }
    }

    pub fn bind(&self, function: &FunctionDecl) -> BoundFunctionSourceSpans {
        let mut stmt_spans = HashMap::new();
        let mut index = 0usize;
        bind_stmt_slice(
            &function.body,
            &self.stmt_spans,
            &mut index,
            &mut stmt_spans,
        );
        debug_assert_eq!(index, self.stmt_spans.len());
        BoundFunctionSourceSpans {
            function_span: self.span,
            stmt_spans,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFileSpans {
    pub functions: Vec<FunctionSourceSpans>,
    pub imports: Vec<Span>,
    pub vars: Vec<Span>,
    pub consts: Vec<Span>,
}

impl SourceFileSpans {
    pub fn empty() -> Self {
        Self {
            functions: Vec::new(),
            imports: Vec::new(),
            vars: Vec::new(),
            consts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFunctionSourceSpans {
    function_span: Span,
    stmt_spans: HashMap<usize, Span>,
}

impl BoundFunctionSourceSpans {
    pub fn function_span(&self) -> Span {
        self.function_span
    }

    pub fn stmt_span(&self, stmt: &Stmt) -> Option<Span> {
        self.stmt_spans.get(&stmt_key(stmt)).copied()
    }
}

fn bind_stmt_slice(
    stmts: &[Stmt],
    spans: &[Span],
    index: &mut usize,
    bound: &mut HashMap<usize, Span>,
) {
    for stmt in stmts {
        bind_stmt(stmt, spans, index, bound);
    }
}

fn bind_stmt(stmt: &Stmt, spans: &[Span], index: &mut usize, bound: &mut HashMap<usize, Span>) {
    match stmt {
        Stmt::Expr(expr) => bind_expr(expr, spans, index, bound),
        Stmt::VarDecl { value, .. } => {
            if let Some(value) = value {
                bind_expr(value, spans, index, bound);
            }
        }
        Stmt::ConstDecl { value, .. } => bind_expr(value, spans, index, bound),
        Stmt::ConstGroup { decls } => {
            for decl in decls {
                bind_expr(&decl.value, spans, index, bound);
            }
        }
        Stmt::ShortVarDecl { value, .. }
        | Stmt::ShortVarDeclPair { value, .. }
        | Stmt::ShortVarDeclTriple { value, .. }
        | Stmt::ShortVarDeclQuad { value, .. } => bind_expr(value, spans, index, bound),
        Stmt::ShortVarDeclList { values, .. } => {
            for value in values {
                bind_expr(value, spans, index, bound);
            }
        }
        Stmt::Assign { target, value } => {
            bind_assign_target(target, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignPair {
            first,
            second,
            value,
        } => {
            bind_assign_target(first, spans, index, bound);
            bind_assign_target(second, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignTriple {
            first,
            second,
            third,
            value,
        } => {
            bind_assign_target(first, spans, index, bound);
            bind_assign_target(second, spans, index, bound);
            bind_assign_target(third, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignQuad {
            first,
            second,
            third,
            fourth,
            value,
        } => {
            bind_assign_target(first, spans, index, bound);
            bind_assign_target(second, spans, index, bound);
            bind_assign_target(third, spans, index, bound);
            bind_assign_target(fourth, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignList { targets, values } => {
            for target in targets {
                bind_assign_target(target, spans, index, bound);
            }
            for value in values {
                bind_expr(value, spans, index, bound);
            }
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
                bind_embedded_stmt(init, spans, index, bound);
            }
            bind_expr(condition, spans, index, bound);
            bind_stmt_slice(then_body, spans, index, bound);
            if let Some(else_body) = else_body {
                bind_stmt_slice(else_body, spans, index, bound);
            }
        }
        Stmt::For {
            init,
            condition,
            post,
            body,
        } => {
            if let Some(init) = init {
                bind_embedded_stmt(init, spans, index, bound);
            }
            if let Some(condition) = condition {
                bind_expr(condition, spans, index, bound);
            }
            if let Some(post) = post {
                bind_embedded_stmt(post, spans, index, bound);
            }
            bind_stmt_slice(body, spans, index, bound);
        }
        Stmt::RangeFor { expr, body, .. } => {
            bind_expr(expr, spans, index, bound);
            bind_stmt_slice(body, spans, index, bound);
        }
        Stmt::Switch {
            init,
            expr,
            cases,
            default,
            ..
        } => {
            if let Some(init) = init {
                bind_embedded_stmt(init, spans, index, bound);
            }
            if let Some(expr) = expr {
                bind_expr(expr, spans, index, bound);
            }
            for case in cases {
                for expr in &case.expressions {
                    bind_expr(expr, spans, index, bound);
                }
                bind_stmt_slice(&case.body, spans, index, bound);
            }
            if let Some(default) = default {
                bind_stmt_slice(default, spans, index, bound);
            }
        }
        Stmt::TypeSwitch {
            init,
            expr,
            cases,
            default,
            ..
        } => {
            if let Some(init) = init {
                bind_embedded_stmt(init, spans, index, bound);
            }
            bind_expr(expr, spans, index, bound);
            for case in cases {
                bind_stmt_slice(&case.body, spans, index, bound);
            }
            if let Some(default) = default {
                bind_stmt_slice(default, spans, index, bound);
            }
        }
        Stmt::Select { cases, default } => {
            for case in cases {
                bind_embedded_stmt(&case.stmt, spans, index, bound);
                bind_stmt_slice(&case.body, spans, index, bound);
            }
            if let Some(default) = default {
                bind_stmt_slice(default, spans, index, bound);
            }
        }
        Stmt::Send { chan, value } => {
            bind_expr(chan, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::Go { call } | Stmt::Defer { call } => bind_expr(call, spans, index, bound),
        Stmt::Labeled { stmt, .. } => bind_stmt(stmt, spans, index, bound),
        Stmt::Return(values) => {
            for value in values {
                bind_expr(value, spans, index, bound);
            }
        }
    }

    let span = *spans
        .get(*index)
        .expect("statement span sequence should match statement traversal");
    bound.insert(stmt_key(stmt), span);
    *index += 1;
}

fn bind_embedded_stmt(
    stmt: &Stmt,
    spans: &[Span],
    index: &mut usize,
    bound: &mut HashMap<usize, Span>,
) {
    match stmt {
        Stmt::Expr(expr) => bind_expr(expr, spans, index, bound),
        Stmt::ShortVarDecl { value, .. }
        | Stmt::ShortVarDeclPair { value, .. }
        | Stmt::ShortVarDeclTriple { value, .. }
        | Stmt::ShortVarDeclQuad { value, .. } => bind_expr(value, spans, index, bound),
        Stmt::ShortVarDeclList { values, .. } => {
            for value in values {
                bind_expr(value, spans, index, bound);
            }
        }
        Stmt::Assign { target, value } => {
            bind_assign_target(target, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignPair {
            first,
            second,
            value,
        } => {
            bind_assign_target(first, spans, index, bound);
            bind_assign_target(second, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignTriple {
            first,
            second,
            third,
            value,
        } => {
            bind_assign_target(first, spans, index, bound);
            bind_assign_target(second, spans, index, bound);
            bind_assign_target(third, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignQuad {
            first,
            second,
            third,
            fourth,
            value,
        } => {
            bind_assign_target(first, spans, index, bound);
            bind_assign_target(second, spans, index, bound);
            bind_assign_target(third, spans, index, bound);
            bind_assign_target(fourth, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        Stmt::AssignList { targets, values } => {
            for target in targets {
                bind_assign_target(target, spans, index, bound);
            }
            for value in values {
                bind_expr(value, spans, index, bound);
            }
        }
        Stmt::Increment { .. } | Stmt::Decrement { .. } => {}
        Stmt::Send { chan, value } => {
            bind_expr(chan, spans, index, bound);
            bind_expr(value, spans, index, bound);
        }
        _ => {}
    }
}

fn bind_assign_target(
    target: &AssignTarget,
    spans: &[Span],
    index: &mut usize,
    bound: &mut HashMap<usize, Span>,
) {
    match target {
        AssignTarget::DerefIndex { index: expr, .. } | AssignTarget::Index { index: expr, .. } => {
            bind_expr(expr, spans, index, bound);
        }
        AssignTarget::Ident(_)
        | AssignTarget::Deref { .. }
        | AssignTarget::DerefSelector { .. }
        | AssignTarget::Selector { .. } => {}
    }
}

fn bind_expr(expr: &Expr, spans: &[Span], index: &mut usize, bound: &mut HashMap<usize, Span>) {
    match expr {
        Expr::Unary { expr, .. }
        | Expr::SliceConversion { expr, .. }
        | Expr::TypeAssert { expr, .. }
        | Expr::Spread { expr } => bind_expr(expr, spans, index, bound),
        Expr::Binary { left, right, .. } => {
            bind_expr(left, spans, index, bound);
            bind_expr(right, spans, index, bound);
        }
        Expr::ArrayLiteral { elements, .. } | Expr::SliceLiteral { elements, .. } => {
            for element in elements {
                bind_expr(element, spans, index, bound);
            }
        }
        Expr::MapLiteral { entries, .. } => {
            for entry in entries {
                bind_expr(&entry.key, spans, index, bound);
                bind_expr(&entry.value, spans, index, bound);
            }
        }
        Expr::StructLiteral { fields, .. } => {
            for field in fields {
                bind_expr(&field.value, spans, index, bound);
            }
        }
        Expr::Index {
            target,
            index: expr,
        } => {
            bind_expr(target, spans, index, bound);
            bind_expr(expr, spans, index, bound);
        }
        Expr::SliceExpr { target, low, high } => {
            bind_expr(target, spans, index, bound);
            if let Some(low) = low {
                bind_expr(low, spans, index, bound);
            }
            if let Some(high) = high {
                bind_expr(high, spans, index, bound);
            }
        }
        Expr::Selector { receiver, .. } => bind_expr(receiver, spans, index, bound),
        Expr::Make { args, .. } => {
            for arg in args {
                bind_expr(arg, spans, index, bound);
            }
        }
        Expr::FunctionLiteral { body, .. } => bind_stmt_slice(body, spans, index, bound),
        Expr::Call { callee, args, .. } => {
            bind_expr(callee, spans, index, bound);
            for arg in args {
                bind_expr(arg, spans, index, bound);
            }
        }
        Expr::Ident(_)
        | Expr::NilLiteral
        | Expr::BoolLiteral(_)
        | Expr::IntLiteral(_)
        | Expr::FloatLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::New { .. } => {}
    }
}

fn stmt_key(stmt: &Stmt) -> usize {
    stmt as *const Stmt as usize
}
