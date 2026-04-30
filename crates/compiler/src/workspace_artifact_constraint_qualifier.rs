use super::*;
use crate::types::ConstraintInterface;

pub(crate) fn qualify_type_constraint(
    constraint: &TypeConstraint,
    package_selector: &str,
    local_named_types: &HashSet<String>,
) -> TypeConstraint {
    match constraint {
        TypeConstraint::Any => TypeConstraint::Any,
        TypeConstraint::Comparable => TypeConstraint::Comparable,
        TypeConstraint::Interface(name) => {
            TypeConstraint::Interface(crate::workspace_artifact_exports::qualify_visible_type(
                name,
                package_selector,
                local_named_types,
            ))
        }
        TypeConstraint::InterfaceLiteral(interface) => {
            TypeConstraint::InterfaceLiteral(ConstraintInterface {
                methods: interface
                    .methods
                    .iter()
                    .map(|method| InterfaceMethodDecl {
                        name: method.name.clone(),
                        params: method
                            .params
                            .iter()
                            .map(|param| gowasm_parser::Parameter {
                                name: param.name.clone(),
                                typ: crate::workspace_artifact_exports::qualify_visible_type(
                                    &param.typ,
                                    package_selector,
                                    local_named_types,
                                ),
                                variadic: param.variadic,
                            })
                            .collect(),
                        result_types: method
                            .result_types
                            .iter()
                            .map(|typ| {
                                crate::workspace_artifact_exports::qualify_visible_type(
                                    typ,
                                    package_selector,
                                    local_named_types,
                                )
                            })
                            .collect(),
                    })
                    .collect(),
                embeds: interface
                    .embeds
                    .iter()
                    .map(|embed| {
                        qualify_type_constraint(embed, package_selector, local_named_types)
                    })
                    .collect(),
                type_sets: interface
                    .type_sets
                    .iter()
                    .map(|terms| {
                        terms
                            .iter()
                            .map(|term| {
                                crate::workspace_artifact_exports::qualify_visible_type(
                                    term,
                                    package_selector,
                                    local_named_types,
                                )
                            })
                            .collect()
                    })
                    .collect(),
            })
        }
    }
}
