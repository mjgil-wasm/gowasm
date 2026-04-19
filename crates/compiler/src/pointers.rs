use std::collections::HashSet;

use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn expr_is_addressable(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) => !self.binding_is_const(name) && self.binding_exists(name),
            Expr::Selector { receiver, field } => {
                self.selector_expr_is_addressable(receiver, field)
            }
            Expr::Index { target, .. } => self.index_expr_is_addressable(target),
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => self
                .infer_expr_type_name(expr)
                .and_then(|typ| parse_pointer_type(&typ).map(str::to_string))
                .is_some(),
            _ => false,
        }
    }

    fn binding_exists(&self, name: &str) -> bool {
        self.lookup_local(name).is_some()
            || self.scopes.captured_by_ref.contains(name)
            || self.lookup_global(name).is_some()
    }

    fn selector_expr_is_addressable(&self, receiver: &Expr, field: &str) -> bool {
        if !self.expr_is_addressable(receiver) {
            return false;
        }
        if let Expr::Ident(receiver_name) = receiver {
            if self.env.imported_packages.contains_key(receiver_name) {
                return false;
            }
        }
        if self.lookup_interface_type(receiver).is_some()
            || self
                .lookup_concrete_method_function(receiver, field)
                .is_some()
            || self.lookup_stdlib_value_method(receiver, field).is_some()
        {
            return false;
        }

        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return false;
        };
        self.resolve_field_selector(&receiver_type, field)
            .ok()
            .flatten()
            .is_some()
    }

    fn index_expr_is_addressable(&self, target: &Expr) -> bool {
        let Some(target_type) = self.infer_expr_type_name(target) else {
            return false;
        };
        let underlying = self.instantiated_underlying_type_name(&target_type);
        if underlying.starts_with("[]") {
            return true;
        }
        parse_array_type(&underlying).is_some() && self.expr_is_addressable(target)
    }

    pub(super) fn collect_address_taken_bindings(&self, body: &[Stmt]) -> HashSet<String> {
        let mut analyzer = AddressTakenAnalyzer::new(self);
        analyzer.visit_body(body);
        analyzer.bindings
    }

    fn pointer_type_for_expr(&self, expr: &Expr) -> TypeId {
        self.infer_expr_type_name(expr)
            .map(|typ| self.pointer_runtime_type(&typ))
            .unwrap_or(TYPE_POINTER)
    }

    pub(super) fn compile_selector_expr(
        &mut self,
        dst: usize,
        receiver: &Expr,
        field: &str,
    ) -> Result<(), CompileError> {
        let target = self.compile_selector_receiver(receiver)?;
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            self.emitter.code.push(Instruction::GetField {
                dst,
                target,
                field: field.into(),
            });
            return Ok(());
        };
        if let Some(resolution) = self.resolve_field_selector(&receiver_type, field)? {
            self.emit_selector_path_get(dst, target, &resolution.path);
            return Ok(());
        }
        if let Some(detail) = self.ambiguous_method_selector_detail(receiver, field) {
            return Err(CompileError::Unsupported { detail });
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "unknown field selector `{receiver_type}.{field}` in the current subset"
            ),
        })
    }

    pub(super) fn compile_address_of_expr(
        &mut self,
        dst: usize,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let typ = self.pointer_type_for_expr(expr);
        match expr {
            Expr::Ident(name) => {
                if self.binding_is_const(name) {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "cannot take the address of const `{name}` in the current subset"
                        ),
                    });
                }

                if let Some(src) = self.lookup_local(name) {
                    if self.scopes.captured_by_ref.contains(name) {
                        self.emitter.code.push(Instruction::Move { dst, src });
                        return Ok(());
                    }
                    self.emitter
                        .code
                        .push(Instruction::AddressLocal { dst, src, typ });
                    return Ok(());
                }
                if let Some(global) = self.lookup_global(name) {
                    self.emitter
                        .code
                        .push(Instruction::AddressGlobal { dst, global, typ });
                    return Ok(());
                }

                Err(CompileError::UnknownIdentifier { name: name.clone() })
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => {
                let src = self.compile_value_expr(expr)?;
                self.emitter.code.push(Instruction::Move { dst, src });
                Ok(())
            }
            Expr::Index { target, index } => self.compile_address_of_index(dst, target, index),
            Expr::StructLiteral { type_name, fields } => {
                let src = self.alloc_register();
                self.compile_struct_literal(src, type_name, fields)?;
                let typ = self.pointer_runtime_type(type_name);
                self.emitter
                    .code
                    .push(Instruction::AddressLocal { dst, src, typ });
                Ok(())
            }
            _ => Err(CompileError::Unsupported {
                detail: "address-of currently requires a local or package variable identifier"
                    .into(),
            }),
        }
    }

    pub(super) fn compile_address_of_selector(
        &mut self,
        dst: usize,
        receiver: &Expr,
        field: &str,
    ) -> Result<(), CompileError> {
        let selector = Expr::Selector {
            receiver: Box::new(receiver.clone()),
            field: field.into(),
        };
        let typ = self.pointer_type_for_expr(&selector);
        if self.selector_uses_implicit_pointer_deref(receiver) {
            let src = self.compile_value_expr(receiver)?;
            self.emitter.code.push(Instruction::ProjectFieldPointer {
                dst,
                src,
                field: field.into(),
                typ,
            });
            return Ok(());
        }
        match receiver {
            Expr::Ident(name) => {
                if self.binding_is_const(name) {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "cannot take the address of a field on const `{name}` in the current subset"
                        ),
                    });
                }

                if let Some(src) = self.lookup_local(name) {
                    if self.scopes.captured_by_ref.contains(name) {
                        self.emitter.code.push(Instruction::ProjectFieldPointer {
                            dst,
                            src,
                            field: field.into(),
                            typ,
                        });
                        return Ok(());
                    }
                    self.emitter.code.push(Instruction::AddressLocalField {
                        dst,
                        src,
                        field: field.into(),
                        typ,
                    });
                    return Ok(());
                }
                if let Some(global) = self.lookup_global(name) {
                    self.emitter.code.push(Instruction::AddressGlobalField {
                        dst,
                        global,
                        field: field.into(),
                        typ,
                    });
                    return Ok(());
                }

                Err(CompileError::UnknownIdentifier { name: name.clone() })
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => {
                let src = self.compile_value_expr(expr)?;
                self.emitter.code.push(Instruction::ProjectFieldPointer {
                    dst,
                    src,
                    field: field.into(),
                    typ,
                });
                Ok(())
            }
            _ if self.expr_is_addressable(receiver) => {
                let src = self.compile_addressable_expr(receiver)?;
                self.emitter.code.push(Instruction::ProjectFieldPointer {
                    dst,
                    src,
                    field: field.into(),
                    typ,
                });
                Ok(())
            }
            _ => Err(CompileError::Unsupported {
                detail: "field address-of currently requires an addressable struct field receiver"
                    .into(),
            }),
        }
    }

    fn compile_address_of_index(
        &mut self,
        dst: usize,
        target: &Expr,
        index: &Expr,
    ) -> Result<(), CompileError> {
        let indexed = Expr::Index {
            target: Box::new(target.clone()),
            index: Box::new(index.clone()),
        };
        let typ = self.pointer_type_for_expr(&indexed);
        let Some(target_type) = self.infer_expr_type_name(target) else {
            return Err(CompileError::Unsupported {
                detail: "index address-of currently requires an addressable array or slice target"
                    .into(),
            });
        };
        let underlying = self.instantiated_underlying_type_name(&target_type);
        if parse_map_type(&underlying).is_some() || underlying == "string" {
            return Err(CompileError::Unsupported {
                detail: "cannot take the address of a map or string index in the current subset"
                    .into(),
            });
        }
        if parse_array_type(&underlying).is_none() && !underlying.starts_with("[]") {
            return Err(CompileError::Unsupported {
                detail: "index address-of currently requires an addressable array or slice target"
                    .into(),
            });
        }
        let slice_temporary = underlying.starts_with("[]") && !self.expr_is_addressable(target);
        match target {
            Expr::Ident(name) => {
                if self.binding_is_const(name) {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "cannot take the address of an index on const `{name}` in the current subset"
                        ),
                    });
                }

                let index = self.compile_value_expr(index)?;
                if let Some(src) = self.lookup_local(name) {
                    if self.scopes.captured_by_ref.contains(name) {
                        self.emitter.code.push(Instruction::ProjectIndexPointer {
                            dst,
                            src,
                            index,
                            typ,
                        });
                        return Ok(());
                    }
                    self.emitter.code.push(Instruction::AddressLocalIndex {
                        dst,
                        src,
                        index,
                        typ,
                    });
                    return Ok(());
                }
                if let Some(global) = self.lookup_global(name) {
                    self.emitter.code.push(Instruction::AddressGlobalIndex {
                        dst,
                        global,
                        index,
                        typ,
                    });
                    return Ok(());
                }

                Err(CompileError::UnknownIdentifier { name: name.clone() })
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => {
                let src = self.compile_value_expr(expr)?;
                let index = self.compile_value_expr(index)?;
                self.emitter.code.push(Instruction::ProjectIndexPointer {
                    dst,
                    src,
                    index,
                    typ,
                });
                Ok(())
            }
            _ if slice_temporary => {
                let value = self.compile_value_expr(target)?;
                let base = self.alloc_register();
                self.emitter.code.push(Instruction::BoxHeap {
                    dst: base,
                    src: value,
                    typ: self.pointer_runtime_type(&target_type),
                });
                let index = self.compile_value_expr(index)?;
                self.emitter.code.push(Instruction::ProjectIndexPointer {
                    dst,
                    src: base,
                    index,
                    typ,
                });
                Ok(())
            }
            _ if self.expr_is_addressable(target) => {
                let src = self.compile_addressable_expr(target)?;
                let index = self.compile_value_expr(index)?;
                self.emitter.code.push(Instruction::ProjectIndexPointer {
                    dst,
                    src,
                    index,
                    typ,
                });
                Ok(())
            }
            _ => Err(CompileError::Unsupported {
                detail: "index address-of currently requires an addressable array or slice target"
                    .into(),
            }),
        }
    }

    pub(super) fn compile_deref_expr(
        &mut self,
        dst: usize,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        let src = self.compile_value_expr(expr)?;
        self.emitter.code.push(Instruction::Deref { dst, src });
        Ok(())
    }

    pub(super) fn compile_selector_receiver(
        &mut self,
        receiver: &Expr,
    ) -> Result<usize, CompileError> {
        if self.selector_uses_implicit_pointer_deref(receiver) {
            let pointer = self.compile_value_expr(receiver)?;
            let target = self.alloc_register();
            self.emitter.code.push(Instruction::Deref {
                dst: target,
                src: pointer,
            });
            Ok(target)
        } else {
            self.compile_value_expr(receiver)
        }
    }

    pub(super) fn selector_uses_implicit_pointer_deref(&self, receiver: &Expr) -> bool {
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return false;
        };
        let Some(inner) = parse_pointer_type(&receiver_type) else {
            return false;
        };
        self.instantiated_struct_type(inner).is_some()
    }

    pub(super) fn emit_selector_path_get(&mut self, dst: usize, target: usize, path: &[String]) {
        let mut current = target;
        for (index, field) in path.iter().enumerate() {
            let last = index + 1 == path.len();
            let next = if last { dst } else { self.alloc_register() };
            self.emitter.code.push(Instruction::GetField {
                dst: next,
                target: current,
                field: field.clone(),
            });
            current = next;
        }
    }
}

struct AddressTakenAnalyzer<'a, 'b> {
    builder: &'a FunctionBuilder<'b>,
    bindings: HashSet<String>,
}

impl<'a, 'b> AddressTakenAnalyzer<'a, 'b> {
    fn new(builder: &'a FunctionBuilder<'b>) -> Self {
        Self {
            builder,
            bindings: HashSet::new(),
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
            Stmt::VarDecl { value, .. } => {
                if let Some(value) = value {
                    self.visit_expr(value);
                }
            }
            Stmt::ConstDecl { value, .. } => self.visit_expr(value),
            Stmt::ConstGroup { decls } => {
                for decl in decls {
                    self.visit_expr(&decl.value);
                }
            }
            Stmt::ShortVarDecl { value, .. }
            | Stmt::ShortVarDeclPair { value, .. }
            | Stmt::ShortVarDeclTriple { value, .. }
            | Stmt::ShortVarDeclQuad { value, .. }
            | Stmt::Assign { value, .. }
            | Stmt::AssignPair { value, .. }
            | Stmt::AssignTriple { value, .. }
            | Stmt::AssignQuad { value, .. } => self.visit_expr(value),
            Stmt::ShortVarDeclList { values, .. } | Stmt::AssignList { values, .. } => {
                for value in values {
                    self.visit_expr(value);
                }
            }
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
                self.visit_body(then_body);
                if let Some(else_body) = else_body {
                    self.visit_body(else_body);
                }
            }
            Stmt::For {
                init,
                condition,
                post,
                body,
            } => {
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
            }
            Stmt::RangeFor { expr, body, .. } => {
                self.visit_expr(expr);
                self.visit_body(body);
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
                    self.visit_body(&case.body);
                }
                if let Some(default) = default {
                    self.visit_body(default);
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
                    self.visit_stmt(init);
                }
                self.visit_expr(expr);
                for case in cases {
                    self.visit_body(&case.body);
                }
                if let Some(default) = default {
                    self.visit_body(default);
                }
            }
            Stmt::Select { cases, default } => {
                for case in cases {
                    self.visit_stmt(&case.stmt);
                    self.visit_body(&case.body);
                }
                if let Some(default) = default {
                    self.visit_body(default);
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
            Stmt::Increment { .. }
            | Stmt::Decrement { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. } => {}
            Stmt::Labeled { stmt, .. } => self.visit_stmt(stmt),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Unary {
                op: UnaryOp::AddressOf,
                expr,
            } => {
                self.record_address_target(expr);
                self.visit_expr(expr);
            }
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
            Expr::Spread { expr } | Expr::SliceConversion { expr, .. } => {
                self.visit_expr(expr);
            }
            Expr::Call { callee, args, .. } => {
                if let Expr::Selector { receiver, field } = callee.as_ref() {
                    if self
                        .builder
                        .receiver_uses_implicit_address_of(receiver, field)
                        || self
                            .builder
                            .lookup_stdlib_value_method(receiver, field)
                            .is_some_and(stdlib_function_mutates_first_arg)
                    {
                        self.record_address_target(receiver);
                    }
                }
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::FunctionLiteral { .. }
            | Expr::Ident(_)
            | Expr::NilLiteral
            | Expr::BoolLiteral(_)
            | Expr::IntLiteral(_)
            | Expr::FloatLiteral(_)
            | Expr::StringLiteral(_)
            | Expr::Make { .. }
            | Expr::New { .. } => {}
        }
    }

    fn record_address_target(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => {
                if name != "_" {
                    self.bindings.insert(name.clone());
                }
            }
            Expr::Selector { receiver, .. } => self.record_address_target(receiver),
            Expr::Index { target, .. } => self.record_address_target(target),
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => self.record_address_target(expr),
            _ => {}
        }
    }
}
