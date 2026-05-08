use std::collections::HashMap;

use super::{
    channel_direction_matches, parse_type_key, ConstraintInterface, GenericFunctionDef,
    InterfaceTypeDef, TypeConstraint, TypeKey, TypeParamDef,
};
use crate::CompileError;

pub(crate) fn lower_type_param(param: &gowasm_parser::TypeParam) -> TypeParamDef {
    let constraint = lower_constraint_text(&param.constraint);
    TypeParamDef {
        name: param.name.clone(),
        constraint,
    }
}

pub(crate) fn lower_constraint_text(constraint: &str) -> TypeConstraint {
    let constraint =
        gowasm_parser::parse_type_constraint(constraint).expect("constraint text should parse");
    lower_constraint_repr(&constraint)
}

fn lower_constraint_repr(constraint: &gowasm_parser::TypeConstraintRepr) -> TypeConstraint {
    match constraint {
        gowasm_parser::TypeConstraintRepr::Any => TypeConstraint::Any,
        gowasm_parser::TypeConstraintRepr::Comparable => TypeConstraint::Comparable,
        gowasm_parser::TypeConstraintRepr::Named(name) => TypeConstraint::Interface(name.clone()),
        gowasm_parser::TypeConstraintRepr::Interface(interface) => {
            TypeConstraint::InterfaceLiteral(ConstraintInterface {
                methods: interface.methods.clone(),
                embeds: interface
                    .embeds
                    .iter()
                    .map(|embed| lower_constraint_text(embed))
                    .collect(),
                type_sets: interface.type_sets.clone(),
            })
        }
    }
}

pub(crate) fn substitute_type_params(typ: &str, substitutions: &HashMap<String, String>) -> String {
    if let Some(concrete) = substitutions.get(typ) {
        return concrete.clone();
    }
    if let Some(type_key) = parse_type_key(typ) {
        return substitute_type_key(&type_key, substitutions).render();
    }
    typ.to_string()
}

pub(crate) fn infer_type_args(
    generic_function: &GenericFunctionDef,
    arg_types: &[String],
    _interface_types: &HashMap<String, InterfaceTypeDef>,
) -> Option<Vec<String>> {
    if generic_function.param_types.len() != arg_types.len() {
        return None;
    }

    let mut inferred = HashMap::new();
    for (param_type, arg_type) in generic_function.param_types.iter().zip(arg_types) {
        infer_from_pattern(
            param_type,
            arg_type,
            &generic_function.type_params,
            &mut inferred,
        )?;
    }

    let type_args: Vec<String> = generic_function
        .type_params
        .iter()
        .map(|type_param| {
            let concrete = inferred.get(&type_param.name)?.clone();
            Some(concrete)
        })
        .collect::<Option<_>>()?;
    let substitutions = build_substitutions(&generic_function.type_params, &type_args);
    (substitutions.len() == generic_function.type_params.len()).then_some(type_args)
}

pub(crate) fn build_substitutions(
    type_params: &[TypeParamDef],
    type_args: &[String],
) -> HashMap<String, String> {
    type_params
        .iter()
        .zip(type_args.iter())
        .map(|(type_param, type_arg)| (type_param.name.clone(), type_arg.clone()))
        .collect()
}

pub(crate) fn validate_type_args(
    type_params: &[TypeParamDef],
    type_args: &[String],
    interface_types: &HashMap<String, InterfaceTypeDef>,
) -> Result<(), CompileError> {
    if type_params.len() != type_args.len() {
        return Err(CompileError::Unsupported {
            detail: format!(
                "expected {} type argument(s), found {}",
                type_params.len(),
                type_args.len()
            ),
        });
    }
    for (type_param, type_arg) in type_params.iter().zip(type_args.iter()) {
        check_type_constraint(&type_param.constraint, type_arg, interface_types).map_err(
            |err| match err {
                CompileError::Unsupported { detail } => CompileError::Unsupported {
                    detail: format!(
                        "type argument `{type_arg}` does not satisfy `{}`: {detail}",
                        type_param.name
                    ),
                },
                other => other,
            },
        )?;
    }
    Ok(())
}

pub(crate) fn check_type_constraint(
    constraint: &TypeConstraint,
    concrete_type: &str,
    interface_types: &HashMap<String, InterfaceTypeDef>,
) -> Result<(), CompileError> {
    match constraint {
        TypeConstraint::Any => Ok(()),
        TypeConstraint::Comparable => match concrete_type {
            "int" | "float64" | "string" | "bool" | "byte" | "rune" => Ok(()),
            _ if interface_types.contains_key(concrete_type) => Err(CompileError::Unsupported {
                detail: format!("interface type `{concrete_type}` does not satisfy `comparable`"),
            }),
            _ => Ok(()),
        },
        TypeConstraint::Interface(name) => {
            if interface_types.contains_key(name) {
                Ok(())
            } else {
                Err(CompileError::Unsupported {
                    detail: format!("unknown constraint interface `{name}`"),
                })
            }
        }
        TypeConstraint::InterfaceLiteral(interface) => {
            for embed in &interface.embeds {
                check_type_constraint(embed, concrete_type, interface_types)?;
            }
            for terms in &interface.type_sets {
                if !terms.iter().any(|term| term == concrete_type) {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "type `{concrete_type}` is not in constraint type set `{}`",
                            terms.join(" | ")
                        ),
                    });
                }
            }
            Ok(())
        }
    }
}

fn infer_from_pattern(
    pattern: &str,
    actual: &str,
    type_params: &[TypeParamDef],
    inferred: &mut HashMap<String, String>,
) -> Option<()> {
    let pattern = parse_type_key(pattern)?;
    let actual = parse_type_key(actual)?;
    infer_from_type_key(&pattern, &actual, type_params, inferred)
}

fn substitute_type_key(typ: &TypeKey, substitutions: &HashMap<String, String>) -> TypeKey {
    match typ {
        TypeKey::Name(name) => substitutions
            .get(name)
            .and_then(|concrete| parse_type_key(concrete))
            .unwrap_or_else(|| TypeKey::Name(name.clone())),
        TypeKey::Pointer(inner) => {
            TypeKey::Pointer(Box::new(substitute_type_key(inner, substitutions)))
        }
        TypeKey::Slice(inner) => {
            TypeKey::Slice(Box::new(substitute_type_key(inner, substitutions)))
        }
        TypeKey::Array { len, element } => TypeKey::Array {
            len: *len,
            element: Box::new(substitute_type_key(element, substitutions)),
        },
        TypeKey::Map { key, value } => TypeKey::Map {
            key: Box::new(substitute_type_key(key, substitutions)),
            value: Box::new(substitute_type_key(value, substitutions)),
        },
        TypeKey::Channel { direction, element } => TypeKey::Channel {
            direction: *direction,
            element: Box::new(substitute_type_key(element, substitutions)),
        },
        TypeKey::Function { params, results } => TypeKey::Function {
            params: params
                .iter()
                .map(|param| substitute_type_key(param, substitutions))
                .collect(),
            results: results
                .iter()
                .map(|result| substitute_type_key(result, substitutions))
                .collect(),
        },
        TypeKey::Interface => TypeKey::Interface,
        TypeKey::GenericInstance { base, type_args } => TypeKey::GenericInstance {
            base: base.clone(),
            type_args: type_args
                .iter()
                .map(|type_arg| substitute_type_key(type_arg, substitutions))
                .collect(),
        },
    }
}

fn infer_from_type_key(
    pattern: &TypeKey,
    actual: &TypeKey,
    type_params: &[TypeParamDef],
    inferred: &mut HashMap<String, String>,
) -> Option<()> {
    if let TypeKey::Name(name) = pattern {
        if type_params
            .iter()
            .any(|type_param| type_param.name == *name)
        {
            let actual_rendered = actual.render();
            match inferred.get(name) {
                Some(existing) if existing != &actual_rendered => None,
                Some(_) => Some(()),
                None => {
                    inferred.insert(name.clone(), actual_rendered);
                    Some(())
                }
            }?;
            return Some(());
        }
    }

    match (pattern, actual) {
        (TypeKey::Name(pattern), TypeKey::Name(actual)) => (pattern == actual).then_some(()),
        (TypeKey::Slice(pattern), TypeKey::Slice(actual)) => {
            infer_from_type_key(pattern, actual, type_params, inferred)
        }
        (
            TypeKey::Array {
                len: pattern_len,
                element: pattern_inner,
            },
            TypeKey::Array {
                len: actual_len,
                element: actual_inner,
            },
        ) if pattern_len == actual_len => {
            infer_from_type_key(pattern_inner, actual_inner, type_params, inferred)
        }
        (TypeKey::Pointer(pattern), TypeKey::Pointer(actual)) => {
            infer_from_type_key(pattern, actual, type_params, inferred)
        }
        (
            TypeKey::Map {
                key: pattern_key,
                value: pattern_value,
            },
            TypeKey::Map {
                key: actual_key,
                value: actual_value,
            },
        ) => {
            infer_from_type_key(pattern_key, actual_key, type_params, inferred)?;
            infer_from_type_key(pattern_value, actual_value, type_params, inferred)
        }
        (
            TypeKey::Channel {
                direction: pattern_direction,
                element: pattern_element,
            },
            TypeKey::Channel {
                direction: actual_direction,
                element: actual_element,
            },
        ) if channel_direction_matches(*pattern_direction, *actual_direction) => {
            infer_from_type_key(pattern_element, actual_element, type_params, inferred)
        }
        (
            TypeKey::Function {
                params: pattern_params,
                results: pattern_results,
            },
            TypeKey::Function {
                params: actual_params,
                results: actual_results,
            },
        ) if pattern_params.len() == actual_params.len()
            && pattern_results.len() == actual_results.len() =>
        {
            for (pattern_param, actual_param) in pattern_params.iter().zip(actual_params.iter()) {
                infer_from_type_key(pattern_param, actual_param, type_params, inferred)?;
            }
            for (pattern_result, actual_result) in pattern_results.iter().zip(actual_results.iter())
            {
                infer_from_type_key(pattern_result, actual_result, type_params, inferred)?;
            }
            Some(())
        }
        (
            TypeKey::GenericInstance {
                base: pattern_base,
                type_args: pattern_args,
            },
            TypeKey::GenericInstance {
                base: actual_base,
                type_args: actual_args,
            },
        ) if pattern_base == actual_base && pattern_args.len() == actual_args.len() => {
            for (pattern_arg, actual_arg) in pattern_args.iter().zip(actual_args.iter()) {
                infer_from_type_key(pattern_arg, actual_arg, type_params, inferred)?;
            }
            Some(())
        }
        (TypeKey::Interface, TypeKey::Interface) => Some(()),
        _ => None,
    }
}
