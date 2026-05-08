use std::collections::HashMap;

use gowasm_vm::{InterfaceMethodCheck, TypeCheck};

use super::{CompileError, InterfaceTypeDef, StructTypeDef};
use crate::FunctionBuilder;

fn lower_interface_method_check(method: &super::InterfaceMethodDecl) -> InterfaceMethodCheck {
    InterfaceMethodCheck {
        name: method.name.clone(),
        param_types: method
            .params
            .iter()
            .map(|param| param.typ.clone())
            .collect(),
        result_types: method.result_types.clone(),
    }
}

pub(crate) fn lower_type_assert_target(
    typ: &str,
    struct_types: &HashMap<String, StructTypeDef>,
    interface_types: &HashMap<String, InterfaceTypeDef>,
) -> Result<TypeCheck, CompileError> {
    match typ {
        "int" | "byte" | "rune" => Ok(TypeCheck::Int),
        "float64" => Ok(TypeCheck::Float64),
        "string" => Ok(TypeCheck::String),
        "bool" => Ok(TypeCheck::Bool),
        other => {
            if other == "error" {
                return Ok(TypeCheck::Interface {
                    name: "error".to_string(),
                    methods: vec![InterfaceMethodCheck {
                        name: "Error".to_string(),
                        param_types: Vec::new(),
                        result_types: vec!["string".to_string()],
                    }],
                });
            }
            if let Some(interface_type) = interface_types.get(other) {
                return Ok(TypeCheck::Interface {
                    name: other.to_string(),
                    methods: interface_type
                        .methods
                        .iter()
                        .map(lower_interface_method_check)
                        .collect(),
                });
            }
            struct_types
                .get(other)
                .map(|struct_type| TypeCheck::Struct {
                    type_id: struct_type.type_id,
                    name: other.to_string(),
                })
                .ok_or_else(|| CompileError::Unsupported {
                    detail: format!(
                        "type assertions currently support int, float64, byte, rune, string, bool, error, named struct targets, named pointer targets, and named interface targets, found `{other}`"
                    ),
                })
        }
    }
}

impl FunctionBuilder<'_> {
    fn lower_exact_type_assert_target(&self, typ: &str) -> Option<TypeCheck> {
        self.instantiated_pointer_type(typ)
            .map(|type_id| TypeCheck::Exact {
                type_id,
                name: typ.to_string(),
            })
            .or_else(|| {
                self.instantiated_alias_type(typ)
                    .map(|alias_type| TypeCheck::Exact {
                        type_id: alias_type.type_id,
                        name: typ.to_string(),
                    })
            })
            .or_else(|| {
                self.instantiated_struct_type(typ)
                    .map(|struct_type| TypeCheck::Exact {
                        type_id: struct_type.type_id,
                        name: typ.to_string(),
                    })
            })
    }

    pub(crate) fn lower_type_assert_target(&self, typ: &str) -> Result<TypeCheck, CompileError> {
        match typ {
            "int" | "byte" | "rune" | "float64" | "string" | "bool" | "error" => {
                lower_type_assert_target(typ, self.env.struct_types, self.env.interface_types)
            }
            other => {
                if let Some(interface_type) = self.instantiated_interface_type(other) {
                    return Ok(TypeCheck::Interface {
                        name: other.to_string(),
                        methods: interface_type
                            .methods
                            .iter()
                            .map(lower_interface_method_check)
                            .collect(),
                    });
                }
                if let Some(target) = self.lower_exact_type_assert_target(other) {
                    return Ok(target);
                }
                lower_type_assert_target(other, self.env.struct_types, self.env.interface_types)
            }
        }
    }
}
