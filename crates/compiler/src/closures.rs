use super::*;
use crate::capture_analyzer::CaptureAnalyzer;
use crate::types::{format_function_type, InstanceKey, InstantiationCache};
use gowasm_parser::Parameter;
use gowasm_vm::{Function, FunctionDebugInfo, Instruction, InstructionSourceSpan};
use std::collections::{HashMap, HashSet};

pub(crate) struct GeneratedFunctions {
    base: usize,
    functions: Vec<Option<CompiledFunction>>,
    function_instances: HashMap<String, usize>,
}

pub(crate) fn collect_direct_by_ref_captures(
    initial_bindings: impl IntoIterator<Item = (String, bool)>,
    body: &[Stmt],
) -> HashSet<String> {
    let mut analyzer = DirectCaptureAnalyzer::new(initial_bindings);
    analyzer.visit_body(body);
    analyzer.by_ref_captures
}

#[derive(Debug, Clone)]
pub(crate) struct VisibleCaptureBinding {
    pub(crate) typ: String,
    pub(crate) is_const: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CaptureBinding {
    pub(crate) name: String,
    pub(crate) typ: String,
    pub(crate) by_ref: bool,
    pub(crate) is_const: bool,
}

impl GeneratedFunctions {
    pub(crate) fn new(base: usize) -> Self {
        Self {
            base,
            functions: Vec::new(),
            function_instances: HashMap::new(),
        }
    }

    pub(crate) fn reserve(&mut self) -> usize {
        let id = self.base + self.functions.len();
        self.functions.push(None);
        id
    }

    pub(crate) fn reserve_instance(
        &mut self,
        key: InstanceKey,
        instantiation_cache: &mut InstantiationCache,
    ) -> (String, usize, bool) {
        let name = instantiation_cache.function_name(&key);
        if let Some(function) = self.function_instances.get(&name).copied() {
            return (name, function, false);
        }
        let function = self.reserve();
        self.function_instances.insert(name.clone(), function);
        (name, function, true)
    }

    pub(crate) fn fill(&mut self, id: usize, function: CompiledFunction) {
        let slot = id
            .checked_sub(self.base)
            .and_then(|index| self.functions.get_mut(index))
            .expect("generated function slot should exist");
        *slot = Some(function);
    }

    pub(crate) fn instance_function_ids(&self) -> &HashMap<String, usize> {
        &self.function_instances
    }

    pub(crate) fn append_into(
        self,
        functions: &mut Vec<Function>,
        debug_infos: &mut Vec<FunctionDebugInfo>,
    ) {
        for function in self.functions {
            let function = function.expect("generated function should be filled");
            debug_infos.push(function.debug_info);
            functions.push(function.function);
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_function_literal(
        &mut self,
        dst: usize,
        params: &[Parameter],
        result_types: &[String],
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        let captures = self.collect_closure_captures(params, body);
        let capture_registers = self.compile_closure_capture_values(&captures)?;
        let function = self.generation.generated_functions.reserve();
        let nested = self.build_generated_function(
            format!("__gowasm_closure${function}"),
            &captures,
            params,
            result_types,
            body,
        )?;
        self.generation.generated_functions.fill(function, nested);
        let param_types = params
            .iter()
            .map(|param| param.typ.clone())
            .collect::<Vec<_>>();
        self.emitter.code.push(Instruction::MakeClosure {
            dst,
            concrete_type: Some(
                self.lower_runtime_concrete_type(&format_function_type(
                    &param_types,
                    result_types,
                ))?,
            ),
            function,
            captures: capture_registers,
        });
        Ok(())
    }

    fn build_generated_function(
        &mut self,
        name: String,
        captures: &[CaptureBinding],
        params: &[Parameter],
        result_types: &[String],
        body: &[Stmt],
    ) -> Result<CompiledFunction, CompileError> {
        let mut locals = HashMap::new();
        let mut local_types = HashMap::new();
        let mut captured_by_ref = collect_direct_by_ref_captures(
            captures
                .iter()
                .map(|capture| (capture.name.clone(), capture.is_const))
                .chain(
                    params
                        .iter()
                        .map(|parameter| (parameter.name.clone(), false)),
                ),
            body,
        );
        let mut consts = HashSet::new();
        let mut next_param = 0usize;
        for capture in captures {
            locals.insert(capture.name.clone(), next_param);
            local_types.insert(capture.name.clone(), capture.typ.clone());
            if capture.by_ref {
                captured_by_ref.insert(capture.name.clone());
            }
            if capture.is_const {
                consts.insert(capture.name.clone());
            }
            next_param += 1;
        }
        for parameter in params {
            locals.insert(parameter.name.clone(), next_param);
            local_types.insert(parameter.name.clone(), parameter.typ.clone());
            next_param += 1;
        }

        let imported_packages = self.env.imported_packages.clone();
        let function_ids = self.env.function_ids;
        let function_result_types = self.env.function_result_types;
        let function_types = self.env.function_types;
        let variadic_functions = self.env.variadic_functions;
        let generic_functions = self.env.generic_functions;
        let generic_types = self.env.generic_types;
        let generic_function_templates = self.env.generic_function_templates;
        let generic_method_templates = self.env.generic_method_templates;
        let instantiation_cache = &mut *self.generation.instantiation_cache;
        let method_function_ids = self.env.method_function_ids;
        let promoted_method_bindings = self.env.promoted_method_bindings;
        let struct_types = self.env.struct_types;
        let pointer_types = self.env.pointer_types;
        let interface_types = self.env.interface_types;
        let imported_package_tables = self.env.imported_package_tables;
        let globals = self.env.globals;
        let method_sets = self.env.method_sets;
        let generated_functions = &mut *self.generation.generated_functions;
        let instantiated_generics = &mut *self.generation.instantiated_generics;
        let mut code = InstructionBuffer::default();
        code.set_active_span(self.emitter.code.active_span().or_else(|| {
            self.emitter
                .source_spans
                .as_ref()
                .map(|source_spans| InstructionSourceSpan {
                    path: self
                        .emitter
                        .default_source_path
                        .clone()
                        .expect("generated function source path should exist"),
                    start: source_spans.function_span().start,
                    end: source_spans.function_span().end,
                })
        }));
        let mut builder = FunctionBuilder {
            emitter: EmitterState {
                code,
                next_register: next_param,
                default_source_path: self.emitter.default_source_path.clone(),
                source_spans: self.emitter.source_spans.clone(),
            },
            env: CompilerEnvironment::new(
                ImportContext {
                    imported_packages,
                    imported_package_tables,
                },
                SymbolTables {
                    function_ids,
                    function_result_types,
                    function_types,
                    variadic_functions,
                    method_function_ids,
                    promoted_method_bindings,
                    globals,
                    method_sets,
                },
                TypeContext {
                    generic_functions,
                    generic_types,
                    generic_function_templates,
                    generic_method_templates,
                },
                RuntimeMetadataContext {
                    struct_types,
                    pointer_types,
                    interface_types,
                    alias_types: self.env.alias_types,
                },
            ),
            generation: GenerationState {
                instantiation_cache,
                generated_functions,
                instantiated_generics,
                generic_instance_namespace: self.generation.generic_instance_namespace.clone(),
            },
            scopes: ScopeStack {
                scopes: vec![locals],
                captured_by_ref,
                const_scopes: vec![consts],
                const_value_scopes: vec![HashMap::new()],
                type_scopes: vec![local_types],
            },
            control: ControlFlowContext {
                in_package_init: false,
                current_result_types: result_types.to_vec(),
                current_result_names: Vec::new(),
                break_scopes: Vec::new(),
                loops: Vec::new(),
                pending_label: None,
            },
        };
        builder
            .scopes
            .captured_by_ref
            .extend(builder.collect_address_taken_bindings(body));
        builder.box_captured_parameters(params);

        for stmt in body {
            builder.compile_stmt(stmt)?;
        }
        builder.emitter.code.push(Instruction::Return { src: None });
        let (code, debug_info) = builder.emitter.code.into_parts();

        Ok(CompiledFunction {
            function: Function {
                name,
                param_count: next_param,
                register_count: builder.emitter.next_register,
                code,
            },
            debug_info,
        })
    }

    fn compile_closure_capture_values(
        &mut self,
        captures: &[CaptureBinding],
    ) -> Result<Vec<usize>, CompileError> {
        let mut registers = Vec::with_capacity(captures.len());
        for capture in captures {
            if capture.by_ref {
                let register = self.alloc_register();
                self.compile_address_of_expr(register, &Expr::Ident(capture.name.clone()))?;
                registers.push(register);
            } else {
                registers.push(self.compile_value_expr(&Expr::Ident(capture.name.clone()))?);
            }
        }
        Ok(registers)
    }

    fn collect_closure_captures(&self, params: &[Parameter], body: &[Stmt]) -> Vec<CaptureBinding> {
        let visible = self.visible_capture_bindings();
        let mut analyzer = CaptureAnalyzer::new(params, &visible);
        analyzer.visit_body(body);
        analyzer.captures
    }

    fn visible_capture_bindings(&self) -> HashMap<String, VisibleCaptureBinding> {
        let mut visible = HashMap::new();
        for ((scope, consts), types) in self
            .scopes
            .scopes
            .iter()
            .rev()
            .zip(self.scopes.const_scopes.iter().rev())
            .zip(self.scopes.type_scopes.iter().rev())
        {
            for name in scope.keys() {
                if name == "_" || visible.contains_key(name) {
                    continue;
                }
                let Some(typ) = types.get(name) else {
                    continue;
                };
                visible.insert(
                    name.clone(),
                    VisibleCaptureBinding {
                        typ: typ.clone(),
                        is_const: consts.contains(name),
                    },
                );
            }
        }
        visible
    }
}

struct DirectCaptureAnalyzer {
    scopes: Vec<HashMap<String, bool>>,
    by_ref_captures: HashSet<String>,
}

impl DirectCaptureAnalyzer {
    fn new(initial_bindings: impl IntoIterator<Item = (String, bool)>) -> Self {
        Self {
            scopes: vec![initial_bindings.into_iter().collect()],
            by_ref_captures: HashSet::new(),
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
                self.declare(name, false);
            }
            Stmt::ConstDecl { name, value, .. } => {
                self.visit_expr(value);
                self.declare(name, true);
            }
            Stmt::ConstGroup { decls } => {
                for decl in decls {
                    self.visit_expr(&decl.value);
                    self.declare(&decl.name, true);
                }
            }
            Stmt::ShortVarDecl { name, value } => {
                self.visit_expr(value);
                self.declare(name, false);
            }
            Stmt::ShortVarDeclPair {
                first,
                second,
                value,
            } => {
                self.visit_expr(value);
                self.declare(first, false);
                self.declare(second, false);
            }
            Stmt::ShortVarDeclTriple {
                first,
                second,
                third,
                value,
            } => {
                self.visit_expr(value);
                self.declare(first, false);
                self.declare(second, false);
                self.declare(third, false);
            }
            Stmt::ShortVarDeclQuad {
                first,
                second,
                third,
                fourth,
                value,
            } => {
                self.visit_expr(value);
                self.declare(first, false);
                self.declare(second, false);
                self.declare(third, false);
                self.declare(fourth, false);
            }
            Stmt::ShortVarDeclList { names, values } => {
                for value in values {
                    self.visit_expr(value);
                }
                for name in names {
                    self.declare(name, false);
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
            Stmt::Increment { .. }
            | Stmt::Decrement { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. } => {}
            Stmt::Labeled { stmt, .. } => self.visit_stmt(stmt),
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
                        this.declare(key, false);
                    }
                    if !assign {
                        if let Some(value) = value {
                            this.declare(value, false);
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
                            this.declare(binding, false);
                        }
                        this.visit_body(&case.body);
                    });
                }
                if let Some(default) = default {
                    self.with_scope(|this| {
                        if let Some(binding) = binding {
                            this.declare(binding, false);
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
            Stmt::Return(values) => {
                for value in values {
                    self.visit_expr(value);
                }
            }
        }
    }

    fn visit_assign_target(&mut self, target: &AssignTarget) {
        match target {
            AssignTarget::DerefIndex { index, .. } | AssignTarget::Index { index, .. } => {
                self.visit_expr(index);
            }
            AssignTarget::Ident(_)
            | AssignTarget::Deref { .. }
            | AssignTarget::DerefSelector { .. }
            | AssignTarget::Selector { .. } => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
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
            Expr::Make { args, .. } | Expr::Call { args, .. } => {
                if let Expr::Call { callee, .. } = expr {
                    self.visit_expr(callee);
                }
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::FunctionLiteral { params, body, .. } => {
                let visible = self.visible_bindings();
                let mut analyzer = CaptureAnalyzer::new(params, &visible);
                analyzer.visit_body(body);
                for capture in analyzer.captures {
                    if capture.by_ref {
                        self.by_ref_captures.insert(capture.name);
                    }
                }
            }
            Expr::Spread { expr } | Expr::SliceConversion { expr, .. } => self.visit_expr(expr),
            Expr::New { .. }
            | Expr::Ident(_)
            | Expr::NilLiteral
            | Expr::BoolLiteral(_)
            | Expr::IntLiteral(_)
            | Expr::FloatLiteral(_)
            | Expr::StringLiteral(_) => {}
        }
    }

    fn with_scope(&mut self, f: impl FnOnce(&mut Self)) {
        self.scopes.push(HashMap::new());
        f(self);
        self.scopes.pop().expect("scope should exist");
    }

    fn declare(&mut self, name: &str, is_const: bool) {
        if name == "_" {
            return;
        }
        self.scopes
            .last_mut()
            .expect("scope should exist")
            .insert(name.to_string(), is_const);
    }

    fn visible_bindings(&self) -> HashMap<String, VisibleCaptureBinding> {
        let mut visible = HashMap::new();
        for scope in self.scopes.iter().rev() {
            for (name, is_const) in scope {
                if visible.contains_key(name) {
                    continue;
                }
                visible.insert(
                    name.clone(),
                    VisibleCaptureBinding {
                        typ: String::new(),
                        is_const: *is_const,
                    },
                );
            }
        }
        visible
    }
}
