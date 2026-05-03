use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn validate_comparison_operands(
        &self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), CompileError> {
        match op {
            BinaryOp::Equal | BinaryOp::NotEqual => self.validate_equality_operands(left, right),
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                self.validate_ordered_operands(left, right)
            }
            _ => Ok(()),
        }
    }

    pub(super) fn try_compile_interface_nil_compare(
        &mut self,
        dst: usize,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<bool, CompileError> {
        if !matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) {
            return Ok(false);
        }

        let src = if matches!(left, Expr::NilLiteral) {
            self.interface_compare_register(right)?
        } else if matches!(right, Expr::NilLiteral) {
            self.interface_compare_register(left)?
        } else {
            None
        };

        let Some(src) = src else {
            return Ok(false);
        };

        self.emitter.code.push(Instruction::IsNil { dst, src });
        if matches!(op, BinaryOp::NotEqual) {
            self.emitter.code.push(Instruction::Not { dst, src: dst });
        }
        Ok(true)
    }

    pub(super) fn compile_logical_and(
        &mut self,
        dst: usize,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), CompileError> {
        self.compile_expr_into(dst, left)?;
        let jump_to_end = self.push_jump_if_false(dst);
        self.compile_expr_into(dst, right)?;
        let end_target = self.emitter.code.len();
        self.patch_jump_if_false(jump_to_end, end_target);
        Ok(())
    }

    pub(super) fn compile_logical_or(
        &mut self,
        dst: usize,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), CompileError> {
        self.compile_expr_into(dst, left)?;
        let jump_to_rhs = self.push_jump_if_false(dst);
        let jump_to_end = self.push_jump();
        let rhs_target = self.emitter.code.len();
        self.patch_jump_if_false(jump_to_rhs, rhs_target);
        self.compile_expr_into(dst, right)?;
        let end_target = self.emitter.code.len();
        self.patch_jump(jump_to_end, end_target);
        Ok(())
    }

    fn interface_compare_register(&mut self, expr: &Expr) -> Result<Option<usize>, CompileError> {
        if self.lookup_interface_type(expr).is_none() {
            return Ok(None);
        }
        self.compile_value_expr(expr).map(Some)
    }

    fn validate_equality_operands(&self, left: &Expr, right: &Expr) -> Result<(), CompileError> {
        if matches!(left, Expr::NilLiteral) && matches!(right, Expr::NilLiteral) {
            return Err(self.unsupported_with_active_span(
                "cannot compare `nil` to `nil` in the current subset",
            ));
        }

        if matches!(left, Expr::NilLiteral) {
            return self.validate_nil_comparison_target(right);
        }
        if matches!(right, Expr::NilLiteral) {
            return self.validate_nil_comparison_target(left);
        }

        let Some(left_type) = self.infer_expr_type_name(left) else {
            return Ok(());
        };
        let Some(right_type) = self.infer_expr_type_name(right) else {
            return Ok(());
        };

        if left_type == right_type {
            return self.ensure_type_comparable(&left_type);
        }

        if self.type_is_interface_type(&left_type) && self.type_is_interface_type(&right_type) {
            return Ok(());
        }

        if self.type_is_interface_type(&left_type) {
            return self.ensure_type_comparable(&right_type);
        }
        if self.type_is_interface_type(&right_type) {
            return self.ensure_type_comparable(&left_type);
        }

        if self.types_assignable(&left_type, &right_type)
            && self.types_assignable(&right_type, &left_type)
        {
            self.ensure_type_comparable(&left_type)?;
            return self.ensure_type_comparable(&right_type);
        }

        Err(self.unsupported_with_active_span(format!(
            "cannot compare `{}` and `{}` in the current subset",
            display_type(&left_type),
            display_type(&right_type),
        )))
    }

    fn validate_nil_comparison_target(&self, expr: &Expr) -> Result<(), CompileError> {
        let Some(typ) = self.infer_expr_type_name(expr) else {
            return Ok(());
        };
        if self.type_allows_nil(&typ) {
            return Ok(());
        }
        Err(self.unsupported_with_active_span(format!(
            "type `{}` is not comparable to `nil` in the current subset",
            display_type(&typ),
        )))
    }

    fn validate_ordered_operands(&self, left: &Expr, right: &Expr) -> Result<(), CompileError> {
        let Some(left_type) = self.infer_expr_type_name(left) else {
            return Ok(());
        };
        let Some(right_type) = self.infer_expr_type_name(right) else {
            return Ok(());
        };

        if left_type != right_type
            && !(self.types_assignable(&left_type, &right_type)
                && self.types_assignable(&right_type, &left_type))
        {
            return Err(self.unsupported_with_active_span(format!(
                "cannot order values of types `{}` and `{}` in the current subset",
                display_type(&left_type),
                display_type(&right_type),
            )));
        }

        if self.type_is_ordered(&left_type) && self.type_is_ordered(&right_type) {
            return Ok(());
        }

        Err(self.unsupported_with_active_span(format!(
            "type `{}` is not ordered in the current subset",
            display_type(&left_type),
        )))
    }

    fn ensure_type_comparable(&self, typ: &str) -> Result<(), CompileError> {
        if self.type_is_comparable(typ) {
            return Ok(());
        }
        Err(self.unsupported_with_active_span(format!(
            "type `{}` is not comparable",
            display_type(typ),
        )))
    }

    fn type_is_ordered(&self, typ: &str) -> bool {
        matches!(
            self.instantiated_underlying_type_name(typ).as_str(),
            "int" | "float64" | "string"
        )
    }

    fn type_is_interface_type(&self, typ: &str) -> bool {
        matches!(typ, "interface{}" | "any" | "error")
            || self.instantiated_interface_type(typ).is_some()
    }
}
