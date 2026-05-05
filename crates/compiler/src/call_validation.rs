use super::*;
use crate::types::{
    build_substitutions, display_type, split_generic_type_name, substitute_type_params,
    TypeConstraint, TypeParamDef,
};
use gowasm_parser::TypeDeclKind;

impl FunctionBuilder<'_> {
    pub(super) fn validate_imported_function_call_arity(
        &self,
        package: &str,
        symbol: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some((param_types, _)) = self
            .lookup_imported_function_type(package, symbol)
            .and_then(parse_function_type)
        else {
            return Ok(());
        };
        if self.imported_function_is_variadic(package, symbol) {
            let required = param_types.len().saturating_sub(1);
            if args.len() < required {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "`{package}.{symbol}` expects at least {required} argument(s), found {}",
                        args.len()
                    ),
                });
            }
            return Ok(());
        }
        self.validate_argument_count(
            &format!("`{package}.{symbol}`"),
            param_types.len(),
            args.len(),
        )
    }

    pub(super) fn validate_imported_function_call_types(
        &self,
        package: &str,
        symbol: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some((param_types, _)) = self
            .lookup_imported_function_type(package, symbol)
            .and_then(parse_function_type)
        else {
            return Ok(());
        };
        if self.imported_function_is_variadic(package, symbol) {
            let fixed_count = param_types.len().saturating_sub(1);
            let fixed_params = &param_types[..fixed_count];
            let fixed_args = &args[..fixed_count.min(args.len())];
            return self.validate_argument_types(
                &format!("`{package}.{symbol}`"),
                fixed_params,
                fixed_args,
            );
        }
        self.validate_argument_types(&format!("`{package}.{symbol}`"), &param_types, args)
    }

    pub(super) fn validate_named_function_call_arity(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some((param_types, _)) = self
            .env
            .function_types
            .get(name)
            .and_then(|function_type| parse_function_type(function_type))
        else {
            return Ok(());
        };
        if self.env.variadic_functions.contains(name) {
            let required = param_types.len().saturating_sub(1);
            if args.len() < required {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "`{name}` expects at least {required} argument(s), found {}",
                        args.len()
                    ),
                });
            }
            return Ok(());
        }
        self.validate_argument_count(&format!("`{name}`"), param_types.len(), args.len())
    }

    pub(super) fn validate_named_function_call_types(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some((param_types, _)) = self
            .env
            .function_types
            .get(name)
            .and_then(|function_type| parse_function_type(function_type))
        else {
            return Ok(());
        };
        if self.env.variadic_functions.contains(name) {
            let fixed_count = param_types.len().saturating_sub(1);
            let fixed_params = &param_types[..fixed_count];
            let fixed_args = &args[..fixed_count.min(args.len())];
            self.validate_argument_types(&format!("`{name}`"), fixed_params, fixed_args)?;
            return Ok(());
        }
        self.validate_argument_types(&format!("`{name}`"), &param_types, args)
    }

    pub(super) fn validate_selector_call_arity(
        &self,
        receiver: &Expr,
        field: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        if let Some((interface_name, interface_type)) = self.interface_type_for_expr(receiver) {
            let Some(method) = interface_type
                .methods
                .iter()
                .find(|method| method.name == *field)
            else {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "method `{field}` is not part of interface `{interface_name}` in the current subset"
                    ),
                });
            };
            return self.validate_argument_count(
                &format!("method `{field}` of interface `{interface_name}`"),
                method.params.len(),
                args.len(),
            );
        }

        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return Ok(());
        };
        if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
            let fixed_param_types = stdlib_function_param_types(function)
                .unwrap_or(&[])
                .iter()
                .skip(1)
                .map(|typ| (*typ).to_string())
                .collect::<Vec<_>>();
            if stdlib_function_variadic_param_type(function).is_some() {
                if args.len() < fixed_param_types.len() {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "method `{receiver_type}.{field}` expects at least {} argument(s), found {}",
                            fixed_param_types.len(),
                            args.len()
                        ),
                    });
                }
                return Ok(());
            }
            return self.validate_argument_count(
                &format!("method `{receiver_type}.{field}`"),
                fixed_param_types.len(),
                args.len(),
            );
        }
        if let Some(detail) = self.ambiguous_method_selector_detail(receiver, field) {
            return Err(CompileError::Unsupported { detail });
        }
        let Some(method) = self.lookup_concrete_method(receiver, field) else {
            return Ok(());
        };
        self.validate_argument_count(
            &format!("method `{receiver_type}.{field}`"),
            method.params.len(),
            args.len(),
        )
    }

    pub(super) fn validate_selector_call_types(
        &self,
        receiver: &Expr,
        field: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        if let Some((interface_name, interface_type)) = self.interface_type_for_expr(receiver) {
            let Some(method) = interface_type
                .methods
                .iter()
                .find(|method| method.name == *field)
            else {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "method `{field}` is not part of interface `{interface_name}` in the current subset"
                    ),
                });
            };
            let param_types = method
                .params
                .iter()
                .map(|param| param.typ.clone())
                .collect::<Vec<_>>();
            return self.validate_argument_types(
                &format!("method `{field}` of interface `{interface_name}`"),
                &param_types,
                args,
            );
        }

        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return Ok(());
        };
        if let Some(function) = self.lookup_stdlib_value_method(receiver, field) {
            let mut param_types = stdlib_function_param_types(function)
                .unwrap_or(&[])
                .iter()
                .skip(1)
                .map(|typ| (*typ).to_string())
                .collect::<Vec<_>>();
            if let Some(variadic_param_type) = stdlib_function_variadic_param_type(function) {
                while param_types.len() < args.len() {
                    param_types.push(variadic_param_type.to_string());
                }
            }
            return self.validate_argument_types(
                &format!("method `{receiver_type}.{field}`"),
                &param_types,
                args,
            );
        }
        if let Some(detail) = self.ambiguous_method_selector_detail(receiver, field) {
            return Err(CompileError::Unsupported { detail });
        }
        let Some(method) = self.lookup_concrete_method(receiver, field) else {
            return Ok(());
        };
        let param_types = method
            .params
            .iter()
            .map(|param| param.typ.clone())
            .collect::<Vec<_>>();
        self.validate_argument_types(
            &format!("method `{receiver_type}.{field}`"),
            &param_types,
            args,
        )
    }

    pub(super) fn validate_function_value_call_arity(
        &self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some(function_type) = self.expr_function_type_name(callee) else {
            return Ok(());
        };
        let Some((param_types, _)) = parse_function_type(&function_type) else {
            return Ok(());
        };
        self.validate_argument_count("function value", param_types.len(), args.len())
    }

    pub(super) fn validate_function_value_call_types(
        &self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some(function_type) = self.expr_function_type_name(callee) else {
            return Ok(());
        };
        let Some((param_types, _)) = parse_function_type(&function_type) else {
            return Ok(());
        };
        self.validate_argument_types("function value", &param_types, args)
    }

    pub(super) fn validate_builtin_call_arity(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        match name {
            "len" | "cap" => self.validate_argument_count(&format!("`{name}`"), 1, args.len()),
            "close" => self.validate_argument_count("`close`", 1, args.len()),
            "append" if args.is_empty() => Err(CompileError::Unsupported {
                detail: "`append` expects at least 1 argument in the current subset".into(),
            }),
            "append" => Ok(()),
            "min" | "max" if args.is_empty() => Err(CompileError::Unsupported {
                detail: format!("`{name}` expects at least 1 argument"),
            }),
            "min" | "max" => Ok(()),
            "clear" => self.validate_argument_count("`clear`", 1, args.len()),
            _ => Ok(()),
        }
    }

    pub(super) fn validate_builtin_call_types(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        match name {
            "len" => self.validate_len_target(args),
            "cap" => self.validate_cap_target(args),
            "append" => self.validate_append_args(args),
            "close" => self.validate_close_target(args),
            _ => Ok(()),
        }
    }

    pub(super) fn validate_stdlib_call_signature(
        &self,
        package: &str,
        symbol: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        let Some(function) = resolve_stdlib_function(package, symbol) else {
            return Ok(());
        };
        let param_types = stdlib_function_param_types(function);
        let variadic_param_type = stdlib_function_variadic_param_type(function);
        if param_types.is_none() && variadic_param_type.is_none() {
            return Ok(());
        }

        let callee = format!("`{package}.{symbol}`");
        let fixed_param_types = param_types.unwrap_or(&[]);

        if let Some(variadic_param_type) = variadic_param_type {
            if args.len() < fixed_param_types.len() {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "{callee} expects at least {} argument(s), found {}",
                        fixed_param_types.len(),
                        args.len()
                    ),
                });
            }
            let mut param_types = fixed_param_types
                .iter()
                .map(|typ| (*typ).to_string())
                .collect::<Vec<_>>();
            while param_types.len() < args.len() {
                param_types.push(variadic_param_type.to_string());
            }
            self.validate_argument_types(&callee, &param_types, args)?;
            return self.validate_stdlib_call_semantics(package, symbol, args);
        }

        self.validate_argument_count(&callee, fixed_param_types.len(), args.len())?;
        let param_types = fixed_param_types
            .iter()
            .map(|typ| (*typ).to_string())
            .collect::<Vec<_>>();
        self.validate_argument_types(&callee, &param_types, args)?;
        self.validate_stdlib_call_semantics(package, symbol, args)
    }

    fn validate_argument_count(
        &self,
        callee: &str,
        expected: usize,
        actual: usize,
    ) -> Result<(), CompileError> {
        if expected == actual {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!("{callee} expects {expected} argument(s), found {actual}"),
        })
    }

    fn validate_argument_types(
        &self,
        callee: &str,
        param_types: &[String],
        args: &[Expr],
    ) -> Result<(), CompileError> {
        for (index, (expected, arg)) in param_types.iter().zip(args.iter()).enumerate() {
            if matches!(arg, Expr::NilLiteral) {
                self.validate_assignable_type(Some(expected), arg)
                    .map_err(|_| CompileError::Unsupported {
                        detail: format!(
                            "argument {} to {callee} has type `nil`, expected `{expected}`",
                            index + 1
                        ),
                    })?;
                continue;
            }

            if self.literal_assignable_to(expected, arg) {
                continue;
            }

            let Some(actual) = self.infer_expr_type_name(arg) else {
                continue;
            };
            if self.argument_type_matches(expected, &actual) {
                continue;
            }
            if parse_function_type(expected).is_some() || parse_function_type(&actual).is_some() {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "function value of type `{}` is not assignable to `{}` in the current subset",
                        display_type(&actual),
                        display_type(expected)
                    ),
                });
            }
            return Err(CompileError::Unsupported {
                detail: format!(
                    "argument {} to {callee} has type `{actual}`, expected `{expected}`",
                    index + 1
                ),
            });
        }
        Ok(())
    }

    fn argument_type_matches(&self, expected: &str, actual: &str) -> bool {
        if expected == "int" && actual == "int64" {
            return true;
        }
        self.types_assignable(expected, actual)
    }

    fn validate_len_target(&self, args: &[Expr]) -> Result<(), CompileError> {
        let Some(actual) = args.first().and_then(|arg| self.infer_expr_type_name(arg)) else {
            return Ok(());
        };
        let underlying = self.instantiated_underlying_type_name(&actual);
        if underlying == "string"
            || parse_array_type(&underlying).is_some()
            || underlying.starts_with("[]")
            || parse_map_type(&underlying).is_some()
            || parse_channel_type(&underlying).is_some()
        {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "argument 1 to `len` has type `{actual}`, expected string, array, slice, map, or channel"
            ),
        })
    }

    fn validate_cap_target(&self, args: &[Expr]) -> Result<(), CompileError> {
        let Some(actual) = args.first().and_then(|arg| self.infer_expr_type_name(arg)) else {
            return Ok(());
        };
        let underlying = self.instantiated_underlying_type_name(&actual);
        if parse_array_type(&underlying).is_some()
            || underlying.starts_with("[]")
            || parse_channel_type(&underlying).is_some()
        {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "argument 1 to `cap` has type `{actual}`, expected array, slice, or channel"
            ),
        })
    }

    fn validate_close_target(&self, args: &[Expr]) -> Result<(), CompileError> {
        let Some(actual) = args.first().and_then(|arg| self.infer_expr_type_name(arg)) else {
            return Ok(());
        };
        let underlying = self.instantiated_underlying_type_name(&actual);
        let Some(channel) = parse_channel_type(&underlying) else {
            return Err(CompileError::Unsupported {
                detail: format!("argument 1 to `close` has type `{actual}`, expected channel"),
            });
        };
        if channel.direction.accepts_send() {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "argument 1 to `close` has type `{actual}`, expected bidirectional or send-only channel"
            ),
        })
    }

    fn validate_append_args(&self, args: &[Expr]) -> Result<(), CompileError> {
        let Some(target_type) = args.first().and_then(|arg| self.infer_expr_type_name(arg)) else {
            return Ok(());
        };
        let underlying = self.instantiated_underlying_type_name(&target_type);
        let Some(element_type) = underlying.strip_prefix("[]") else {
            return Err(CompileError::Unsupported {
                detail: format!("argument 1 to `append` has type `{target_type}`, expected slice"),
            });
        };

        for (index, arg) in args.iter().enumerate().skip(1) {
            if matches!(arg, Expr::NilLiteral) {
                self.validate_assignable_type(Some(element_type), arg)
                    .map_err(|_| CompileError::Unsupported {
                        detail: format!(
                            "argument {} to `append` has type `nil`, expected `{element_type}`",
                            index + 1
                        ),
                    })?;
                continue;
            }
            let Some(actual) = self.infer_expr_type_name(arg) else {
                continue;
            };
            if self.argument_type_matches(element_type, &actual) {
                continue;
            }
            return Err(CompileError::Unsupported {
                detail: format!(
                    "argument {} to `append` has type `{actual}`, expected `{element_type}`",
                    index + 1
                ),
            });
        }
        Ok(())
    }

    fn validate_stdlib_call_semantics(
        &self,
        package: &str,
        symbol: &str,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        match (package, symbol) {
            ("context", "WithValue") => self.validate_context_with_value_key(args),
            _ => Ok(()),
        }
    }

    fn validate_context_with_value_key(&self, args: &[Expr]) -> Result<(), CompileError> {
        let Some(key) = args.get(1) else {
            return Ok(());
        };
        if matches!(key, Expr::NilLiteral) {
            return Err(CompileError::Unsupported {
                detail:
                    "argument 2 to `context.WithValue` cannot be `nil`; context keys must be non-nil and comparable"
                        .into(),
            });
        }
        let Some(key_type) = self.infer_expr_type_name(key) else {
            return Ok(());
        };
        if self.type_is_comparable(&key_type) {
            return Ok(());
        }
        Err(CompileError::Unsupported {
            detail: format!(
                "argument 2 to `context.WithValue` has non-comparable type `{key_type}`"
            ),
        })
    }

    pub(super) fn type_is_comparable(&self, typ: &str) -> bool {
        self.type_is_comparable_in_context(typ, &[])
    }

    pub(super) fn type_is_comparable_in_context(
        &self,
        typ: &str,
        type_params: &[TypeParamDef],
    ) -> bool {
        if let Some(constraint) = type_params
            .iter()
            .find(|type_param| type_param.name == typ)
            .map(|type_param| &type_param.constraint)
        {
            return matches!(constraint, TypeConstraint::Comparable);
        }

        if typ == "interface{}"
            || typ == "any"
            || typ == "error"
            || self.instantiated_interface_type(typ).is_some()
        {
            return true;
        }

        let underlying = self.instantiated_underlying_type_name(typ);
        match underlying.as_str() {
            "int" | "float64" | "string" | "bool" => true,
            _ if parse_pointer_type(&underlying).is_some() => true,
            _ if parse_channel_type(&underlying).is_some() => true,
            _ if parse_map_type(&underlying).is_some() => false,
            _ if underlying.starts_with("[]") => false,
            _ if parse_function_type(&underlying).is_some() => false,
            _ => {
                if let Some((base_name, type_args)) = split_generic_type_name(&underlying) {
                    if let Some(generic_type) = self.env.generic_types.get(&base_name) {
                        if generic_type.type_params.len() != type_args.len() {
                            return false;
                        }
                        let substitutions =
                            build_substitutions(&generic_type.type_params, &type_args);
                        return match &generic_type.kind {
                            TypeDeclKind::Struct { fields } => fields.iter().all(|field| {
                                self.type_is_comparable_in_context(
                                    &substitute_type_params(&field.typ, &substitutions),
                                    type_params,
                                )
                            }),
                            TypeDeclKind::Alias { underlying } => self
                                .type_is_comparable_in_context(
                                    &substitute_type_params(underlying, &substitutions),
                                    type_params,
                                ),
                            TypeDeclKind::Interface { .. } => true,
                        };
                    }
                }
                if let Some((_, element_type)) = parse_array_type(&underlying) {
                    return self.type_is_comparable_in_context(element_type, type_params);
                }
                if let Some(struct_type) = self.instantiated_struct_type(&underlying) {
                    return struct_type
                        .fields
                        .iter()
                        .all(|field| self.type_is_comparable_in_context(&field.typ, type_params));
                }
                false
            }
        }
    }
}
