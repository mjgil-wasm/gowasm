use std::collections::HashMap;

use gowasm_vm::{
    ConcreteType, ProgramTypeInventory, RuntimeChannelDirection, RuntimeTypeField, RuntimeTypeInfo,
    RuntimeTypeKind, TypeId, TYPE_ANY, TYPE_ARRAY, TYPE_BOOL, TYPE_CHANNEL, TYPE_ERROR,
    TYPE_FLOAT64, TYPE_FUNCTION, TYPE_INT, TYPE_INT64, TYPE_MAP, TYPE_POINTER, TYPE_SLICE,
    TYPE_STRING,
};

use super::*;
use crate::types::{ChannelDirection, TypeTables};

pub(super) fn build_package_type_inventory(
    package_path: &str,
    type_tables: &TypeTables,
    instantiated_generics: &generic_instances::InstantiatedGenerics,
) -> Result<ProgramTypeInventory, CompileError> {
    let builder = InventoryBuilder {
        package_path,
        type_tables,
        instantiated_generics,
    };
    builder.build()
}

pub(super) fn merge_program_type_inventories(
    inventories: impl IntoIterator<Item = ProgramTypeInventory>,
) -> ProgramTypeInventory {
    let mut merged = base_runtime_type_inventory();
    for inventory in inventories {
        merged.types_by_id.extend(inventory.types_by_id);
    }
    merged
}

struct InventoryBuilder<'a> {
    package_path: &'a str,
    type_tables: &'a TypeTables,
    instantiated_generics: &'a generic_instances::InstantiatedGenerics,
}

impl InventoryBuilder<'_> {
    fn build(&self) -> Result<ProgramTypeInventory, CompileError> {
        let mut inventory = base_runtime_type_inventory();
        self.register_structs(&mut inventory, &self.type_tables.structs)?;
        self.register_interfaces(&mut inventory, &self.type_tables.interfaces)?;
        self.register_aliases(&mut inventory, &self.type_tables.aliases)?;
        self.register_pointers(&mut inventory, &self.type_tables.pointers)?;
        self.register_structs(&mut inventory, self.instantiated_generics.struct_types())?;
        self.register_interfaces(&mut inventory, self.instantiated_generics.interface_types())?;
        self.register_aliases(&mut inventory, self.instantiated_generics.alias_types())?;
        self.register_pointers(&mut inventory, self.instantiated_generics.pointer_types())?;
        Ok(inventory)
    }

    fn register_structs(
        &self,
        inventory: &mut ProgramTypeInventory,
        structs: &HashMap<String, StructTypeDef>,
    ) -> Result<(), CompileError> {
        for name in structs.keys() {
            inventory.register(self.type_info_for_type_name(name)?);
        }
        Ok(())
    }

    fn register_interfaces(
        &self,
        inventory: &mut ProgramTypeInventory,
        interfaces: &HashMap<String, InterfaceTypeDef>,
    ) -> Result<(), CompileError> {
        for name in interfaces.keys() {
            inventory.register(self.type_info_for_type_name(name)?);
        }
        Ok(())
    }

    fn register_aliases(
        &self,
        inventory: &mut ProgramTypeInventory,
        aliases: &HashMap<String, AliasTypeDef>,
    ) -> Result<(), CompileError> {
        for name in aliases.keys() {
            inventory.register(self.type_info_for_type_name(name)?);
        }
        Ok(())
    }

    fn register_pointers(
        &self,
        inventory: &mut ProgramTypeInventory,
        pointers: &HashMap<String, TypeId>,
    ) -> Result<(), CompileError> {
        for name in pointers.keys() {
            inventory.register(self.type_info_for_type_name(name)?);
        }
        Ok(())
    }

    fn type_info_for_type_name(&self, typ: &str) -> Result<RuntimeTypeInfo, CompileError> {
        if let Some(struct_type) = self.lookup_struct(typ) {
            return Ok(RuntimeTypeInfo {
                display_name: typ.to_string(),
                package_path: self.named_type_package_path(typ),
                kind: RuntimeTypeKind::Struct,
                type_id: Some(struct_type.type_id),
                fields: struct_type
                    .fields
                    .iter()
                    .map(|field| {
                        Ok(RuntimeTypeField {
                            name: field.name.clone(),
                            typ: self.lower_runtime_concrete_type(&field.typ)?,
                            embedded: field.embedded,
                            tag: field.tag.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?,
                elem: None,
                key: None,
                len: None,
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            });
        }
        if let Some(interface_type) = self.lookup_interface(typ) {
            return Ok(RuntimeTypeInfo {
                display_name: typ.to_string(),
                package_path: self.named_type_package_path(typ),
                kind: RuntimeTypeKind::Interface,
                type_id: Some(interface_type.type_id),
                fields: Vec::new(),
                elem: None,
                key: None,
                len: None,
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            });
        }
        if let Some(alias_type) = self.lookup_alias(typ) {
            let underlying = self.lower_runtime_concrete_type(&alias_type.underlying)?;
            let mut info = self.runtime_info_from_concrete(&underlying)?;
            info.display_name = typ.to_string();
            info.package_path = self.named_type_package_path(typ);
            info.type_id = Some(alias_type.type_id);
            info.underlying = Some(Box::new(underlying));
            return Ok(info);
        }
        if let Some(pointer_type) = self.lookup_pointer_type(typ) {
            let element = parse_pointer_type(typ).ok_or_else(|| CompileError::Unsupported {
                detail: format!("runtime reflection metadata expected pointer type `{typ}`"),
            })?;
            return Ok(RuntimeTypeInfo {
                display_name: typ.to_string(),
                package_path: None,
                kind: RuntimeTypeKind::Pointer,
                type_id: Some(pointer_type),
                fields: Vec::new(),
                elem: Some(Box::new(self.lower_runtime_concrete_type(element)?)),
                key: None,
                len: None,
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            });
        }
        self.runtime_info_from_concrete(&self.lower_runtime_concrete_type(typ)?)
    }

    fn runtime_info_from_concrete(
        &self,
        typ: &ConcreteType,
    ) -> Result<RuntimeTypeInfo, CompileError> {
        match typ {
            ConcreteType::TypeId(type_id) => self.type_info_for_type_id(*type_id),
            ConcreteType::Array { len, element } => Ok(RuntimeTypeInfo {
                display_name: format!("[{len}]{}", self.display_concrete_type(element)?),
                package_path: None,
                kind: RuntimeTypeKind::Array,
                type_id: None,
                fields: Vec::new(),
                elem: Some(element.clone()),
                key: None,
                len: Some(*len),
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            }),
            ConcreteType::Slice { element } => Ok(RuntimeTypeInfo {
                display_name: format!("[]{}", self.display_concrete_type(element)?),
                package_path: None,
                kind: RuntimeTypeKind::Slice,
                type_id: None,
                fields: Vec::new(),
                elem: Some(element.clone()),
                key: None,
                len: None,
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            }),
            ConcreteType::Map { key, value } => Ok(RuntimeTypeInfo {
                display_name: format!(
                    "map[{}]{}",
                    self.display_concrete_type(key)?,
                    self.display_concrete_type(value)?
                ),
                package_path: None,
                kind: RuntimeTypeKind::Map,
                type_id: None,
                fields: Vec::new(),
                elem: Some(value.clone()),
                key: Some(key.clone()),
                len: None,
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            }),
            ConcreteType::Pointer { element } => Ok(RuntimeTypeInfo {
                display_name: format!("*{}", self.display_concrete_type(element)?),
                package_path: None,
                kind: RuntimeTypeKind::Pointer,
                type_id: None,
                fields: Vec::new(),
                elem: Some(element.clone()),
                key: None,
                len: None,
                params: Vec::new(),
                results: Vec::new(),
                underlying: None,
                channel_direction: None,
            }),
            ConcreteType::Function { params, results } => Ok(RuntimeTypeInfo {
                display_name: format!(
                    "__gowasm_func__({})->({})",
                    self.display_concrete_types(params)?,
                    self.display_concrete_types(results)?,
                ),
                package_path: None,
                kind: RuntimeTypeKind::Function,
                type_id: None,
                fields: Vec::new(),
                elem: None,
                key: None,
                len: None,
                params: params.clone(),
                results: results.clone(),
                underlying: None,
                channel_direction: None,
            }),
            ConcreteType::Channel { direction, element } => {
                let prefix = match direction {
                    RuntimeChannelDirection::Bidirectional => "chan ",
                    RuntimeChannelDirection::SendOnly => "chan<- ",
                    RuntimeChannelDirection::ReceiveOnly => "<-chan ",
                };
                Ok(RuntimeTypeInfo {
                    display_name: format!("{prefix}{}", self.display_concrete_type(element)?),
                    package_path: None,
                    kind: RuntimeTypeKind::Channel,
                    type_id: None,
                    fields: Vec::new(),
                    elem: Some(element.clone()),
                    key: None,
                    len: None,
                    params: Vec::new(),
                    results: Vec::new(),
                    underlying: None,
                    channel_direction: Some(*direction),
                })
            }
        }
    }

    fn type_info_for_type_id(&self, type_id: TypeId) -> Result<RuntimeTypeInfo, CompileError> {
        if let Some(info) = base_runtime_type_info(type_id) {
            return Ok(info);
        }
        let name = self
            .type_name_for_id(type_id)
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!(
                    "runtime reflection metadata is missing type id `{}`",
                    type_id.0
                ),
            })?;
        self.type_info_for_type_name(&name)
    }

    fn type_name_for_id(&self, type_id: TypeId) -> Option<String> {
        self.lookup_name_by_type_id(&self.type_tables.structs, type_id)
            .or_else(|| self.lookup_name_by_type_id(&self.type_tables.interfaces, type_id))
            .or_else(|| self.lookup_name_by_type_id(&self.type_tables.aliases, type_id))
            .or_else(|| self.lookup_name_by_type_id(&self.type_tables.pointers, type_id))
            .or_else(|| {
                self.lookup_name_by_type_id(self.instantiated_generics.struct_types(), type_id)
            })
            .or_else(|| {
                self.lookup_name_by_type_id(self.instantiated_generics.interface_types(), type_id)
            })
            .or_else(|| {
                self.lookup_name_by_type_id(self.instantiated_generics.alias_types(), type_id)
            })
            .or_else(|| {
                self.lookup_name_by_type_id(self.instantiated_generics.pointer_types(), type_id)
            })
    }

    fn lower_runtime_concrete_type(&self, typ: &str) -> Result<ConcreteType, CompileError> {
        match typ {
            "int" | "byte" | "rune" => Ok(ConcreteType::TypeId(TYPE_INT)),
            "int64" => Ok(ConcreteType::TypeId(TYPE_INT64)),
            "float64" => Ok(ConcreteType::TypeId(TYPE_FLOAT64)),
            "string" => Ok(ConcreteType::TypeId(TYPE_STRING)),
            "bool" => Ok(ConcreteType::TypeId(TYPE_BOOL)),
            "interface{}" | "any" => Ok(ConcreteType::TypeId(TYPE_ANY)),
            "error" => Ok(ConcreteType::TypeId(TYPE_ERROR)),
            _ => {
                if let Some(struct_type) = self.lookup_struct(typ) {
                    return Ok(ConcreteType::TypeId(struct_type.type_id));
                }
                if let Some(interface_type) = self.lookup_interface(typ) {
                    return Ok(ConcreteType::TypeId(interface_type.type_id));
                }
                if let Some(alias_type) = self.lookup_alias(typ) {
                    return Ok(ConcreteType::TypeId(alias_type.type_id));
                }
                if let Some(pointer_type) = self.lookup_pointer_type(typ) {
                    return Ok(ConcreteType::TypeId(pointer_type));
                }
                if let Some((len, element)) = parse_array_type(typ) {
                    return Ok(ConcreteType::Array {
                        len,
                        element: Box::new(self.lower_runtime_concrete_type(element)?),
                    });
                }
                if let Some(element) = typ.strip_prefix("[]") {
                    return Ok(ConcreteType::Slice {
                        element: Box::new(self.lower_runtime_concrete_type(element)?),
                    });
                }
                if let Some((key, value)) = parse_map_type(typ) {
                    return Ok(ConcreteType::Map {
                        key: Box::new(self.lower_runtime_concrete_type(key)?),
                        value: Box::new(self.lower_runtime_concrete_type(value)?),
                    });
                }
                if let Some(channel) = parse_channel_type(typ) {
                    return Ok(ConcreteType::Channel {
                        direction: runtime_channel_direction(channel.direction),
                        element: Box::new(self.lower_runtime_concrete_type(channel.element_type)?),
                    });
                }
                if let Some((params, results)) = parse_function_type(typ) {
                    return Ok(ConcreteType::Function {
                        params: params
                            .iter()
                            .map(|typ| self.lower_runtime_concrete_type(typ))
                            .collect::<Result<Vec<_>, _>>()?,
                        results: results
                            .iter()
                            .map(|typ| self.lower_runtime_concrete_type(typ))
                            .collect::<Result<Vec<_>, _>>()?,
                    });
                }
                if let Some(inner) = parse_pointer_type(typ) {
                    return Ok(ConcreteType::Pointer {
                        element: Box::new(self.lower_runtime_concrete_type(inner)?),
                    });
                }
                Err(CompileError::Unsupported {
                    detail: format!(
                        "runtime reflection metadata does not yet support type `{typ}`"
                    ),
                })
            }
        }
    }

    fn display_concrete_type(&self, typ: &ConcreteType) -> Result<String, CompileError> {
        Ok(self.runtime_info_from_concrete(typ)?.display_name)
    }

    fn display_concrete_types(&self, types: &[ConcreteType]) -> Result<String, CompileError> {
        types
            .iter()
            .map(|typ| self.display_concrete_type(typ))
            .collect::<Result<Vec<_>, _>>()
            .map(|types| types.join(","))
    }

    fn named_type_package_path(&self, typ: &str) -> Option<String> {
        if typ == "struct{}" || typ.starts_with("struct{") {
            return None;
        }
        qualified_package_path(typ).or_else(|| match self.package_path {
            "" => None,
            "." => Some("main".into()),
            other => Some(other.to_string()),
        })
    }

    fn lookup_struct(&self, typ: &str) -> Option<&StructTypeDef> {
        self.instantiated_generics
            .struct_type(typ)
            .or_else(|| self.type_tables.structs.get(typ))
    }

    fn lookup_interface(&self, typ: &str) -> Option<&InterfaceTypeDef> {
        self.instantiated_generics
            .interface_type(typ)
            .or_else(|| self.type_tables.interfaces.get(typ))
    }

    fn lookup_alias(&self, typ: &str) -> Option<&AliasTypeDef> {
        self.instantiated_generics
            .alias_type(typ)
            .or_else(|| self.type_tables.aliases.get(typ))
    }

    fn lookup_pointer_type(&self, typ: &str) -> Option<TypeId> {
        self.instantiated_generics
            .pointer_type(typ)
            .or_else(|| self.type_tables.pointers.get(typ).copied())
    }

    fn lookup_name_by_type_id<T>(
        &self,
        entries: &HashMap<String, T>,
        type_id: TypeId,
    ) -> Option<String>
    where
        T: RuntimeTypeId,
    {
        entries
            .iter()
            .find_map(|(name, entry)| (entry.runtime_type_id() == type_id).then(|| name.clone()))
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn lower_runtime_concrete_type(
        &self,
        typ: &str,
    ) -> Result<ConcreteType, CompileError> {
        match typ {
            "int" | "byte" | "rune" => Ok(ConcreteType::TypeId(TYPE_INT)),
            "int64" => Ok(ConcreteType::TypeId(TYPE_INT64)),
            "float64" => Ok(ConcreteType::TypeId(TYPE_FLOAT64)),
            "string" => Ok(ConcreteType::TypeId(TYPE_STRING)),
            "bool" => Ok(ConcreteType::TypeId(TYPE_BOOL)),
            "interface{}" | "any" => Ok(ConcreteType::TypeId(TYPE_ANY)),
            "error" => Ok(ConcreteType::TypeId(TYPE_ERROR)),
            _ => {
                if let Some(struct_type) = self.instantiated_struct_type(typ) {
                    return Ok(ConcreteType::TypeId(struct_type.type_id));
                }
                if let Some(interface_type) = self.instantiated_interface_type(typ) {
                    return Ok(ConcreteType::TypeId(interface_type.type_id));
                }
                if let Some(alias_type) = self.instantiated_alias_type(typ) {
                    return Ok(ConcreteType::TypeId(alias_type.type_id));
                }
                if let Some(pointer_type) = self.instantiated_pointer_type(typ) {
                    return Ok(ConcreteType::TypeId(pointer_type));
                }
                if let Some((len, element)) = parse_array_type(typ) {
                    return Ok(ConcreteType::Array {
                        len,
                        element: Box::new(self.lower_runtime_concrete_type(element)?),
                    });
                }
                if let Some(element) = typ.strip_prefix("[]") {
                    return Ok(ConcreteType::Slice {
                        element: Box::new(self.lower_runtime_concrete_type(element)?),
                    });
                }
                if let Some((key, value)) = parse_map_type(typ) {
                    return Ok(ConcreteType::Map {
                        key: Box::new(self.lower_runtime_concrete_type(key)?),
                        value: Box::new(self.lower_runtime_concrete_type(value)?),
                    });
                }
                if let Some(channel) = parse_channel_type(typ) {
                    return Ok(ConcreteType::Channel {
                        direction: runtime_channel_direction(channel.direction),
                        element: Box::new(self.lower_runtime_concrete_type(channel.element_type)?),
                    });
                }
                if let Some((params, results)) = parse_function_type(typ) {
                    return Ok(ConcreteType::Function {
                        params: params
                            .iter()
                            .map(|typ| self.lower_runtime_concrete_type(typ))
                            .collect::<Result<Vec<_>, _>>()?,
                        results: results
                            .iter()
                            .map(|typ| self.lower_runtime_concrete_type(typ))
                            .collect::<Result<Vec<_>, _>>()?,
                    });
                }
                if let Some(inner) = parse_pointer_type(typ) {
                    return Ok(ConcreteType::Pointer {
                        element: Box::new(self.lower_runtime_concrete_type(inner)?),
                    });
                }
                Err(CompileError::Unsupported {
                    detail: format!(
                        "runtime reflection metadata does not yet support type `{typ}`"
                    ),
                })
            }
        }
    }
}

trait RuntimeTypeId {
    fn runtime_type_id(&self) -> TypeId;
}

impl RuntimeTypeId for StructTypeDef {
    fn runtime_type_id(&self) -> TypeId {
        self.type_id
    }
}

impl RuntimeTypeId for InterfaceTypeDef {
    fn runtime_type_id(&self) -> TypeId {
        self.type_id
    }
}

impl RuntimeTypeId for AliasTypeDef {
    fn runtime_type_id(&self) -> TypeId {
        self.type_id
    }
}

impl RuntimeTypeId for TypeId {
    fn runtime_type_id(&self) -> TypeId {
        *self
    }
}

fn base_runtime_type_inventory() -> ProgramTypeInventory {
    let mut inventory = ProgramTypeInventory::default();
    for info in [
        RuntimeTypeInfo::scalar("int", RuntimeTypeKind::Int, Some(TYPE_INT)),
        RuntimeTypeInfo::scalar("int64", RuntimeTypeKind::Int, Some(TYPE_INT64)),
        RuntimeTypeInfo::scalar("float64", RuntimeTypeKind::Float64, Some(TYPE_FLOAT64)),
        RuntimeTypeInfo::scalar("string", RuntimeTypeKind::String, Some(TYPE_STRING)),
        RuntimeTypeInfo::scalar("bool", RuntimeTypeKind::Bool, Some(TYPE_BOOL)),
        RuntimeTypeInfo::scalar("interface{}", RuntimeTypeKind::Interface, Some(TYPE_ANY)),
        RuntimeTypeInfo::scalar("error", RuntimeTypeKind::Interface, Some(TYPE_ERROR)),
        generic_runtime_type_info("array", RuntimeTypeKind::Array, TYPE_ARRAY),
        generic_runtime_type_info("slice", RuntimeTypeKind::Slice, TYPE_SLICE),
        generic_runtime_type_info("map", RuntimeTypeKind::Map, TYPE_MAP),
        generic_runtime_type_info("pointer", RuntimeTypeKind::Pointer, TYPE_POINTER),
        generic_runtime_type_info("function", RuntimeTypeKind::Function, TYPE_FUNCTION),
        generic_runtime_type_info("channel", RuntimeTypeKind::Channel, TYPE_CHANNEL),
        imported_interface_runtime_type_info("context.Context", TypeId(100)),
        imported_interface_runtime_type_info("fs.FS", TypeId(102)),
        imported_interface_runtime_type_info("fs.File", TypeId(103)),
        imported_interface_runtime_type_info("fs.FileInfo", TypeId(104)),
        imported_interface_runtime_type_info("fs.DirEntry", TypeId(105)),
        imported_interface_runtime_type_info("fs.ReadDirFile", TypeId(106)),
        imported_interface_runtime_type_info("fs.ReadFileFS", TypeId(107)),
        imported_interface_runtime_type_info("fs.StatFS", TypeId(108)),
        imported_interface_runtime_type_info("fs.ReadDirFS", TypeId(109)),
        imported_interface_runtime_type_info("fs.GlobFS", TypeId(110)),
        imported_interface_runtime_type_info("fs.SubFS", TypeId(111)),
        imported_interface_runtime_type_info("io.Reader", TypeId(112)),
        imported_interface_runtime_type_info("io.Closer", TypeId(113)),
        imported_interface_runtime_type_info("io.ReadCloser", TypeId(114)),
    ] {
        inventory.register(info);
    }
    inventory
}

fn base_runtime_type_info(type_id: TypeId) -> Option<RuntimeTypeInfo> {
    match type_id {
        TYPE_INT => Some(RuntimeTypeInfo::scalar(
            "int",
            RuntimeTypeKind::Int,
            Some(TYPE_INT),
        )),
        TYPE_INT64 => Some(RuntimeTypeInfo::scalar(
            "int64",
            RuntimeTypeKind::Int,
            Some(TYPE_INT64),
        )),
        TYPE_FLOAT64 => Some(RuntimeTypeInfo::scalar(
            "float64",
            RuntimeTypeKind::Float64,
            Some(TYPE_FLOAT64),
        )),
        TYPE_STRING => Some(RuntimeTypeInfo::scalar(
            "string",
            RuntimeTypeKind::String,
            Some(TYPE_STRING),
        )),
        TYPE_BOOL => Some(RuntimeTypeInfo::scalar(
            "bool",
            RuntimeTypeKind::Bool,
            Some(TYPE_BOOL),
        )),
        TYPE_ANY => Some(RuntimeTypeInfo::scalar(
            "interface{}",
            RuntimeTypeKind::Interface,
            Some(TYPE_ANY),
        )),
        TYPE_ERROR => Some(RuntimeTypeInfo::scalar(
            "error",
            RuntimeTypeKind::Interface,
            Some(TYPE_ERROR),
        )),
        TypeId(100) => Some(imported_interface_runtime_type_info(
            "context.Context",
            TypeId(100),
        )),
        TypeId(102) => Some(imported_interface_runtime_type_info("fs.FS", TypeId(102))),
        TypeId(103) => Some(imported_interface_runtime_type_info("fs.File", TypeId(103))),
        TypeId(104) => Some(imported_interface_runtime_type_info(
            "fs.FileInfo",
            TypeId(104),
        )),
        TypeId(105) => Some(imported_interface_runtime_type_info(
            "fs.DirEntry",
            TypeId(105),
        )),
        TypeId(106) => Some(imported_interface_runtime_type_info(
            "fs.ReadDirFile",
            TypeId(106),
        )),
        TypeId(107) => Some(imported_interface_runtime_type_info(
            "fs.ReadFileFS",
            TypeId(107),
        )),
        TypeId(108) => Some(imported_interface_runtime_type_info(
            "fs.StatFS",
            TypeId(108),
        )),
        TypeId(109) => Some(imported_interface_runtime_type_info(
            "fs.ReadDirFS",
            TypeId(109),
        )),
        TypeId(110) => Some(imported_interface_runtime_type_info(
            "fs.GlobFS",
            TypeId(110),
        )),
        TypeId(111) => Some(imported_interface_runtime_type_info(
            "fs.SubFS",
            TypeId(111),
        )),
        TypeId(112) => Some(imported_interface_runtime_type_info(
            "io.Reader",
            TypeId(112),
        )),
        TypeId(113) => Some(imported_interface_runtime_type_info(
            "io.Closer",
            TypeId(113),
        )),
        TypeId(114) => Some(imported_interface_runtime_type_info(
            "io.ReadCloser",
            TypeId(114),
        )),
        TYPE_ARRAY => Some(generic_runtime_type_info(
            "array",
            RuntimeTypeKind::Array,
            TYPE_ARRAY,
        )),
        TYPE_SLICE => Some(generic_runtime_type_info(
            "slice",
            RuntimeTypeKind::Slice,
            TYPE_SLICE,
        )),
        TYPE_MAP => Some(generic_runtime_type_info(
            "map",
            RuntimeTypeKind::Map,
            TYPE_MAP,
        )),
        TYPE_POINTER => Some(generic_runtime_type_info(
            "pointer",
            RuntimeTypeKind::Pointer,
            TYPE_POINTER,
        )),
        TYPE_FUNCTION => Some(generic_runtime_type_info(
            "function",
            RuntimeTypeKind::Function,
            TYPE_FUNCTION,
        )),
        TYPE_CHANNEL => Some(generic_runtime_type_info(
            "channel",
            RuntimeTypeKind::Channel,
            TYPE_CHANNEL,
        )),
        _ => None,
    }
}

fn imported_interface_runtime_type_info(display_name: &str, type_id: TypeId) -> RuntimeTypeInfo {
    RuntimeTypeInfo {
        display_name: display_name.into(),
        package_path: display_name
            .split_once('.')
            .map(|(package_path, _)| package_path.into()),
        kind: RuntimeTypeKind::Interface,
        type_id: Some(type_id),
        fields: Vec::new(),
        elem: None,
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: None,
        channel_direction: None,
    }
}

fn generic_runtime_type_info(
    display_name: &str,
    kind: RuntimeTypeKind,
    type_id: TypeId,
) -> RuntimeTypeInfo {
    RuntimeTypeInfo {
        display_name: display_name.to_string(),
        package_path: None,
        kind,
        type_id: Some(type_id),
        fields: Vec::new(),
        elem: None,
        key: None,
        len: None,
        params: Vec::new(),
        results: Vec::new(),
        underlying: None,
        channel_direction: None,
    }
}

fn qualified_package_path(typ: &str) -> Option<String> {
    let head = typ.strip_prefix('*').unwrap_or(typ);
    let head = head.split('[').next().unwrap_or(head);
    let qualifier = head.rsplit_once('.')?.0;
    Some(
        match qualifier {
            "base64" => "encoding/base64",
            "fs" => "io/fs",
            "http" => "net/http",
            "url" => "net/url",
            other => other,
        }
        .to_string(),
    )
}

fn runtime_channel_direction(direction: ChannelDirection) -> RuntimeChannelDirection {
    match direction {
        ChannelDirection::Bidirectional => RuntimeChannelDirection::Bidirectional,
        ChannelDirection::SendOnly => RuntimeChannelDirection::SendOnly,
        ChannelDirection::ReceiveOnly => RuntimeChannelDirection::ReceiveOnly,
    }
}
