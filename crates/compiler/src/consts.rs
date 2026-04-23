use super::*;
use crate::stdlib_function_values::{
    imported_stdlib_selector_function_type, imported_stdlib_selector_value_type,
};
use crate::typed_lowering::predeclared_conversion_target;
use crate::types::format_function_type;

pub(crate) fn rewrite_const_iota_expr(expr: &Expr, iota: usize) -> Expr {
    match expr {
        Expr::Ident(name) if name == "iota" => Expr::IntLiteral(iota as i64),
        Expr::Unary { op, expr } => Expr::Unary {
            op: *op,
            expr: Box::new(rewrite_const_iota_expr(expr, iota)),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(rewrite_const_iota_expr(left, iota)),
            op: *op,
            right: Box::new(rewrite_const_iota_expr(right, iota)),
        },
        _ => expr.clone(),
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn expr_function_type_name(&self, expr: &Expr) -> Option<String> {
        let function_type = self.infer_expr_type_name(expr)?;
        let function_type = self.instantiated_underlying_type_name(&function_type);
        parse_function_type(&function_type)?;
        Some(function_type)
    }

    pub(super) fn expr_function_result_types(&self, expr: &Expr) -> Option<Vec<String>> {
        let function_type = self.expr_function_type_name(expr)?;
        let (_, result_types) = parse_function_type(&function_type)?;
        Some(result_types)
    }

    pub(super) fn infer_expr_type_name(&self, expr: &Expr) -> Option<String> {
        if let Some(info) = self.try_eval_const_expr(expr) {
            return Some(info.visible_type_name());
        }
        match expr {
            Expr::Unary { op, expr } => {
                let inner = self.infer_expr_type_name(expr)?;
                match op {
                    UnaryOp::Not if inner == "bool" => Some("bool".into()),
                    UnaryOp::Negate if inner == "int" || inner == "float64" => Some(inner),
                    UnaryOp::BitNot if inner == "int" => Some("int".into()),
                    UnaryOp::AddressOf => Some(format!("*{inner}")),
                    UnaryOp::Deref => parse_pointer_type(&inner).map(str::to_string),
                    UnaryOp::Receive => parse_channel_type(&inner).and_then(|channel_type| {
                        channel_type
                            .direction
                            .accepts_recv()
                            .then(|| channel_type.element_type.to_string())
                    }),
                    _ => None,
                }
            }
            Expr::Binary { left, op, right } => {
                let left_type = self.infer_expr_type_name(left)?;
                let right_type = self.infer_expr_type_name(right)?;
                match op {
                    BinaryOp::Add
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::Add if left_type == "string" && right_type == "string" => {
                        Some("string".into())
                    }
                    BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::ShiftLeft
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::ShiftRight
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::BitOr
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::BitXor
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::BitAnd
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::BitClear
                        if left_type == right_type && self.int_like_const_type(&left_type) =>
                    {
                        Some(left_type)
                    }
                    BinaryOp::And | BinaryOp::Or if left_type == "bool" && right_type == "bool" => {
                        Some("bool".into())
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual if left_type == right_type => {
                        Some("bool".into())
                    }
                    BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                        if left_type == "int" && right_type == "int" =>
                    {
                        Some("bool".into())
                    }
                    BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                        if left_type == "string" && right_type == "string" =>
                    {
                        Some("bool".into())
                    }
                    _ => None,
                }
            }
            Expr::ArrayLiteral {
                len, element_type, ..
            } => Some(format!("[{len}]{element_type}")),
            Expr::SliceLiteral { element_type, .. } => Some(format!("[]{element_type}")),
            Expr::SliceConversion { element_type, .. } => Some(format!("[]{element_type}")),
            Expr::MapLiteral {
                key_type,
                value_type,
                ..
            } => Some(format!("map[{key_type}]{value_type}")),
            Expr::StructLiteral { type_name, .. } => Some(type_name.clone()),
            Expr::TypeAssert { asserted_type, .. } => Some(asserted_type.clone()),
            Expr::Make { type_name, .. } => Some(type_name.clone()),
            Expr::FunctionLiteral {
                params,
                result_types,
                ..
            } => Some(format_function_type(
                &params
                    .iter()
                    .map(|parameter| parameter.typ.clone())
                    .collect::<Vec<_>>(),
                result_types,
            )),
            Expr::Selector { receiver, field } => {
                if let Expr::Ident(receiver_name) = receiver.as_ref() {
                    if let Some(package_path) = self.env.imported_packages.get(receiver_name) {
                        return resolve_stdlib_constant(package_path, field)
                            .map(|constant| constant.typ.to_string())
                            .or_else(|| imported_stdlib_selector_value_type(package_path, field))
                            .or_else(|| imported_stdlib_selector_function_type(package_path, field))
                            .or_else(|| {
                                self.lookup_imported_global_type(package_path, field)
                                    .map(str::to_string)
                            })
                            .or_else(|| {
                                self.lookup_imported_function_type(package_path, field)
                                    .map(str::to_string)
                            });
                    }
                }
                if let Some(function_type) = self.selector_method_expression_type(receiver, field) {
                    return Some(function_type);
                }
                let mut receiver_type = self.infer_expr_type_name(receiver)?;
                if let Some(inner) = parse_pointer_type(&receiver_type) {
                    receiver_type = inner.to_string();
                }
                if let Some(field_resolution) = self
                    .resolve_field_selector(&receiver_type, field)
                    .ok()
                    .flatten()
                {
                    return Some(field_resolution.typ);
                }
                if let Some(interface_type) = self.instantiated_interface_type(&receiver_type) {
                    if let Some(method) = interface_type
                        .methods
                        .iter()
                        .find(|method| method.name == *field)
                    {
                        let param_types: Vec<String> = method
                            .params
                            .iter()
                            .map(|param| param.typ.clone())
                            .collect();
                        return Some(format_function_type(&param_types, &method.result_types));
                    }
                }
                if let Some(method) = self
                    .instantiated_method_set(&receiver_type)
                    .and_then(|methods| methods.into_iter().find(|m| m.name == *field))
                {
                    let param_types: Vec<String> =
                        method.params.iter().map(|p| p.typ.clone()).collect();
                    return Some(format_function_type(&param_types, &method.result_types));
                }
                if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
                    let param_types = stdlib_function_param_types(function)?
                        .iter()
                        .skip(1)
                        .map(|typ| (*typ).to_string())
                        .collect::<Vec<_>>();
                    let result_types = stdlib_function_result_types(function)?
                        .iter()
                        .map(|typ| (*typ).to_string())
                        .collect::<Vec<_>>();
                    return Some(format_function_type(&param_types, &result_types));
                }
                None
            }
            Expr::Index { target, .. } => {
                let target_type = self.infer_expr_type_name(target)?;
                if let Some((_, element_type)) = parse_array_type(&target_type) {
                    Some(element_type.to_string())
                } else if let Some(element_type) = target_type.strip_prefix("[]") {
                    Some(element_type.to_string())
                } else if let Some((_, value_type)) = parse_map_type(&target_type) {
                    Some(value_type.to_string())
                } else if target_type == "string" {
                    Some("int".into())
                } else {
                    None
                }
            }
            Expr::SliceExpr { target, .. } => self.infer_expr_type_name(target),
            Expr::Call {
                callee,
                type_args,
                args,
            } => {
                if let Some(generic_call) = self.resolve_generic_call(callee, type_args, args) {
                    let generic_call = generic_call.ok()?;
                    if generic_call.result_types.len() == 1 {
                        return Some(generic_call.result_types[0].clone());
                    }
                    return None;
                }
                match callee.as_ref() {
                    Expr::Ident(name) => {
                        if predeclared_conversion_target(name.as_str()).is_some() && args.len() == 1
                        {
                            return Some(name.clone());
                        }
                        if self.instantiated_alias_type(name.as_str()).is_some() {
                            return Some(name.clone());
                        }
                        self.env
                            .function_result_types
                            .get(name)
                            .and_then(|result_types| {
                                (result_types.len() == 1).then(|| result_types[0].clone())
                            })
                            .or_else(|| {
                                self.expr_function_result_types(callee)
                                    .and_then(|result_types| {
                                        (result_types.len() == 1).then(|| result_types[0].clone())
                                    })
                            })
                    }
                    Expr::Selector { receiver, field } => {
                        if let Expr::Ident(receiver_name) = receiver.as_ref() {
                            if let Some(alias_name) =
                                self.imported_selector_alias_name(receiver_name, field)
                            {
                                return Some(alias_name);
                            }
                            if let Some(package_path) =
                                self.env.imported_packages.get(receiver_name)
                            {
                                if let Some(function) = resolve_stdlib_function(package_path, field)
                                {
                                    if let Some(result_types) =
                                        stdlib_function_result_types(function)
                                    {
                                        if result_types.len() == 1 {
                                            return Some(result_types[0].to_string());
                                        }
                                    }
                                }
                                if let Some(result_types) =
                                    self.lookup_imported_function_result_types(package_path, field)
                                {
                                    if result_types.len() == 1 {
                                        return Some(result_types[0].clone());
                                    }
                                }
                            }
                        }
                        if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
                            if let Some(result_types) = stdlib_function_result_types(function) {
                                if result_types.len() == 1 {
                                    return Some(result_types[0].to_string());
                                }
                            }
                        }
                        let receiver_type = self.infer_expr_type_name(receiver)?;
                        if let Some(interface_type) =
                            self.instantiated_interface_type(&receiver_type)
                        {
                            interface_type
                                .methods
                                .iter()
                                .find(|method| method.name == *field)
                                .and_then(|method| {
                                    (method.result_types.len() == 1)
                                        .then(|| method.result_types[0].clone())
                                })
                        } else {
                            self.lookup_concrete_method(receiver, field)
                                .and_then(|method| {
                                    (method.result_types.len() == 1)
                                        .then(|| method.result_types[0].clone())
                                })
                        }
                    }
                    _ => self
                        .expr_function_result_types(callee)
                        .and_then(|result_types| {
                            (result_types.len() == 1).then(|| result_types[0].clone())
                        }),
                }
            }
            Expr::Ident(name) => self
                .lookup_local_type(name)
                .map(str::to_string)
                .or_else(|| self.lookup_global_type(name).map(str::to_string))
                .or_else(|| self.env.function_types.get(name).cloned()),
            Expr::New { type_name } => Some(format!("*{type_name}")),
            _ => None,
        }
    }

    pub(super) fn validate_const_initializer(
        &self,
        target_type: Option<&str>,
        value: &Expr,
    ) -> Result<(), CompileError> {
        self.validate_const_expr(value)?;
        if let Some(target_type) = target_type {
            if let Some(info) = self.try_eval_const_expr(value) {
                return self.validate_const_assignable_type(target_type, &info);
            }
        }
        self.validate_assignable_type(target_type, value)?;
        Ok(())
    }

    fn validate_const_expr(&self, expr: &Expr) -> Result<(), CompileError> {
        self.eval_const_expr(expr).map(|_| ())
    }
}
