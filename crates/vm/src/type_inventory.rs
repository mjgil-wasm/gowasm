use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::{
    Program, TypeId, Value, ValueData, Vm, TYPE_ARRAY, TYPE_CHANNEL, TYPE_FUNCTION,
    TYPE_HTTP_REQUEST_BODY, TYPE_HTTP_RESPONSE_BODY, TYPE_MAP, TYPE_NIL, TYPE_POINTER, TYPE_SLICE,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeTypeKind {
    Nil,
    Int,
    Float64,
    String,
    Bool,
    Array,
    Slice,
    Map,
    Struct,
    Interface,
    Pointer,
    Function,
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeChannelDirection {
    Bidirectional,
    SendOnly,
    ReceiveOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcreteType {
    TypeId(TypeId),
    Array {
        len: usize,
        element: Box<ConcreteType>,
    },
    Slice {
        element: Box<ConcreteType>,
    },
    Map {
        key: Box<ConcreteType>,
        value: Box<ConcreteType>,
    },
    Pointer {
        element: Box<ConcreteType>,
    },
    Function {
        params: Vec<ConcreteType>,
        results: Vec<ConcreteType>,
    },
    Channel {
        direction: RuntimeChannelDirection,
        element: Box<ConcreteType>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTypeField {
    pub name: String,
    pub typ: ConcreteType,
    pub embedded: bool,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTypeInfo {
    pub display_name: String,
    pub package_path: Option<String>,
    pub kind: RuntimeTypeKind,
    pub type_id: Option<TypeId>,
    pub fields: Vec<RuntimeTypeField>,
    pub elem: Option<Box<ConcreteType>>,
    pub key: Option<Box<ConcreteType>>,
    pub len: Option<usize>,
    pub params: Vec<ConcreteType>,
    pub results: Vec<ConcreteType>,
    pub underlying: Option<Box<ConcreteType>>,
    pub channel_direction: Option<RuntimeChannelDirection>,
}

impl RuntimeTypeInfo {
    pub fn scalar(
        display_name: impl Into<String>,
        kind: RuntimeTypeKind,
        type_id: Option<TypeId>,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            package_path: None,
            kind,
            type_id,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProgramTypeInventory {
    pub types_by_id: HashMap<TypeId, RuntimeTypeInfo>,
}

impl ProgramTypeInventory {
    pub fn register(&mut self, info: RuntimeTypeInfo) {
        if let Some(type_id) = info.type_id {
            self.types_by_id.insert(type_id, info);
        }
    }

    pub fn type_info_for_type_id(&self, type_id: TypeId) -> Option<RuntimeTypeInfo> {
        self.types_by_id.get(&type_id).cloned()
    }

    pub fn resolve_concrete_type(&self, typ: &ConcreteType) -> Option<RuntimeTypeInfo> {
        match typ {
            ConcreteType::TypeId(type_id) => self.type_info_for_type_id(*type_id),
            ConcreteType::Array { len, element } => Some(RuntimeTypeInfo {
                display_name: format!("[{len}]{}", self.display_concrete_type(element)),
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
            ConcreteType::Slice { element } => Some(RuntimeTypeInfo {
                display_name: format!("[]{}", self.display_concrete_type(element)),
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
            ConcreteType::Map { key, value } => Some(RuntimeTypeInfo {
                display_name: format!(
                    "map[{}]{}",
                    self.display_concrete_type(key),
                    self.display_concrete_type(value)
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
            ConcreteType::Pointer { element } => Some(RuntimeTypeInfo {
                display_name: format!("*{}", self.display_concrete_type(element)),
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
            ConcreteType::Function { params, results } => Some(RuntimeTypeInfo {
                display_name: format!(
                    "__gowasm_func__({})->({})",
                    params
                        .iter()
                        .map(|typ| self.display_concrete_type(typ))
                        .collect::<Vec<_>>()
                        .join(","),
                    results
                        .iter()
                        .map(|typ| self.display_concrete_type(typ))
                        .collect::<Vec<_>>()
                        .join(",")
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
                Some(RuntimeTypeInfo {
                    display_name: format!("{prefix}{}", self.display_concrete_type(element)),
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

    pub fn value_type_info(
        &self,
        vm: &Vm,
        program: &Program,
        value: &Value,
    ) -> Option<RuntimeTypeInfo> {
        if matches!(value.typ, TYPE_NIL) && matches!(value.data, ValueData::Nil) {
            return Some(RuntimeTypeInfo::scalar("nil", RuntimeTypeKind::Nil, None));
        }
        if !matches!(value.data, ValueData::Nil) && type_id_is_interface(self, value.typ) {
            if let Some(concrete_type) = concrete_type_for_value(vm, program, value) {
                if let Some(info) = self.resolve_concrete_type(&concrete_type) {
                    return Some(info);
                }
            }
        }
        if prefers_value_type_id(value.typ) {
            if let Some(info) = self.type_info_for_type_id(value.typ) {
                return Some(info);
            }
        }
        if let Some(concrete_type) = concrete_type_for_value(vm, program, value) {
            return self.resolve_concrete_type(&concrete_type);
        }
        self.type_info_for_type_id(value.typ)
    }

    fn display_concrete_type(&self, typ: &ConcreteType) -> String {
        self.resolve_concrete_type(typ)
            .map(|info| info.display_name)
            .unwrap_or_else(|| "<unknown>".into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProgramTypeKey {
    functions_ptr: usize,
    function_count: usize,
}

fn registry() -> &'static Mutex<HashMap<ProgramTypeKey, Arc<ProgramTypeInventory>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<ProgramTypeKey, Arc<ProgramTypeInventory>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn type_key(program: &Program) -> ProgramTypeKey {
    ProgramTypeKey {
        functions_ptr: program.functions.as_ptr() as usize,
        function_count: program.functions.len(),
    }
}

pub fn register_program_type_inventory(program: &Program, inventory: ProgramTypeInventory) {
    registry()
        .lock()
        .expect("program type registry lock should not be poisoned")
        .insert(type_key(program), Arc::new(inventory));
}

pub fn program_type_inventory(program: &Program) -> Option<ProgramTypeInventory> {
    lookup_program_type_inventory(program).map(|inventory| (*inventory).clone())
}

pub fn value_runtime_type(program: &Program, vm: &Vm, value: &Value) -> Option<RuntimeTypeInfo> {
    lookup_program_type_inventory(program)?.value_type_info(vm, program, value)
}

fn lookup_program_type_inventory(program: &Program) -> Option<Arc<ProgramTypeInventory>> {
    registry()
        .lock()
        .expect("program type registry lock should not be poisoned")
        .get(&type_key(program))
        .cloned()
}

pub(crate) fn explicit_concrete_type_for_value(value: &Value) -> Option<ConcreteType> {
    match &value.data {
        ValueData::Array(array) => array.concrete_type.clone(),
        ValueData::Slice(slice) => slice.concrete_type.clone(),
        ValueData::Map(map) => map.concrete_type.clone(),
        ValueData::Channel(channel) => channel.concrete_type.clone(),
        ValueData::Function(function) => function.concrete_type.clone(),
        ValueData::Pointer(pointer) => pointer.concrete_type.clone(),
        ValueData::Struct(_)
        | ValueData::Int(_)
        | ValueData::Float(_)
        | ValueData::String(_)
        | ValueData::Bool(_)
        | ValueData::Error(_) => Some(ConcreteType::TypeId(value.typ)),
        ValueData::Nil => (value.typ != TYPE_NIL).then_some(ConcreteType::TypeId(value.typ)),
    }
}

pub(crate) fn concrete_type_for_value(
    vm: &Vm,
    program: &Program,
    value: &Value,
) -> Option<ConcreteType> {
    if program_type_inventory(program)
        .is_some_and(|inventory| type_id_is_interface(&inventory, value.typ))
    {
        if let Some(hidden_type) = interface_hidden_concrete_type(value) {
            return Some(hidden_type);
        }
        match &value.data {
            ValueData::Int(_) => return Some(ConcreteType::TypeId(crate::TYPE_INT)),
            ValueData::Float(_) => return Some(ConcreteType::TypeId(crate::TYPE_FLOAT64)),
            ValueData::String(_) => return Some(ConcreteType::TypeId(crate::TYPE_STRING)),
            ValueData::Bool(_) => return Some(ConcreteType::TypeId(crate::TYPE_BOOL)),
            ValueData::Error(_) => return Some(ConcreteType::TypeId(crate::TYPE_ERROR)),
            _ => {}
        }
    }
    explicit_concrete_type_for_value(value).or_else(|| match &value.data {
        ValueData::Pointer(pointer) => (!pointer.is_nil())
            .then(|| vm.deref_pointer(program, value).ok())
            .flatten()
            .and_then(|value| concrete_type_for_value(vm, program, &value))
            .map(|element| ConcreteType::Pointer {
                element: Box::new(element),
            }),
        _ => None,
    })
}

fn prefers_value_type_id(type_id: TypeId) -> bool {
    !matches!(
        type_id,
        TYPE_ARRAY | TYPE_SLICE | TYPE_MAP | TYPE_POINTER | TYPE_FUNCTION | TYPE_CHANNEL
    )
}

fn type_id_is_interface(inventory: &ProgramTypeInventory, type_id: TypeId) -> bool {
    inventory
        .type_info_for_type_id(type_id)
        .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
}

fn interface_hidden_concrete_type(value: &Value) -> Option<ConcreteType> {
    let ValueData::Struct(fields) = &value.data else {
        return None;
    };
    if fields
        .iter()
        .any(|(name, _)| name == "__http_request_body_id")
    {
        return Some(ConcreteType::TypeId(TYPE_HTTP_REQUEST_BODY));
    }
    if fields
        .iter()
        .any(|(name, _)| name == "__http_response_body_id")
    {
        return Some(ConcreteType::TypeId(TYPE_HTTP_RESPONSE_BODY));
    }
    None
}
