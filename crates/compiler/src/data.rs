use std::collections::HashMap;

use super::*;
use gowasm_parser::{Expr, MapLiteralEntry, StructLiteralField};

impl FunctionBuilder<'_> {
    pub(super) fn compile_zero_value(&mut self, dst: usize, typ: &str) -> Result<(), CompileError> {
        match typ {
            "int" | "int64" | "byte" | "rune" => self
                .emitter
                .code
                .push(Instruction::LoadInt { dst, value: 0 }),
            "float64" => self.emitter.code.push(Instruction::LoadFloat {
                dst,
                value: gowasm_vm::Float64(0.0),
            }),
            "bool" => self
                .emitter
                .code
                .push(Instruction::LoadBool { dst, value: false }),
            "string" => self.emitter.code.push(Instruction::LoadString {
                dst,
                value: String::new(),
            }),
            typ if parse_function_type(typ).is_some() => {
                self.emitter.code.push(Instruction::LoadNil { dst })
            }
            typ if parse_pointer_type(typ).is_some() => {
                self.emitter.code.push(Instruction::LoadNilPointer {
                    dst,
                    typ: self.pointer_runtime_type(parse_pointer_type(typ).expect("pointer type")),
                    concrete_type: Some(self.lower_runtime_concrete_type(typ)?),
                })
            }
            typ if parse_channel_type(typ).is_some() => {
                self.emitter.code.push(Instruction::LoadNilChannel {
                    dst,
                    concrete_type: Some(self.lower_runtime_concrete_type(typ)?),
                })
            }
            typ if typ.starts_with("[]") => self.emitter.code.push(Instruction::LoadNilSlice {
                dst,
                concrete_type: Some(self.lower_runtime_concrete_type(typ)?),
            }),
            other => {
                if other == "interface{}"
                    || other == "error"
                    || other == "any"
                    || self.instantiated_interface_type(other).is_some()
                {
                    self.emitter.code.push(Instruction::LoadNil { dst });
                    let interface_type = match other {
                        "interface{}" | "any" => Some(gowasm_vm::TYPE_ANY),
                        "error" => Some(gowasm_vm::TYPE_ERROR),
                        _ => self
                            .instantiated_interface_type(other)
                            .map(|typ| typ.type_id),
                    };
                    if let Some(interface_type) = interface_type {
                        self.emitter.code.push(Instruction::Retag {
                            dst,
                            src: dst,
                            typ: interface_type,
                        });
                    }
                    return Ok(());
                }
                if let Some((_key_type, value_type)) = parse_map_type(other) {
                    let zero = self.alloc_register();
                    self.compile_zero_value(zero, value_type)?;
                    self.emitter.code.push(Instruction::MakeNilMap {
                        dst,
                        concrete_type: Some(self.lower_runtime_concrete_type(other)?),
                        zero,
                    });
                    return Ok(());
                }
                if let Some((len, element_type)) = parse_array_type(other) {
                    let mut items = Vec::with_capacity(len);
                    for _ in 0..len {
                        let item = self.alloc_register();
                        self.compile_zero_value(item, element_type)?;
                        items.push(item);
                    }
                    self.emitter.code.push(Instruction::MakeArray {
                        dst,
                        concrete_type: Some(self.lower_runtime_concrete_type(other)?),
                        items,
                    });
                    return Ok(());
                }
                if let Some(struct_type) = self.instantiated_struct_type(other) {
                    return self.compile_zero_struct(dst, &struct_type);
                }
                if let Some(alias_type) = self.instantiated_alias_type(other) {
                    self.compile_zero_value(dst, &alias_type.underlying)?;
                    self.emitter.code.push(Instruction::Retag {
                        dst,
                        src: dst,
                        typ: alias_type.type_id,
                    });
                    return Ok(());
                }
                return Err(CompileError::Unsupported {
                    detail: format!("unsupported local `var` type `{other}`"),
                });
            }
        }
        Ok(())
    }

    pub(super) fn compile_collection_items(
        &mut self,
        elements: &[Expr],
    ) -> Result<Vec<usize>, CompileError> {
        let mut items = Vec::with_capacity(elements.len());
        for element in elements {
            items.push(self.compile_value_expr(element)?);
        }
        Ok(items)
    }

    pub(super) fn compile_map_literal(
        &mut self,
        dst: usize,
        key_type: &str,
        value_type: &str,
        entries: &[MapLiteralEntry],
    ) -> Result<(), CompileError> {
        let mut lowered_entries = Vec::with_capacity(entries.len());
        for entry in entries {
            self.validate_assignable_type(Some(key_type), &entry.key)?;
            let key = self.alloc_register();
            self.compile_expr_into_with_hint(key, &entry.key, Some(key_type))?;
            self.validate_assignable_type(Some(value_type), &entry.value)?;
            let value = self.alloc_register();
            self.compile_expr_into_with_hint(value, &entry.value, Some(value_type))?;
            lowered_entries.push((key, value));
        }
        let zero = self.alloc_register();
        self.compile_zero_value(zero, value_type)?;
        self.emitter.code.push(Instruction::MakeMap {
            dst,
            concrete_type: Some(
                self.lower_runtime_concrete_type(&format!("map[{key_type}]{value_type}"))?,
            ),
            entries: lowered_entries,
            zero,
        });
        Ok(())
    }

    pub(super) fn compile_struct_literal(
        &mut self,
        dst: usize,
        type_name: &str,
        fields: &[StructLiteralField],
    ) -> Result<(), CompileError> {
        self.ensure_runtime_visible_type(type_name)?;
        let struct_type =
            self.instantiated_struct_type(type_name)
                .ok_or_else(|| CompileError::Unsupported {
                    detail: format!("unknown struct type `{type_name}`"),
                })?;
        let positional = fields.first().is_some_and(|f| f.name.is_empty());

        let mut lowered_fields = Vec::with_capacity(struct_type.fields.len());
        if positional {
            if fields.len() != struct_type.fields.len() {
                return Err(CompileError::Unsupported {
                    detail: format!(
                        "struct `{type_name}` has {} fields, but literal provides {}",
                        struct_type.fields.len(),
                        fields.len()
                    ),
                });
            }
            if let Some(field) = struct_type.fields.iter().find(|field| {
                !self.field_is_accessible_from_current_package(type_name, &field.name)
            }) {
                return Err(self.literal_rejects_unexported_field(type_name, &field.name));
            }
            for (type_field, literal_field) in struct_type.fields.iter().zip(fields) {
                self.validate_assignable_type(Some(&type_field.typ), &literal_field.value)?;
                let register = self.alloc_register();
                self.compile_expr_into_with_hint(
                    register,
                    &literal_field.value,
                    Some(&type_field.typ),
                )?;
                lowered_fields.push((type_field.name.clone(), register));
            }
        } else {
            let mut provided = HashMap::new();
            for field in fields {
                if provided.insert(field.name.as_str(), &field.value).is_some() {
                    return Err(CompileError::Unsupported {
                        detail: format!(
                            "duplicate field `{}` in `{type_name}` literal",
                            field.name
                        ),
                    });
                }
            }
            for field in &struct_type.fields {
                if provided.contains_key(field.name.as_str())
                    && !self.field_is_accessible_from_current_package(type_name, &field.name)
                {
                    return Err(self.literal_rejects_unexported_field(type_name, &field.name));
                }
            }

            for field in &struct_type.fields {
                let register = self.alloc_register();
                if let Some(value) = provided.remove(field.name.as_str()) {
                    self.validate_assignable_type(Some(&field.typ), value)?;
                    self.compile_expr_into_with_hint(register, value, Some(&field.typ))?;
                } else {
                    self.compile_zero_value(register, &field.typ)?;
                }
                lowered_fields.push((field.name.clone(), register));
            }

            if let Some(field) = provided.keys().next() {
                return Err(CompileError::Unsupported {
                    detail: format!("unknown field `{field}` in `{type_name}` literal"),
                });
            }
        }

        self.emitter.code.push(Instruction::MakeStruct {
            dst,
            typ: struct_type.type_id,
            fields: lowered_fields,
        });
        Ok(())
    }

    pub(super) fn compile_zero_struct(
        &mut self,
        dst: usize,
        struct_type: &StructTypeDef,
    ) -> Result<(), CompileError> {
        let mut fields = Vec::with_capacity(struct_type.fields.len());
        for field in &struct_type.fields {
            let register = self.alloc_register();
            self.compile_zero_value(register, &field.typ)?;
            fields.push((field.name.clone(), register));
        }
        self.emitter.code.push(Instruction::MakeStruct {
            dst,
            typ: struct_type.type_id,
            fields,
        });
        Ok(())
    }
}
