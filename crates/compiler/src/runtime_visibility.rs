use super::*;
use crate::types::{split_generic_type_name, TypeParamDef};

impl FunctionBuilder<'_> {
    pub(super) fn ensure_runtime_visible_type(&mut self, typ: &str) -> Result<(), CompileError> {
        self.ensure_runtime_visible_type_in_context(typ, &[])
    }

    pub(super) fn ensure_runtime_visible_type_in_context(
        &mut self,
        typ: &str,
        type_params: &[TypeParamDef],
    ) -> Result<(), CompileError> {
        if typ.is_empty()
            || matches!(
                typ,
                "int"
                    | "byte"
                    | "rune"
                    | "float64"
                    | "string"
                    | "bool"
                    | "interface{}"
                    | "error"
                    | "any"
            )
        {
            return Ok(());
        }
        if type_params.iter().any(|type_param| type_param.name == typ) {
            return Ok(());
        }
        if self.instantiated_named_type(typ) || self.instantiated_pointer_type(typ).is_some() {
            return Ok(());
        }
        if let Some(inner) = parse_pointer_type(typ) {
            return self.ensure_runtime_visible_type_in_context(inner, type_params);
        }
        if let Some((_len, element_type)) = parse_array_type(typ) {
            return self.ensure_runtime_visible_type_in_context(element_type, type_params);
        }
        if let Some(element_type) = typ.strip_prefix("[]") {
            return self.ensure_runtime_visible_type_in_context(element_type, type_params);
        }
        if let Some((key_type, value_type)) = parse_map_type(typ) {
            self.ensure_runtime_visible_type_in_context(key_type, type_params)?;
            self.validate_map_key_type_comparable_in_context(key_type, type_params)?;
            return self.ensure_runtime_visible_type_in_context(value_type, type_params);
        }
        if let Some(channel_type) = parse_channel_type(typ) {
            return self
                .ensure_runtime_visible_type_in_context(channel_type.element_type, type_params);
        }
        if let Some((params, results)) = parse_function_type(typ) {
            for param in &params {
                self.ensure_runtime_visible_type_in_context(param, type_params)?;
            }
            for result in &results {
                self.ensure_runtime_visible_type_in_context(result, type_params)?;
            }
            return Ok(());
        }
        if let Some((base_name, type_args)) = split_generic_type_name(typ) {
            for type_arg in &type_args {
                self.ensure_runtime_visible_type_in_context(type_arg, type_params)?;
            }
            if type_args.iter().any(|type_arg| {
                type_params
                    .iter()
                    .any(|type_param| type_param.name == *type_arg)
            }) {
                return Ok(());
            }
            if self.instantiate_visible_generic_named_type(&base_name, &type_args)? {
                return Ok(());
            }
        }
        Ok(())
    }

    pub(super) fn validate_map_key_type_comparable_in_context(
        &self,
        key_type: &str,
        type_params: &[TypeParamDef],
    ) -> Result<(), CompileError> {
        if self.type_is_comparable_in_context(key_type, type_params) {
            return Ok(());
        }
        if type_params
            .iter()
            .any(|type_param| type_param.name == key_type)
        {
            return Err(self.unsupported_with_active_span(format!(
                "map key type parameter `{key_type}` must satisfy `comparable`"
            )));
        }
        Err(self
            .unsupported_with_active_span(format!("map key type `{key_type}` is not comparable")))
    }

    pub(super) fn ensure_expr_runtime_types(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Unary { expr, .. } => self.ensure_expr_runtime_types(expr)?,
            Expr::Binary { left, right, .. } => {
                self.ensure_expr_runtime_types(left)?;
                self.ensure_expr_runtime_types(right)?;
            }
            Expr::ArrayLiteral { elements, .. } | Expr::SliceLiteral { elements, .. } => {
                for element in elements {
                    self.ensure_expr_runtime_types(element)?;
                }
            }
            Expr::SliceConversion { expr, .. }
            | Expr::TypeAssert { expr, .. }
            | Expr::Spread { expr } => self.ensure_expr_runtime_types(expr)?,
            Expr::MapLiteral { entries, .. } => {
                for entry in entries {
                    self.ensure_expr_runtime_types(&entry.key)?;
                    self.ensure_expr_runtime_types(&entry.value)?;
                }
            }
            Expr::StructLiteral { type_name, fields } => {
                self.ensure_runtime_visible_type(type_name)?;
                for field in fields {
                    self.ensure_expr_runtime_types(&field.value)?;
                }
            }
            Expr::Index { target, index } => {
                self.ensure_expr_runtime_types(target)?;
                self.ensure_expr_runtime_types(index)?;
            }
            Expr::SliceExpr { target, low, high } => {
                self.ensure_expr_runtime_types(target)?;
                if let Some(low) = low {
                    self.ensure_expr_runtime_types(low)?;
                }
                if let Some(high) = high {
                    self.ensure_expr_runtime_types(high)?;
                }
            }
            Expr::Selector { receiver, .. } => {
                self.ensure_expr_runtime_types(receiver)?;
            }
            Expr::New { type_name } | Expr::Make { type_name, .. } => {
                self.ensure_runtime_visible_type(type_name)?;
            }
            Expr::FunctionLiteral {
                params,
                result_types,
                body: _,
            } => {
                for param in params {
                    self.ensure_runtime_visible_type(&param.typ)?;
                }
                for result in result_types {
                    self.ensure_runtime_visible_type(result)?;
                }
            }
            Expr::Call { callee, args, .. } => {
                self.ensure_expr_runtime_types(callee)?;
                for arg in args {
                    self.ensure_expr_runtime_types(arg)?;
                }
                if let Some(result_types) = self.infer_expr_result_types(expr) {
                    for result_type in &result_types {
                        self.ensure_runtime_visible_type(result_type)?;
                    }
                }
            }
            Expr::NilLiteral
            | Expr::BoolLiteral(_)
            | Expr::IntLiteral(_)
            | Expr::FloatLiteral(_)
            | Expr::StringLiteral(_)
            | Expr::Ident(_) => {}
        }

        if let Some(type_name) = self.infer_expr_type_name(expr) {
            self.ensure_runtime_visible_type(&type_name)?;
        }
        Ok(())
    }
}
