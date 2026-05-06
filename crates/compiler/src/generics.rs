use gowasm_parser::Expr;

use super::*;
use crate::program::compile_function_with_imports;
use crate::types::{
    build_substitutions, display_type, infer_type_args, substitute_type_params, validate_type_args,
    ConstraintInterface, InstanceKey, TypeConstraint, TypeParamDef,
};

type InferredGenericResult = Option<Result<(GenericCallOrigin, String, Vec<String>), CompileError>>;

#[derive(Debug, Clone)]
enum GenericCallOrigin {
    Local,
    Imported { package_path: String },
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedGenericCall {
    pub(super) name: String,
    pub(super) type_args: Vec<String>,
    pub(super) param_types: Vec<String>,
    pub(super) result_types: Vec<String>,
    origin: GenericCallOrigin,
}

impl ResolvedGenericCall {
    fn display_name(&self) -> String {
        let base_name = match &self.origin {
            GenericCallOrigin::Local => self.name.clone(),
            GenericCallOrigin::Imported { package_path } => {
                format!("{package_path}.{}", self.name)
            }
        };
        InstanceKey {
            base_name,
            type_args: self.type_args.clone(),
        }
        .mangled_name()
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn resolve_generic_call(
        &self,
        callee: &Expr,
        type_args: &[String],
        args: &[Expr],
    ) -> Option<Result<ResolvedGenericCall, CompileError>> {
        if let Some((origin, name, explicit_type_args)) =
            self.explicit_generic_call(callee, type_args)
        {
            return Some(self.resolve_generic_call_signature(origin, &name, explicit_type_args));
        }
        if let Some(inferred_generic_call) = self.inferred_generic_call(callee, args) {
            return Some(
                inferred_generic_call.and_then(|(origin, name, inferred_type_args)| {
                    self.resolve_generic_call_signature(origin, &name, inferred_type_args)
                }),
            );
        }
        None
    }

    pub(super) fn compile_generic_call(
        &mut self,
        resolved: &ResolvedGenericCall,
        args: &[Expr],
        dst: Option<usize>,
    ) -> Result<(), CompileError> {
        self.validate_generic_call_args(resolved, args)?;
        if dst.is_some() && resolved.result_types.len() != 1 {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "`{}` returns {} value(s) and cannot be used in single-value position",
                    resolved.display_name(),
                    resolved.result_types.len()
                ),
            });
        }
        let function = self.instantiate_generic_function(resolved)?;
        let registers = self.compile_param_typed_args(&resolved.param_types, args)?;
        self.emitter.code.push(Instruction::CallFunction {
            function,
            args: registers,
            dst,
        });
        Ok(())
    }

    pub(super) fn compile_generic_multi_call(
        &mut self,
        resolved: &ResolvedGenericCall,
        args: &[Expr],
        dsts: &[usize],
    ) -> Result<(), CompileError> {
        self.validate_generic_call_args(resolved, args)?;
        if resolved.result_types.len() != dsts.len() {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "`{}` returns {} value(s), not {}",
                    resolved.display_name(),
                    resolved.result_types.len(),
                    dsts.len()
                ),
            });
        }
        let function = self.instantiate_generic_function(resolved)?;
        let registers = self.compile_param_typed_args(&resolved.param_types, args)?;
        self.emitter.code.push(Instruction::CallFunctionMulti {
            function,
            args: registers,
            dsts: dsts.to_vec(),
        });
        Ok(())
    }

    fn resolve_generic_call_signature(
        &self,
        origin: GenericCallOrigin,
        name: &str,
        type_args: Vec<String>,
    ) -> Result<ResolvedGenericCall, CompileError> {
        let generic_function = match &origin {
            GenericCallOrigin::Local => self.env.generic_functions.get(name),
            GenericCallOrigin::Imported { package_path } => {
                self.imported_generic_function(package_path, name)
            }
        }
        .ok_or_else(|| CompileError::UnknownFunction {
            name: name.to_string(),
        })?;
        self.validate_type_args_in_context(&generic_function.type_params, &type_args)?;
        let substitutions = build_substitutions(&generic_function.type_params, &type_args);
        Ok(ResolvedGenericCall {
            name: name.to_string(),
            type_args,
            param_types: generic_function
                .param_types
                .iter()
                .map(|param_type| substitute_type_params(param_type, &substitutions))
                .collect(),
            result_types: generic_function
                .result_types
                .iter()
                .map(|result_type| substitute_type_params(result_type, &substitutions))
                .collect(),
            origin,
        })
    }

    fn validate_generic_call_args(
        &self,
        resolved: &ResolvedGenericCall,
        args: &[Expr],
    ) -> Result<(), CompileError> {
        if resolved.param_types.len() != args.len() {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "`{}` expects {} argument(s), found {}",
                    resolved.display_name(),
                    resolved.param_types.len(),
                    args.len()
                ),
            });
        }
        for (index, (expected, arg)) in resolved.param_types.iter().zip(args.iter()).enumerate() {
            if matches!(arg, Expr::NilLiteral) {
                self.validate_assignable_type(Some(expected), arg)
                    .map_err(|_| CompileError::Unsupported {
                        detail: format!(
                            "argument {} to `{}` has type `nil`, expected `{expected}`",
                            index + 1,
                            resolved.display_name()
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
            if self.types_assignable(expected, &actual) {
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
                    "argument {} to `{}` has type `{actual}`, expected `{expected}`",
                    index + 1,
                    resolved.display_name()
                ),
            });
        }
        Ok(())
    }

    pub(super) fn validate_type_args_in_context(
        &self,
        type_params: &[TypeParamDef],
        type_args: &[String],
    ) -> Result<(), CompileError> {
        let result = (|| {
            let interface_types = self.visible_interface_types();
            validate_type_args(type_params, type_args, &interface_types)?;
            for (type_param, type_arg) in type_params.iter().zip(type_args) {
                match &type_param.constraint {
                    TypeConstraint::Any => {}
                    TypeConstraint::Comparable => {
                        if !self.type_is_comparable(type_arg) {
                            return Err(CompileError::Unsupported {
                                detail: format!(
                                    "type argument `{type_arg}` does not satisfy `{}` (constraint `comparable`)",
                                    type_param.name
                                ),
                            });
                        }
                    }
                    TypeConstraint::Interface(name) => {
                        let interface_type =
                            self.instantiated_interface_type(name).ok_or_else(|| {
                                CompileError::Unsupported {
                                    detail: format!("unknown constraint interface `{name}`"),
                                }
                            })?;
                        if !self.type_satisfies_interface(name, type_arg, &interface_type) {
                            return Err(CompileError::Unsupported {
                                detail: format!(
                                    "type argument `{type_arg}` does not satisfy `{}` (constraint `{name}`)",
                                    type_param.name
                                ),
                            });
                        }
                    }
                    TypeConstraint::InterfaceLiteral(interface) => {
                        self.validate_inline_interface_constraint(type_arg, type_param, interface)?;
                    }
                }
            }
            Ok(())
        })();
        result.map_err(|error| self.annotate_compile_error(error))
    }

    fn validate_inline_interface_constraint(
        &self,
        type_arg: &str,
        type_param: &TypeParamDef,
        interface: &ConstraintInterface,
    ) -> Result<(), CompileError> {
        for embed in &interface.embeds {
            self.validate_type_arg_against_constraint(type_arg, type_param, embed)?;
        }
        for terms in &interface.type_sets {
            if !terms.iter().any(|term| term == type_arg) {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "type argument `{type_arg}` does not satisfy `{}` (constraint `{}`)",
                        type_param.name,
                        display_constraint(&TypeConstraint::InterfaceLiteral(interface.clone()))
                    ),
                });
            }
        }
        if !interface.methods.is_empty() {
            let interface_type = InterfaceTypeDef {
                type_id: gowasm_vm::TypeId(0),
                methods: interface.methods.clone(),
            };
            if !self.type_satisfies_interface("<inline constraint>", type_arg, &interface_type) {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "type argument `{type_arg}` does not satisfy `{}` (constraint `{}`)",
                        type_param.name,
                        display_constraint(&TypeConstraint::InterfaceLiteral(interface.clone()))
                    ),
                });
            }
        }
        Ok(())
    }

    fn validate_type_arg_against_constraint(
        &self,
        type_arg: &str,
        type_param: &TypeParamDef,
        constraint: &TypeConstraint,
    ) -> Result<(), CompileError> {
        match constraint {
            TypeConstraint::Any => Ok(()),
            TypeConstraint::Comparable => {
                if self.type_is_comparable(type_arg) {
                    Ok(())
                } else {
                    Err(CompileError::Unsupported {
                        detail: format!(
                            "type argument `{type_arg}` does not satisfy `{}` (constraint `comparable`)",
                            type_param.name
                        ),
                    })
                }
            }
            TypeConstraint::Interface(name) => {
                let interface_type = self.instantiated_interface_type(name).ok_or_else(|| {
                    CompileError::Unsupported {
                        detail: format!("unknown constraint interface `{name}`"),
                    }
                })?;
                if self.type_satisfies_interface(name, type_arg, &interface_type) {
                    Ok(())
                } else {
                    Err(CompileError::Unsupported {
                        detail: format!(
                            "type argument `{type_arg}` does not satisfy `{}` (constraint `{name}`)",
                            type_param.name
                        ),
                    })
                }
            }
            TypeConstraint::InterfaceLiteral(interface) => {
                self.validate_inline_interface_constraint(type_arg, type_param, interface)
            }
        }
    }

    fn instantiate_generic_function(
        &mut self,
        resolved: &ResolvedGenericCall,
    ) -> Result<usize, CompileError> {
        if let GenericCallOrigin::Imported { package_path } = &resolved.origin {
            return self.instantiate_imported_generic_function(package_path, resolved);
        }
        let template = self
            .env
            .generic_function_templates
            .get(&resolved.name)
            .cloned()
            .ok_or_else(|| CompileError::UnknownFunction {
                name: resolved.name.clone(),
            })?;
        let generic_function = self
            .env
            .generic_functions
            .get(&resolved.name)
            .cloned()
            .ok_or_else(|| CompileError::UnknownFunction {
                name: resolved.name.clone(),
            })?;
        let key = InstanceKey {
            base_name: self.namespaced_generic_instance_base_name(&resolved.name),
            type_args: resolved.type_args.clone(),
        };
        let (concrete_name, function, created) = self
            .generation
            .generated_functions
            .reserve_instance(key.clone(), self.generation.instantiation_cache);
        if !created {
            return Ok(function);
        }

        let substitutions = build_substitutions(&generic_function.type_params, &resolved.type_args);
        let concrete_decl = crate::generic_substitute::instantiate_function_decl(
            &template.decl,
            &substitutions,
            concrete_name,
        );

        let function_ids = self.env.function_ids;
        let function_result_types = self.env.function_result_types;
        let function_types = self.env.function_types;
        let variadic_functions = self.env.variadic_functions;
        let generic_functions = self.env.generic_functions;
        let generic_types = self.env.generic_types;
        let generic_function_templates = self.env.generic_function_templates;
        let generic_method_templates = self.env.generic_method_templates;
        let instantiation_cache = &mut *self.generation.instantiation_cache;
        let generated_functions = &mut *self.generation.generated_functions;
        let instantiated_generics = &mut *self.generation.instantiated_generics;
        let method_function_ids = self.env.method_function_ids;
        let promoted_method_bindings = self.env.promoted_method_bindings;
        let struct_types = self.env.struct_types;
        let pointer_types = self.env.pointer_types;
        let interface_types = self.env.interface_types;
        let alias_types = self.env.alias_types;
        let globals = self.env.globals;
        let method_sets = self.env.method_sets;
        let imported_package_tables = self.env.imported_package_tables;

        let compiled = compile_function_with_imports(
            template.imported_packages,
            &template.source_path,
            Some(&template.spans),
            &concrete_decl,
            imported_package_tables,
            function_ids,
            function_result_types,
            function_types,
            variadic_functions,
            generic_functions,
            generic_types,
            generic_function_templates,
            generic_method_templates,
            instantiation_cache,
            generated_functions,
            instantiated_generics,
            method_function_ids,
            promoted_method_bindings,
            struct_types,
            pointer_types,
            interface_types,
            alias_types,
            method_sets,
            globals,
        )?;
        self.generation.generated_functions.fill(function, compiled);
        Ok(function)
    }

    fn explicit_generic_call(
        &self,
        callee: &Expr,
        type_args: &[String],
    ) -> Option<(GenericCallOrigin, String, Vec<String>)> {
        if !type_args.is_empty() {
            match callee {
                Expr::Ident(name) => {
                    if self.lookup_local(name).is_some() || self.lookup_global(name).is_some() {
                        return None;
                    }
                    return self
                        .env
                        .generic_functions
                        .contains_key(name)
                        .then(|| (GenericCallOrigin::Local, name.clone(), type_args.to_vec()));
                }
                Expr::Selector { receiver, field } => {
                    let Expr::Ident(receiver_name) = receiver.as_ref() else {
                        return None;
                    };
                    let package_path = self.env.imported_packages.get(receiver_name)?;
                    return self
                        .imported_generic_function(package_path, field)
                        .is_some()
                        .then(|| {
                            (
                                GenericCallOrigin::Imported {
                                    package_path: package_path.clone(),
                                },
                                field.clone(),
                                type_args.to_vec(),
                            )
                        });
                }
                _ => return None,
            }
        }

        let Expr::Index { target, index } = callee else {
            return None;
        };
        match target.as_ref() {
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some() || self.lookup_global(name).is_some() {
                    return None;
                }
                if !self.env.generic_functions.contains_key(name) {
                    return None;
                }
                explicit_generic_index_type(index)
                    .map(|type_arg| (GenericCallOrigin::Local, name.clone(), vec![type_arg]))
            }
            Expr::Selector { receiver, field } => {
                let Expr::Ident(receiver_name) = receiver.as_ref() else {
                    return None;
                };
                let package_path = self.env.imported_packages.get(receiver_name)?;
                self.imported_generic_function(package_path, field)?;
                explicit_generic_index_type(index).map(|type_arg| {
                    (
                        GenericCallOrigin::Imported {
                            package_path: package_path.clone(),
                        },
                        field.clone(),
                        vec![type_arg],
                    )
                })
            }
            _ => None,
        }
    }

    fn inferred_generic_call(&self, callee: &Expr, args: &[Expr]) -> InferredGenericResult {
        let (origin, name, generic_function) = match callee {
            Expr::Ident(name) => {
                if self.lookup_local(name).is_some() || self.lookup_global(name).is_some() {
                    return None;
                }
                let generic_function = self.env.generic_functions.get(name)?;
                (GenericCallOrigin::Local, name.clone(), generic_function)
            }
            Expr::Selector { receiver, field } => {
                let Expr::Ident(receiver_name) = receiver.as_ref() else {
                    return None;
                };
                let package_path = self.env.imported_packages.get(receiver_name)?;
                let generic_function = self.imported_generic_function(package_path, field)?;
                (
                    GenericCallOrigin::Imported {
                        package_path: package_path.clone(),
                    },
                    field.clone(),
                    generic_function,
                )
            }
            _ => return None,
        };
        let arg_types = args
            .iter()
            .map(|arg| self.infer_expr_type_name(arg))
            .collect::<Option<Vec<_>>>();
        let interface_types = self.visible_interface_types();
        Some(
            arg_types
                .and_then(|arg_types| {
                    infer_type_args(generic_function, &arg_types, &interface_types)
                })
                .map(|type_args| (origin, name.clone(), type_args))
                .ok_or_else(|| CompileError::Unsupported {
                    detail: format!(
                        "could not infer type arguments for generic call `{name}(...)` in the current subset"
                    ),
                }),
        )
    }

    fn visible_interface_types(&self) -> HashMap<String, InterfaceTypeDef> {
        let mut interface_types = self
            .env
            .interface_types
            .iter()
            .map(|(name, interface_type)| (name.clone(), interface_type.clone()))
            .collect::<HashMap<_, _>>();
        interface_types.extend(
            self.env
                .imported_package_tables
                .interfaces
                .iter()
                .map(|(name, interface_type)| (name.clone(), interface_type.clone())),
        );
        interface_types.extend(
            self.generation
                .instantiated_generics
                .interface_types()
                .clone(),
        );
        interface_types
    }
}

fn display_constraint(constraint: &TypeConstraint) -> String {
    match constraint {
        TypeConstraint::Any => "interface{}".to_string(),
        TypeConstraint::Comparable => "comparable".to_string(),
        TypeConstraint::Interface(name) => name.clone(),
        TypeConstraint::InterfaceLiteral(interface) => {
            let mut clauses = Vec::new();
            clauses.extend(interface.methods.iter().map(render_constraint_method));
            clauses.extend(interface.embeds.iter().map(display_constraint));
            clauses.extend(interface.type_sets.iter().map(|terms| terms.join("|")));
            format!("interface{{{}}}", clauses.join(";"))
        }
    }
}

fn render_constraint_method(method: &InterfaceMethodDecl) -> String {
    let params = method
        .params
        .iter()
        .map(|param| {
            let prefix = if param.variadic { "..." } else { "" };
            format!("{} {}{}", param.name, prefix, param.typ)
        })
        .collect::<Vec<_>>()
        .join(",");
    match method.result_types.as_slice() {
        [] => format!("{}({params})", method.name),
        [result] => format!("{}({params}) {result}", method.name),
        results => format!("{}({params}) ({})", method.name, results.join(",")),
    }
}

fn explicit_generic_index_type(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(name) => Some(name.clone()),
        Expr::Selector { receiver, field } => {
            let Expr::Ident(receiver_name) = receiver.as_ref() else {
                return None;
            };
            Some(format!("{receiver_name}.{field}"))
        }
        _ => None,
    }
}
