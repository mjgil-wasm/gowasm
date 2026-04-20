use super::closures::{CaptureBinding, VisibleCaptureBinding};
use gowasm_parser::{AssignTarget, Expr, Parameter, Stmt};
use std::collections::{HashMap, HashSet};

pub(crate) struct CaptureAnalyzer<'a> {
    pub(crate) visible: &'a HashMap<String, VisibleCaptureBinding>,
    scopes: Vec<HashSet<String>>,
    pub(crate) seen: HashSet<String>,
    pub(crate) captures: Vec<CaptureBinding>,
}

impl<'a> CaptureAnalyzer<'a> {
    pub(crate) fn new(
        params: &[Parameter],
        visible: &'a HashMap<String, VisibleCaptureBinding>,
    ) -> Self {
        let mut scope = HashSet::new();
        for parameter in params {
            scope.insert(parameter.name.clone());
        }
        Self {
            visible,
            scopes: vec![scope],
            seen: HashSet::new(),
            captures: Vec::new(),
        }
    }

    pub(crate) fn visit_body(&mut self, body: &[Stmt]) {
        for stmt in body {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) | Stmt::Defer { call: expr } => self.visit_expr(expr),
            Stmt::VarDecl { name, value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value);
                }
                self.declare(name);
            }
            Stmt::ConstDecl { name, value, .. } => {
                self.visit_expr(value);
                self.declare(name);
            }
            Stmt::ConstGroup { decls } => {
                for decl in decls {
                    self.visit_expr(&decl.value);
                    self.declare(&decl.name);
                }
            }
            Stmt::ShortVarDecl { name, value } => {
                self.visit_expr(value);
                self.declare(name);
            }
            Stmt::ShortVarDeclPair {
                first,
                second,
                value,
            } => {
                self.visit_expr(value);
                self.declare(first);
                self.declare(second);
            }
            Stmt::ShortVarDeclTriple {
                first,
                second,
                third,
                value,
            } => {
                self.visit_expr(value);
                self.declare(first);
                self.declare(second);
                self.declare(third);
            }
            Stmt::ShortVarDeclQuad {
                first,
                second,
                third,
                fourth,
                value,
            } => {
                self.visit_expr(value);
                self.declare(first);
                self.declare(second);
                self.declare(third);
                self.declare(fourth);
            }
            Stmt::ShortVarDeclList { names, values } => {
                for value in values {
                    self.visit_expr(value);
                }
                for name in names {
                    self.declare(name);
                }
            }
            Stmt::Assign { target, value } => {
                self.visit_assign_target(target);
                self.visit_expr(value);
            }
            Stmt::AssignPair {
                first,
                second,
                value,
            } => {
                self.visit_assign_target(first);
                self.visit_assign_target(second);
                self.visit_expr(value);
            }
            Stmt::AssignTriple {
                first,
                second,
                third,
                value,
            } => {
                self.visit_assign_target(first);
                self.visit_assign_target(second);
                self.visit_assign_target(third);
                self.visit_expr(value);
            }
            Stmt::AssignQuad {
                first,
                second,
                third,
                fourth,
                value,
            } => {
                self.visit_assign_target(first);
                self.visit_assign_target(second);
                self.visit_assign_target(third);
                self.visit_assign_target(fourth);
                self.visit_expr(value);
            }
            Stmt::AssignList { targets, values } => {
                for target in targets {
                    self.visit_assign_target(target);
                }
                for value in values {
                    self.visit_expr(value);
                }
            }
            Stmt::Increment { name } | Stmt::Decrement { name } => self.capture_name(name),
            Stmt::If {
                init,
                condition,
                then_body,
                else_body,
            } => {
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                self.visit_expr(condition);
                self.with_scope(|this| this.visit_body(then_body));
                if let Some(else_body) = else_body {
                    self.with_scope(|this| this.visit_body(else_body));
                }
            }
            Stmt::For {
                init,
                condition,
                post,
                body,
            } => self.with_scope(|this| {
                if let Some(init) = init {
                    this.visit_stmt(init);
                }
                if let Some(condition) = condition {
                    this.visit_expr(condition);
                }
                if let Some(post) = post {
                    this.visit_stmt(post);
                }
                this.visit_body(body);
            }),
            Stmt::RangeFor {
                key,
                value,
                assign,
                expr,
                body,
            } => {
                self.visit_expr(expr);
                self.with_scope(|this| {
                    if !assign && key != "_" {
                        this.declare(key);
                    }
                    if !assign {
                        if let Some(value) = value {
                            this.declare(value);
                        }
                    }
                    this.visit_body(body);
                });
            }
            Stmt::Switch {
                init,
                expr,
                cases,
                default,
                ..
            } => {
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                if let Some(expr) = expr {
                    self.visit_expr(expr);
                }
                for case in cases {
                    for expr in &case.expressions {
                        self.visit_expr(expr);
                    }
                    self.with_scope(|this| this.visit_body(&case.body));
                }
                if let Some(default) = default {
                    self.with_scope(|this| this.visit_body(default));
                }
            }
            Stmt::TypeSwitch {
                init,
                expr,
                cases,
                default,
                binding,
                ..
            } => {
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                self.visit_expr(expr);
                for case in cases {
                    self.with_scope(|this| {
                        if let Some(binding) = binding {
                            this.declare(binding);
                        }
                        this.visit_body(&case.body);
                    });
                }
                if let Some(default) = default {
                    self.with_scope(|this| {
                        if let Some(binding) = binding {
                            this.declare(binding);
                        }
                        this.visit_body(default);
                    });
                }
            }
            Stmt::Select { cases, default } => {
                for case in cases {
                    self.with_scope(|this| {
                        this.visit_stmt(&case.stmt);
                        this.visit_body(&case.body);
                    });
                }
                if let Some(default) = default {
                    self.with_scope(|this| this.visit_body(default));
                }
            }
            Stmt::Send { chan, value } => {
                self.visit_expr(chan);
                self.visit_expr(value);
            }
            Stmt::Go { call } => self.visit_expr(call),
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Labeled { stmt, .. } => self.visit_stmt(stmt),
            Stmt::Return(values) => {
                for value in values {
                    self.visit_expr(value);
                }
            }
        }
    }

    fn visit_assign_target(&mut self, target: &AssignTarget) {
        match target {
            AssignTarget::Ident(name) | AssignTarget::Deref { target: name } => {
                self.capture_name(name)
            }
            AssignTarget::DerefSelector { target, .. } => self.capture_name(target),
            AssignTarget::DerefIndex { target, index } => {
                self.capture_name(target);
                self.visit_expr(index);
            }
            AssignTarget::Selector { receiver, .. } => self.capture_name(receiver),
            AssignTarget::Index { target, index } => {
                self.capture_name(target);
                self.visit_expr(index);
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => self.capture_name(name),
            Expr::Unary { expr, .. } => self.visit_expr(expr),
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::ArrayLiteral { elements, .. } | Expr::SliceLiteral { elements, .. } => {
                for element in elements {
                    self.visit_expr(element);
                }
            }
            Expr::MapLiteral { entries, .. } => {
                for entry in entries {
                    self.visit_expr(&entry.key);
                    self.visit_expr(&entry.value);
                }
            }
            Expr::StructLiteral { fields, .. } => {
                for field in fields {
                    self.visit_expr(&field.value);
                }
            }
            Expr::Index { target, index } => {
                self.visit_expr(target);
                self.visit_expr(index);
            }
            Expr::SliceExpr { target, low, high } => {
                self.visit_expr(target);
                if let Some(low) = low {
                    self.visit_expr(low);
                }
                if let Some(high) = high {
                    self.visit_expr(high);
                }
            }
            Expr::Selector { receiver, .. } | Expr::TypeAssert { expr: receiver, .. } => {
                self.visit_expr(receiver);
            }
            Expr::FunctionLiteral { params, body, .. } => {
                let mut analyzer = CaptureAnalyzer::new(params, self.visible);
                analyzer.visit_body(body);
                for capture in analyzer.captures {
                    if self.seen.insert(capture.name.clone()) {
                        self.captures.push(capture);
                    }
                }
            }
            Expr::Make { args, .. } | Expr::Call { args, .. } => {
                if let Expr::Call { callee, .. } = expr {
                    self.visit_expr(callee);
                }
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::Spread { expr } | Expr::SliceConversion { expr, .. } => self.visit_expr(expr),
            Expr::New { .. }
            | Expr::NilLiteral
            | Expr::BoolLiteral(_)
            | Expr::IntLiteral(_)
            | Expr::FloatLiteral(_)
            | Expr::StringLiteral(_) => {}
        }
    }

    fn with_scope(&mut self, f: impl FnOnce(&mut Self)) {
        self.scopes.push(HashSet::new());
        f(self);
        self.scopes.pop().expect("scope should exist");
    }

    fn declare(&mut self, name: &str) {
        if name == "_" {
            return;
        }
        self.scopes
            .last_mut()
            .expect("scope should exist")
            .insert(name.to_string());
    }

    fn capture_name(&mut self, name: &str) {
        if name == "_" || self.is_local(name) {
            return;
        }
        let Some(visible) = self.visible.get(name) else {
            return;
        };
        if self.seen.insert(name.to_string()) {
            self.captures.push(CaptureBinding {
                name: name.to_string(),
                typ: visible.typ.clone(),
                by_ref: !visible.is_const,
                is_const: visible.is_const,
            });
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }
}
