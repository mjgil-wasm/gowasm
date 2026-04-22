use std::collections::HashMap;
use std::sync::Arc;

use gowasm_vm::{
    ConcreteType, Function, Instruction, ProgramTypeInventory, RuntimeTypeField, RuntimeTypeInfo,
    TypeCheck, TypeId,
};

use super::*;

#[derive(Default)]
pub(super) struct ArtifactRebaseMap {
    function_ranges: Vec<IndexRange>,
    global_ranges: Vec<IndexRange>,
    type_ranges: Vec<TypeRange>,
}

#[derive(Clone, Copy)]
struct IndexRange {
    old_start: usize,
    old_len: usize,
    new_start: usize,
}

#[derive(Clone, Copy)]
struct TypeRange {
    old_start: u32,
    old_len: u32,
    new_start: u32,
}

struct ArtifactRebaser<'a> {
    prior: &'a ArtifactRebaseMap,
    previous: &'a CompiledPackageArtifact,
    function_start: usize,
    global_start: usize,
    user_type_offset: u32,
}

impl ArtifactRebaseMap {
    pub(super) fn record_artifact(
        &mut self,
        previous: &CompiledPackageArtifact,
        current: &CompiledPackageArtifact,
    ) {
        self.function_ranges.push(IndexRange {
            old_start: previous.function_start,
            old_len: previous.functions.len(),
            new_start: current.function_start,
        });
        self.global_ranges.push(IndexRange {
            old_start: previous.global_start,
            old_len: previous.global_count,
            new_start: current.global_start,
        });
        self.type_ranges.push(TypeRange {
            old_start: previous.user_type_offset,
            old_len: previous.user_type_span,
            new_start: current.user_type_offset,
        });
    }

    pub(super) fn rebase_artifact(
        &self,
        previous: &CompiledPackageArtifact,
        function_start: usize,
        global_start: usize,
        user_type_offset: u32,
    ) -> CompiledPackageArtifact {
        let rebaser = ArtifactRebaser {
            prior: self,
            previous,
            function_start,
            global_start,
            user_type_offset,
        };
        CompiledPackageArtifact {
            import_path: previous.import_path.clone(),
            function_start,
            functions: previous
                .functions
                .iter()
                .map(|function| rebaser.rebase_function(function))
                .collect(),
            debug_infos: previous.debug_infos.clone(),
            methods: previous
                .methods
                .iter()
                .map(|method| rebaser.rebase_method_binding(method))
                .collect(),
            global_start,
            global_count: previous.global_count,
            user_type_offset,
            user_type_span: previous.user_type_span,
            type_inventory: rebaser.rebase_type_inventory(&previous.type_inventory),
            entry_function: previous
                .entry_function
                .map(|function| rebaser.rebase_function_index(function)),
            package_init_function: previous
                .package_init_function
                .map(|function| rebaser.rebase_function_index(function)),
            qualified_function_ids: previous
                .qualified_function_ids
                .iter()
                .map(|(name, function)| (name.clone(), rebaser.rebase_function_index(*function)))
                .collect(),
            qualified_generic_function_instances: previous
                .qualified_generic_function_instances
                .iter()
                .map(|(key, function)| (key.clone(), rebaser.rebase_function_index(*function)))
                .collect(),
            qualified_function_result_types: previous.qualified_function_result_types.clone(),
            qualified_function_types: previous.qualified_function_types.clone(),
            qualified_variadic_functions: previous.qualified_variadic_functions.clone(),
            qualified_globals: previous
                .qualified_globals
                .iter()
                .map(|(name, binding)| {
                    (
                        name.clone(),
                        GlobalBinding {
                            index: rebaser.rebase_global_index(binding.index),
                            typ: binding.typ.clone(),
                            is_const: binding.is_const,
                            const_value: binding.const_value.clone(),
                        },
                    )
                })
                .collect(),
            qualified_structs: previous
                .qualified_structs
                .iter()
                .map(|(name, struct_type)| {
                    (
                        name.clone(),
                        StructTypeDef {
                            type_id: rebaser.rebase_type_id(struct_type.type_id),
                            fields: struct_type.fields.clone(),
                        },
                    )
                })
                .collect(),
            qualified_interfaces: previous
                .qualified_interfaces
                .iter()
                .map(|(name, interface_type)| {
                    (
                        name.clone(),
                        InterfaceTypeDef {
                            type_id: rebaser.rebase_type_id(interface_type.type_id),
                            methods: interface_type.methods.clone(),
                        },
                    )
                })
                .collect(),
            qualified_pointers: previous
                .qualified_pointers
                .iter()
                .map(|(name, type_id)| (name.clone(), rebaser.rebase_type_id(*type_id)))
                .collect(),
            qualified_aliases: previous
                .qualified_aliases
                .iter()
                .map(|(name, alias_type)| {
                    (
                        name.clone(),
                        AliasTypeDef {
                            type_id: rebaser.rebase_type_id(alias_type.type_id),
                            underlying: alias_type.underlying.clone(),
                        },
                    )
                })
                .collect(),
            qualified_method_function_ids: previous
                .qualified_method_function_ids
                .iter()
                .map(|(key, function)| (key.clone(), rebaser.rebase_function_index(*function)))
                .collect(),
            qualified_promoted_method_bindings: previous.qualified_promoted_method_bindings.clone(),
            qualified_method_sets: previous.qualified_method_sets.clone(),
            dependency_edges: previous.dependency_edges.clone(),
            generic_function_template_sources: previous.generic_function_template_sources.clone(),
            generic_method_template_sources: previous.generic_method_template_sources.clone(),
            generic_package_context: previous
                .generic_package_context
                .as_ref()
                .map(|context| Arc::new(rebaser.rebase_generic_package_context(context))),
        }
    }

    fn rebase_function_index(&self, function: usize) -> Option<usize> {
        self.function_ranges
            .iter()
            .find(|range| function_in_range(function, **range))
            .map(|range| range.new_start + (function - range.old_start))
    }

    fn rebase_global_index(&self, global: usize) -> Option<usize> {
        self.global_ranges
            .iter()
            .find(|range| function_in_range(global, **range))
            .map(|range| range.new_start + (global - range.old_start))
    }

    fn rebase_type_id(&self, type_id: TypeId) -> Option<TypeId> {
        self.type_ranges
            .iter()
            .find(|range| type_in_range(type_id, **range))
            .map(|range| TypeId(range.new_start + (type_id.0 - range.old_start)))
    }
}

impl ArtifactRebaser<'_> {
    fn rebase_function(&self, function: &Function) -> Function {
        Function {
            name: function.name.clone(),
            param_count: function.param_count,
            register_count: function.register_count,
            code: function
                .code
                .iter()
                .map(|instruction| self.rebase_instruction(instruction))
                .collect(),
        }
    }

    fn rebase_instruction(&self, instruction: &Instruction) -> Instruction {
        let mut instruction = instruction.clone();
        match &mut instruction {
            Instruction::LoadNilChannel { concrete_type, .. }
            | Instruction::LoadNilSlice { concrete_type, .. }
            | Instruction::MakeArray { concrete_type, .. }
            | Instruction::MakeSlice { concrete_type, .. }
            | Instruction::MakeChannel { concrete_type, .. }
            | Instruction::MakeMap { concrete_type, .. }
            | Instruction::MakeNilMap { concrete_type, .. } => {
                *concrete_type = concrete_type
                    .as_ref()
                    .map(|typ| self.rebase_concrete_type(typ));
            }
            Instruction::LoadNilPointer {
                typ, concrete_type, ..
            } => {
                *typ = self.rebase_type_id(*typ);
                *concrete_type = concrete_type
                    .as_ref()
                    .map(|concrete| self.rebase_concrete_type(concrete));
            }
            Instruction::BoxHeap { typ, .. }
            | Instruction::AddressLocal { typ, .. }
            | Instruction::ProjectFieldPointer { typ, .. }
            | Instruction::ProjectIndexPointer { typ, .. }
            | Instruction::AddressLocalField { typ, .. }
            | Instruction::AddressLocalIndex { typ, .. }
            | Instruction::MakeStruct { typ, .. }
            | Instruction::Retag { typ, .. } => {
                *typ = self.rebase_type_id(*typ);
            }
            Instruction::AddressGlobal { global, typ, .. }
            | Instruction::AddressGlobalField { global, typ, .. }
            | Instruction::AddressGlobalIndex { global, typ, .. } => {
                *global = self.rebase_global_index(*global);
                *typ = self.rebase_type_id(*typ);
            }
            Instruction::LoadGlobal { global, .. } | Instruction::StoreGlobal { global, .. } => {
                *global = self.rebase_global_index(*global);
            }
            Instruction::AssertType { target, .. } | Instruction::TypeMatches { target, .. } => {
                *target = self.rebase_type_check(target);
            }
            Instruction::MakeClosure {
                concrete_type,
                function,
                ..
            } => {
                *concrete_type = concrete_type
                    .as_ref()
                    .map(|concrete| self.rebase_concrete_type(concrete));
                *function = self.rebase_function_index(*function);
            }
            Instruction::GoCall { function, .. }
            | Instruction::CallFunction { function, .. }
            | Instruction::DeferFunction { function, .. }
            | Instruction::CallFunctionMulti { function, .. } => {
                *function = self.rebase_function_index(*function);
            }
            _ => {}
        }
        instruction
    }

    fn rebase_method_binding(&self, method: &gowasm_vm::MethodBinding) -> gowasm_vm::MethodBinding {
        gowasm_vm::MethodBinding {
            receiver_type: self.rebase_type_id(method.receiver_type),
            target_receiver_type: self.rebase_type_id(method.target_receiver_type),
            name: method.name.clone(),
            function: self.rebase_function_index(method.function),
            param_types: method.param_types.clone(),
            result_types: method.result_types.clone(),
            promoted_fields: method.promoted_fields.clone(),
        }
    }

    fn rebase_type_check(&self, check: &TypeCheck) -> TypeCheck {
        match check {
            TypeCheck::Int => TypeCheck::Int,
            TypeCheck::Float64 => TypeCheck::Float64,
            TypeCheck::String => TypeCheck::String,
            TypeCheck::Bool => TypeCheck::Bool,
            TypeCheck::Exact { type_id, name } => TypeCheck::Exact {
                type_id: self.rebase_type_id(*type_id),
                name: name.clone(),
            },
            TypeCheck::Interface { name, methods } => TypeCheck::Interface {
                name: name.clone(),
                methods: methods.clone(),
            },
            TypeCheck::Struct { type_id, name } => TypeCheck::Struct {
                type_id: self.rebase_type_id(*type_id),
                name: name.clone(),
            },
        }
    }

    fn rebase_type_inventory(&self, inventory: &ProgramTypeInventory) -> ProgramTypeInventory {
        ProgramTypeInventory {
            types_by_id: inventory
                .types_by_id
                .iter()
                .map(|(type_id, info)| {
                    (
                        self.rebase_type_id(*type_id),
                        self.rebase_runtime_type(info),
                    )
                })
                .collect(),
        }
    }

    fn rebase_runtime_type(&self, info: &RuntimeTypeInfo) -> RuntimeTypeInfo {
        RuntimeTypeInfo {
            display_name: info.display_name.clone(),
            package_path: info.package_path.clone(),
            kind: info.kind.clone(),
            type_id: info.type_id.map(|type_id| self.rebase_type_id(type_id)),
            fields: info
                .fields
                .iter()
                .map(|field| RuntimeTypeField {
                    name: field.name.clone(),
                    typ: self.rebase_concrete_type(&field.typ),
                    embedded: field.embedded,
                    tag: field.tag.clone(),
                })
                .collect(),
            elem: info
                .elem
                .as_ref()
                .map(|typ| Box::new(self.rebase_concrete_type(typ))),
            key: info
                .key
                .as_ref()
                .map(|typ| Box::new(self.rebase_concrete_type(typ))),
            len: info.len,
            params: info
                .params
                .iter()
                .map(|typ| self.rebase_concrete_type(typ))
                .collect(),
            results: info
                .results
                .iter()
                .map(|typ| self.rebase_concrete_type(typ))
                .collect(),
            underlying: info
                .underlying
                .as_ref()
                .map(|typ| Box::new(self.rebase_concrete_type(typ))),
            channel_direction: info.channel_direction,
        }
    }

    fn rebase_concrete_type(&self, typ: &ConcreteType) -> ConcreteType {
        match typ {
            ConcreteType::TypeId(type_id) => ConcreteType::TypeId(self.rebase_type_id(*type_id)),
            ConcreteType::Array { len, element } => ConcreteType::Array {
                len: *len,
                element: Box::new(self.rebase_concrete_type(element)),
            },
            ConcreteType::Slice { element } => ConcreteType::Slice {
                element: Box::new(self.rebase_concrete_type(element)),
            },
            ConcreteType::Map { key, value } => ConcreteType::Map {
                key: Box::new(self.rebase_concrete_type(key)),
                value: Box::new(self.rebase_concrete_type(value)),
            },
            ConcreteType::Pointer { element } => ConcreteType::Pointer {
                element: Box::new(self.rebase_concrete_type(element)),
            },
            ConcreteType::Function { params, results } => ConcreteType::Function {
                params: params
                    .iter()
                    .map(|param| self.rebase_concrete_type(param))
                    .collect(),
                results: results
                    .iter()
                    .map(|result| self.rebase_concrete_type(result))
                    .collect(),
            },
            ConcreteType::Channel { direction, element } => ConcreteType::Channel {
                direction: *direction,
                element: Box::new(self.rebase_concrete_type(element)),
            },
        }
    }

    fn rebase_generic_package_context(
        &self,
        context: &imported_generics::ImportedGenericPackageContext,
    ) -> imported_generics::ImportedGenericPackageContext {
        imported_generics::ImportedGenericPackageContext {
            package_path: context.package_path.clone(),
            package_selector: context.package_selector.clone(),
            local_named_types: context.local_named_types.clone(),
            imported_bindings: self.rebase_imported_bindings(&context.imported_bindings),
            visible_generic_functions: context.visible_generic_functions.clone(),
            generic_functions: context.generic_functions.clone(),
            generic_types: context.generic_types.clone(),
            generic_function_templates: context.generic_function_templates.clone(),
            generic_method_templates: context.generic_method_templates.clone(),
            instantiation_cache: types::InstantiationCache {
                function_instances: context.instantiation_cache.function_instances.clone(),
                type_instances: context
                    .instantiation_cache
                    .type_instances
                    .iter()
                    .map(|(key, type_id)| (key.clone(), self.rebase_type_id(*type_id)))
                    .collect(),
            },
            function_ids: context
                .function_ids
                .iter()
                .map(|(name, function)| (name.clone(), self.rebase_function_index(*function)))
                .collect(),
            function_result_types: context.function_result_types.clone(),
            function_types: context.function_types.clone(),
            variadic_functions: context.variadic_functions.clone(),
            globals: context
                .globals
                .iter()
                .map(|(name, binding)| {
                    (
                        name.clone(),
                        GlobalBinding {
                            index: self.rebase_global_index(binding.index),
                            typ: binding.typ.clone(),
                            is_const: binding.is_const,
                            const_value: binding.const_value.clone(),
                        },
                    )
                })
                .collect(),
            structs: context
                .structs
                .iter()
                .map(|(name, struct_type)| {
                    (
                        name.clone(),
                        StructTypeDef {
                            type_id: self.rebase_type_id(struct_type.type_id),
                            fields: struct_type.fields.clone(),
                        },
                    )
                })
                .collect(),
            interfaces: context
                .interfaces
                .iter()
                .map(|(name, interface_type)| {
                    (
                        name.clone(),
                        InterfaceTypeDef {
                            type_id: self.rebase_type_id(interface_type.type_id),
                            methods: interface_type.methods.clone(),
                        },
                    )
                })
                .collect(),
            pointers: context
                .pointers
                .iter()
                .map(|(name, type_id)| (name.clone(), self.rebase_type_id(*type_id)))
                .collect(),
            aliases: context
                .aliases
                .iter()
                .map(|(name, alias_type)| {
                    (
                        name.clone(),
                        AliasTypeDef {
                            type_id: self.rebase_type_id(alias_type.type_id),
                            underlying: alias_type.underlying.clone(),
                        },
                    )
                })
                .collect(),
            method_function_ids: context
                .method_function_ids
                .iter()
                .map(|(key, function)| (key.clone(), self.rebase_function_index(*function)))
                .collect(),
            promoted_method_bindings: context.promoted_method_bindings.clone(),
            method_sets: context.method_sets.clone(),
        }
    }

    fn rebase_imported_bindings(
        &self,
        snapshot: &imported_generics::ImportedBindingsSnapshot,
    ) -> imported_generics::ImportedBindingsSnapshot {
        imported_generics::ImportedBindingsSnapshot {
            function_ids: snapshot
                .function_ids
                .iter()
                .map(|(name, function)| (name.clone(), self.rebase_function_index(*function)))
                .collect(),
            generic_function_instances: snapshot
                .generic_function_instances
                .iter()
                .map(|(key, function)| (key.clone(), self.rebase_function_index(*function)))
                .collect(),
            function_result_types: snapshot.function_result_types.clone(),
            function_types: snapshot.function_types.clone(),
            variadic_functions: snapshot.variadic_functions.clone(),
            globals: snapshot
                .globals
                .iter()
                .map(|(name, binding)| {
                    (
                        name.clone(),
                        GlobalBinding {
                            index: self.rebase_global_index(binding.index),
                            typ: binding.typ.clone(),
                            is_const: binding.is_const,
                            const_value: binding.const_value.clone(),
                        },
                    )
                })
                .collect(),
            structs: snapshot
                .structs
                .iter()
                .map(|(name, struct_type)| {
                    (
                        name.clone(),
                        StructTypeDef {
                            type_id: self.rebase_type_id(struct_type.type_id),
                            fields: struct_type.fields.clone(),
                        },
                    )
                })
                .collect(),
            interfaces: snapshot
                .interfaces
                .iter()
                .map(|(name, interface_type)| {
                    (
                        name.clone(),
                        InterfaceTypeDef {
                            type_id: self.rebase_type_id(interface_type.type_id),
                            methods: interface_type.methods.clone(),
                        },
                    )
                })
                .collect(),
            pointers: snapshot
                .pointers
                .iter()
                .map(|(name, type_id)| (name.clone(), self.rebase_type_id(*type_id)))
                .collect(),
            aliases: snapshot
                .aliases
                .iter()
                .map(|(name, alias_type)| {
                    (
                        name.clone(),
                        AliasTypeDef {
                            type_id: self.rebase_type_id(alias_type.type_id),
                            underlying: alias_type.underlying.clone(),
                        },
                    )
                })
                .collect(),
            method_function_ids: snapshot
                .method_function_ids
                .iter()
                .map(|(key, function)| (key.clone(), self.rebase_function_index(*function)))
                .collect(),
            promoted_method_bindings: snapshot.promoted_method_bindings.clone(),
            method_sets: snapshot.method_sets.clone(),
            generic_package_contexts: snapshot
                .generic_package_contexts
                .iter()
                .map(|(package, context)| {
                    (
                        package.clone(),
                        Arc::new(self.rebase_generic_package_context(context)),
                    )
                })
                .collect::<HashMap<_, _>>(),
        }
    }

    fn rebase_function_index(&self, function: usize) -> usize {
        self.prior
            .rebase_function_index(function)
            .or_else(|| self.rebase_local_function_index(function))
            .unwrap_or(function)
    }

    fn rebase_local_function_index(&self, function: usize) -> Option<usize> {
        if function >= self.previous.function_start
            && function < self.previous.function_start + self.previous.functions.len()
        {
            Some(self.function_start + (function - self.previous.function_start))
        } else {
            None
        }
    }

    fn rebase_global_index(&self, global: usize) -> usize {
        self.prior
            .rebase_global_index(global)
            .or_else(|| self.rebase_local_global_index(global))
            .unwrap_or(global)
    }

    fn rebase_local_global_index(&self, global: usize) -> Option<usize> {
        if global >= self.previous.global_start
            && global < self.previous.global_start + self.previous.global_count
        {
            Some(self.global_start + (global - self.previous.global_start))
        } else {
            None
        }
    }

    fn rebase_type_id(&self, type_id: TypeId) -> TypeId {
        self.prior
            .rebase_type_id(type_id)
            .or_else(|| self.rebase_local_type_id(type_id))
            .unwrap_or(type_id)
    }

    fn rebase_local_type_id(&self, type_id: TypeId) -> Option<TypeId> {
        if type_id.0 >= self.previous.user_type_offset
            && type_id.0 < self.previous.user_type_offset + self.previous.user_type_span
        {
            Some(TypeId(
                self.user_type_offset + (type_id.0 - self.previous.user_type_offset),
            ))
        } else {
            None
        }
    }
}

fn function_in_range(index: usize, range: IndexRange) -> bool {
    index >= range.old_start && index < range.old_start + range.old_len
}

fn type_in_range(type_id: TypeId, range: TypeRange) -> bool {
    type_id.0 >= range.old_start && type_id.0 < range.old_start + range.old_len
}
