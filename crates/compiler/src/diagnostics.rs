use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum CallableContext {
    PlainCall,
    MultiCall { expected_results: usize },
    Go,
    Defer,
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_call_args(
        &mut self,
        name: &str,
        args: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        self.compile_named_call_args(name, args)
    }

    pub(super) fn local_non_callable_call_error(
        &self,
        name: &str,
        callee: &Expr,
        context: CallableContext,
    ) -> CompileError {
        let detail = match context {
            CallableContext::PlainCall | CallableContext::MultiCall { .. } => format!(
                "calling local variable `{name}` is not supported: {}",
                self.call_target_detail(callee, context)
            ),
            CallableContext::Go => format!(
                "`go` cannot call local variable `{name}`: {}",
                self.call_target_detail(callee, context)
            ),
            CallableContext::Defer => format!(
                "`defer` cannot call local variable `{name}`: {}",
                self.call_target_detail(callee, context)
            ),
        };
        self.unsupported_with_active_span(detail)
    }

    pub(super) fn global_non_callable_call_error(
        &self,
        name: &str,
        callee: &Expr,
        context: CallableContext,
    ) -> CompileError {
        let detail = match context {
            CallableContext::PlainCall | CallableContext::MultiCall { .. } => format!(
                "calling package variable `{name}` is not supported: {}",
                self.call_target_detail(callee, context)
            ),
            CallableContext::Go => format!(
                "`go` cannot call package variable `{name}`: {}",
                self.call_target_detail(callee, context)
            ),
            CallableContext::Defer => format!(
                "`defer` cannot call package variable `{name}`: {}",
                self.call_target_detail(callee, context)
            ),
        };
        self.unsupported_with_active_span(detail)
    }

    pub(super) fn unsupported_call_target_error(
        &self,
        callee: &Expr,
        context: CallableContext,
    ) -> CompileError {
        let prefix = match context {
            CallableContext::PlainCall => "unsupported call target",
            CallableContext::MultiCall { .. } => "unsupported multi-result call target",
            CallableContext::Go => {
                "`go` currently supports only function-value calls and concrete method calls"
            }
            CallableContext::Defer => "`defer` currently supports function and selector calls",
        };
        self.unsupported_with_active_span(format!(
            "{prefix}: {}",
            self.call_target_detail(callee, context)
        ))
    }

    pub(super) fn function_value_result_count_error(
        &self,
        callee: &Expr,
        actual: usize,
        expected: usize,
    ) -> CompileError {
        self.unsupported_with_active_span(format!(
            "call target {} returns {actual} value(s); expected {expected}",
            self.call_target_label(callee)
        ))
    }

    fn call_target_detail(&self, callee: &Expr, context: CallableContext) -> String {
        let expected = match context {
            CallableContext::PlainCall => {
                "a function value, named function, or supported builtin".to_string()
            }
            CallableContext::MultiCall { expected_results } => {
                format!("a function value returning {expected_results} value(s)")
            }
            CallableContext::Go => "a function value or concrete method call".to_string(),
            CallableContext::Defer => {
                "a function value, named function, or selector call".to_string()
            }
        };
        if let Some(actual) = self.infer_expr_type_name(callee) {
            format!(
                "call target {} has type `{actual}`; expected {expected}",
                self.call_target_label(callee)
            )
        } else {
            format!(
                "call target {} does not have a known callable type in the current subset; expected {expected}",
                self.call_target_label(callee)
            )
        }
    }

    fn call_target_label(&self, callee: &Expr) -> String {
        match callee {
            Expr::Ident(name) => format!("`{name}`"),
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    format!("`{receiver_name}.{field}`")
                } else {
                    format!("selector `{field}`")
                }
            }
            Expr::FunctionLiteral { .. } => "function literal".into(),
            Expr::Call { .. } => "call expression".into(),
            Expr::Index { .. } => "index expression".into(),
            Expr::TypeAssert { .. } => "type assertion".into(),
            Expr::Unary { .. } => "unary expression".into(),
            Expr::Binary { .. } => "binary expression".into(),
            Expr::NilLiteral => "`nil`".into(),
            Expr::BoolLiteral(_) => "bool literal".into(),
            Expr::IntLiteral(_) => "int literal".into(),
            Expr::FloatLiteral(_) => "float literal".into(),
            Expr::StringLiteral(_) => "string literal".into(),
            Expr::ArrayLiteral { .. } => "array literal".into(),
            Expr::SliceLiteral { .. } => "slice literal".into(),
            Expr::SliceConversion { .. } => "slice conversion".into(),
            Expr::MapLiteral { .. } => "map literal".into(),
            Expr::StructLiteral { .. } => "struct literal".into(),
            Expr::SliceExpr { .. } => "slice expression".into(),
            Expr::New { .. } => "`new(...)`".into(),
            Expr::Make { .. } => "`make(...)`".into(),
            Expr::Spread { .. } => "spread expression".into(),
        }
    }
}
