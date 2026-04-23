use std::collections::{HashMap, HashSet};

use gowasm_parser::{
    AssignTarget, Expr, FunctionDecl, PackageVarDecl, SourceFile, SourceFileSpans, Stmt,
};

use crate::CompileError;

pub(crate) struct PackageInitFile<'a> {
    pub(crate) path: &'a str,
    pub(crate) file: &'a SourceFile,
    pub(crate) spans: Option<&'a SourceFileSpans>,
}

pub(crate) struct OrderedPackageVarInit<'a> {
    pub(crate) path: &'a str,
    pub(crate) spans: Option<&'a SourceFileSpans>,
    pub(crate) index: usize,
    pub(crate) decl: &'a PackageVarDecl,
}

struct PackageVarInitDecl<'a> {
    path: &'a str,
    spans: Option<&'a SourceFileSpans>,
    index: usize,
    decl: &'a PackageVarDecl,
    deps: HashSet<String>,
}

struct FunctionDependencyContext<'a> {
    package_vars: &'a HashSet<String>,
    package_consts: &'a HashSet<String>,
    function_bodies: &'a HashMap<String, &'a FunctionDecl>,
}

pub(crate) fn order_package_var_inits<'a>(
    files: &[PackageInitFile<'a>],
) -> Result<Vec<OrderedPackageVarInit<'a>>, CompileError> {
    let package_vars = files
        .iter()
        .flat_map(|file| file.file.vars.iter().map(|var| var.name.clone()))
        .collect::<HashSet<_>>();
    let package_consts = files
        .iter()
        .flat_map(|file| {
            file.file
                .consts
                .iter()
                .map(|constant| constant.name.clone())
        })
        .collect::<HashSet<_>>();
    let function_bodies = files
        .iter()
        .flat_map(|file| {
            file.file
                .functions
                .iter()
                .filter(|function| function.receiver.is_none())
                .map(|function| (function.name.clone(), function))
        })
        .collect::<HashMap<_, _>>();
    let context = FunctionDependencyContext {
        package_vars: &package_vars,
        package_consts: &package_consts,
        function_bodies: &function_bodies,
    };

    let mut function_cache = HashMap::new();
    let mut function_visiting = HashSet::new();
    let mut declarations = Vec::new();
    for file in files {
        let imported = file
            .file
            .imports
            .iter()
            .map(|decl| {
                decl.path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&decl.path)
                    .to_string()
            })
            .collect::<HashSet<_>>();
        for (index, decl) in file.file.vars.iter().enumerate() {
            let deps = decl.value.as_ref().map_or_else(HashSet::new, |value| {
                expr_dependencies(
                    value,
                    &imported,
                    &context,
                    &mut function_cache,
                    &mut function_visiting,
                )
            });
            declarations.push(PackageVarInitDecl {
                path: file.path,
                spans: file.spans,
                index,
                decl,
                deps,
            });
        }
    }

    let mut initialized = HashSet::new();
    let mut emitted = vec![false; declarations.len()];
    let mut ordered = Vec::with_capacity(declarations.len());
    while ordered.len() < declarations.len() {
        let mut progress = false;
        for (declaration_index, declaration) in declarations.iter().enumerate() {
            if emitted[declaration_index] {
                continue;
            }
            if declaration
                .deps
                .iter()
                .all(|dependency| initialized.contains(dependency))
            {
                ordered.push(OrderedPackageVarInit {
                    path: declaration.path,
                    spans: declaration.spans,
                    index: declaration.index,
                    decl: declaration.decl,
                });
                initialized.insert(declaration.decl.name.clone());
                emitted[declaration_index] = true;
                progress = true;
            }
        }

        if progress {
            continue;
        }

        let remaining = declarations
            .iter()
            .enumerate()
            .filter(|(index, _)| !emitted[*index])
            .map(|(_, declaration)| declaration.decl.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(CompileError::Unsupported {
            detail: format!("package initialization cycle involving {remaining}"),
        });
    }

    Ok(ordered)
}

fn expr_dependencies<'a>(
    expr: &'a Expr,
    imported: &HashSet<String>,
    context: &FunctionDependencyContext<'a>,
    function_cache: &mut HashMap<String, HashSet<String>>,
    function_visiting: &mut HashSet<String>,
) -> HashSet<String> {
    let mut collector =
        DependencyCollector::new(imported, context, function_cache, function_visiting);
    collector.visit_expr(expr);
    collector.deps
}

fn function_dependencies<'a>(
    function_name: &str,
    imported: &HashSet<String>,
    context: &FunctionDependencyContext<'a>,
    function_cache: &mut HashMap<String, HashSet<String>>,
    function_visiting: &mut HashSet<String>,
) -> HashSet<String> {
    if let Some(cached) = function_cache.get(function_name) {
        return cached.clone();
    }
    if !function_visiting.insert(function_name.to_string()) {
        return HashSet::new();
    }

    let deps = context
        .function_bodies
        .get(function_name)
        .map(|function| {
            let mut collector =
                DependencyCollector::new(imported, context, function_cache, function_visiting);
            collector.push_scope(function.params.iter().map(|param| param.name.as_str()));
            collector.visit_body(&function.body);
            collector.pop_scope();
            collector.deps
        })
        .unwrap_or_default();

    function_visiting.remove(function_name);
    function_cache.insert(function_name.to_string(), deps.clone());
    deps
}

struct DependencyCollector<'a, 'ctx> {
    imported: &'a HashSet<String>,
    context: &'ctx FunctionDependencyContext<'ctx>,
    function_cache: &'a mut HashMap<String, HashSet<String>>,
    function_visiting: &'a mut HashSet<String>,
    scopes: Vec<HashSet<String>>,
    deps: HashSet<String>,
}

impl<'a, 'ctx> DependencyCollector<'a, 'ctx> {
    fn new(
        imported: &'a HashSet<String>,
        context: &'ctx FunctionDependencyContext<'ctx>,
        function_cache: &'a mut HashMap<String, HashSet<String>>,
        function_visiting: &'a mut HashSet<String>,
    ) -> Self {
        Self {
            imported,
            context,
            function_cache,
            function_visiting,
            scopes: vec![HashSet::new()],
            deps: HashSet::new(),
        }
    }

    fn visit_body(&mut self, body: &[Stmt]) {
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
            Stmt::Increment { name } | Stmt::Decrement { name } => self.record_ident(name),
            Stmt::If {
                init,
                condition,
                then_body,
                else_body,
            } => {
                self.push_scope(std::iter::empty::<&str>());
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                self.visit_expr(condition);
                self.visit_scoped_body(then_body);
                if let Some(else_body) = else_body {
                    self.visit_scoped_body(else_body);
                }
                self.pop_scope();
            }
            Stmt::For {
                init,
                condition,
                post,
                body,
            } => {
                self.push_scope(std::iter::empty::<&str>());
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                if let Some(condition) = condition {
                    self.visit_expr(condition);
                }
                if let Some(post) = post {
                    self.visit_stmt(post);
                }
                self.visit_body(body);
                self.pop_scope();
            }
            Stmt::RangeFor {
                key,
                value,
                assign,
                expr,
                body,
            } => {
                self.visit_expr(expr);
                self.push_scope(std::iter::empty::<&str>());
                if !assign && key != "_" {
                    self.declare(key);
                }
                if !assign {
                    if let Some(value) = value {
                        self.declare(value);
                    }
                }
                self.visit_body(body);
                self.pop_scope();
            }
            Stmt::Switch {
                init,
                expr,
                cases,
                default,
                ..
            } => {
                self.push_scope(std::iter::empty::<&str>());
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
                    self.visit_scoped_body(&case.body);
                }
                if let Some(default) = default {
                    self.visit_scoped_body(default);
                }
                self.pop_scope();
            }
            Stmt::TypeSwitch {
                init,
                expr,
                cases,
                default,
                binding,
                ..
            } => {
                self.push_scope(std::iter::empty::<&str>());
                if let Some(init) = init {
                    self.visit_stmt(init);
                }
                self.visit_expr(expr);
                for case in cases {
                    self.push_scope(std::iter::empty::<&str>());
                    if let Some(binding) = binding {
                        self.declare(binding);
                    }
                    self.visit_body(&case.body);
                    self.pop_scope();
                }
                if let Some(default) = default {
                    self.push_scope(std::iter::empty::<&str>());
                    if let Some(binding) = binding {
                        self.declare(binding);
                    }
                    self.visit_body(default);
                    self.pop_scope();
                }
                self.pop_scope();
            }
            Stmt::Select { cases, default } => {
                for case in cases {
                    self.push_scope(std::iter::empty::<&str>());
                    self.visit_stmt(&case.stmt);
                    self.visit_body(&case.body);
                    self.pop_scope();
                }
                if let Some(default) = default {
                    self.visit_scoped_body(default);
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

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => self.record_ident(name),
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
            Expr::New { .. }
            | Expr::NilLiteral
            | Expr::BoolLiteral(_)
            | Expr::IntLiteral(_)
            | Expr::FloatLiteral(_)
            | Expr::StringLiteral(_) => {}
            Expr::Make { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::FunctionLiteral { .. } => {}
            Expr::Call { callee, args, .. } => {
                match callee.as_ref() {
                    Expr::Ident(name)
                        if !self.is_local(name)
                            && !self.imported.contains(name)
                            && self.context.function_bodies.contains_key(name) =>
                    {
                        self.deps.extend(function_dependencies(
                            name,
                            self.imported,
                            self.context,
                            self.function_cache,
                            self.function_visiting,
                        ));
                    }
                    Expr::FunctionLiteral { params, body, .. } => {
                        self.push_scope(params.iter().map(|param| param.name.as_str()));
                        self.visit_body(body);
                        self.pop_scope();
                    }
                    _ => self.visit_expr(callee),
                }
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::Spread { expr } | Expr::SliceConversion { expr, .. } => self.visit_expr(expr),
        }
    }

    fn visit_assign_target(&mut self, target: &AssignTarget) {
        match target {
            AssignTarget::Ident(name) | AssignTarget::Deref { target: name } => {
                self.record_ident(name)
            }
            AssignTarget::DerefSelector { target, .. } => self.record_ident(target),
            AssignTarget::DerefIndex { target, index } => {
                self.record_ident(target);
                self.visit_expr(index);
            }
            AssignTarget::Selector { receiver, .. } => self.record_ident(receiver),
            AssignTarget::Index { target, index } => {
                self.record_ident(target);
                self.visit_expr(index);
            }
        }
    }

    fn visit_scoped_body(&mut self, body: &[Stmt]) {
        self.push_scope(std::iter::empty::<&str>());
        self.visit_body(body);
        self.pop_scope();
    }

    fn push_scope<'b>(&mut self, names: impl IntoIterator<Item = &'b str>) {
        let mut scope = HashSet::new();
        for name in names {
            if !name.is_empty() {
                scope.insert(name.to_string());
            }
        }
        self.scopes.push(scope);
    }

    fn pop_scope(&mut self) {
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

    fn is_local(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    fn record_ident(&mut self, name: &str) {
        if name == "_" || self.is_local(name) || self.imported.contains(name) {
            return;
        }
        if self.context.package_consts.contains(name) {
            return;
        }
        if self.context.package_vars.contains(name) {
            self.deps.insert(name.to_string());
        }
    }
}
