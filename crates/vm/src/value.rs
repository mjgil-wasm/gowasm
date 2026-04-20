use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::CompareOp;
use crate::map_value::{SharedMapEntries, SharedMapIndex};
use crate::ConcreteType;

#[derive(Debug, Clone, Copy)]
pub struct Float64(pub f64);

impl Serialize for Float64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0.to_bits())
    }
}

impl<'de> Deserialize<'de> for Float64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(f64::from_bits(u64::deserialize(deserializer)?)))
    }
}

impl PartialEq for Float64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Float64 {}

impl std::fmt::Display for Float64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.0;
        if v.is_nan() {
            return write!(f, "NaN");
        }
        if v.is_infinite() {
            return if v.is_sign_positive() {
                write!(f, "+Inf")
            } else {
                write!(f, "-Inf")
            };
        }
        if v == v.trunc() && v.is_finite() {
            write!(f, "{v:.1}")
        } else {
            write!(f, "{v}")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayValue {
    pub values: Rc<RefCell<Vec<Value>>>,
    pub concrete_type: Option<ConcreteType>,
}

impl Clone for ArrayValue {
    fn clone(&self) -> Self {
        Self {
            values: Rc::new(RefCell::new(self.values.borrow().clone())),
            concrete_type: self.concrete_type.clone(),
        }
    }
}

impl ArrayValue {
    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }

    pub fn values_snapshot(&self) -> Vec<Value> {
        self.values.borrow().clone()
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        self.values.borrow().get(index).cloned()
    }

    pub fn set(&self, index: usize, value: Value) -> bool {
        let mut values = self.values.borrow_mut();
        let Some(slot) = values.get_mut(index) else {
            return false;
        };
        *slot = value;
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceValue {
    pub values: Rc<RefCell<Vec<Value>>>,
    pub start: usize,
    pub len: usize,
    pub cap: usize,
    pub is_nil: bool,
    pub concrete_type: Option<ConcreteType>,
}

impl SliceValue {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn values_snapshot(&self) -> Vec<Value> {
        let values = self.values.borrow();
        values[self.start..self.start + self.len].to_vec()
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        if index >= self.len {
            return None;
        }
        self.values.borrow().get(self.start + index).cloned()
    }

    pub fn set(&self, index: usize, value: Value) -> bool {
        if index >= self.len {
            return false;
        }
        let mut values = self.values.borrow_mut();
        let Some(slot) = values.get_mut(self.start + index) else {
            return false;
        };
        *slot = value;
        true
    }

    pub fn subslice(&self, low: usize, high: usize) -> Self {
        Self {
            values: self.values.clone(),
            start: self.start + low,
            len: high - low,
            cap: self.cap - low,
            is_nil: false,
            concrete_type: self.concrete_type.clone(),
        }
    }

    pub fn appended(&self, appended: &[Value]) -> Self {
        let new_len = self.len + appended.len();
        if new_len <= self.cap {
            let mut values = self.values.borrow_mut();
            let write_start = self.start + self.len;
            for (offset, value) in appended.iter().enumerate() {
                let index = write_start + offset;
                if index < values.len() {
                    values[index] = value.clone();
                } else {
                    values.push(value.clone());
                }
            }
            drop(values);
            return Self {
                values: self.values.clone(),
                start: self.start,
                len: new_len,
                cap: self.cap,
                is_nil: false,
                concrete_type: self.concrete_type.clone(),
            };
        }

        let mut values = self.values_snapshot();
        values.extend(appended.iter().cloned());
        let old_cap = self.cap.max(self.len);
        let new_cap = if old_cap == 0 {
            new_len
        } else {
            (old_cap * 2).max(new_len)
        };
        Self {
            values: Rc::new(RefCell::new(values)),
            start: 0,
            len: new_len,
            cap: new_cap,
            is_nil: false,
            concrete_type: self.concrete_type.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorValue {
    pub message: String,
    pub kind_message: Option<String>,
    pub wrapped: Option<Box<Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapValue {
    pub(crate) entries: Option<SharedMapEntries>,
    pub(crate) index: Option<SharedMapIndex>,
    pub zero_value: Box<Value>,
    pub concrete_type: Option<ConcreteType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerValue {
    pub target: PointerTarget,
    pub concrete_type: Option<ConcreteType>,
}

impl PointerValue {
    pub fn is_nil(&self) -> bool {
        matches!(&self.target, PointerTarget::Nil)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionValue {
    pub function: usize,
    pub captures: Vec<Value>,
    pub concrete_type: Option<ConcreteType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelValue {
    pub id: Option<u64>,
    pub concrete_type: Option<ConcreteType>,
}

impl ChannelValue {
    pub fn is_nil(&self) -> bool {
        self.id.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerTarget {
    Nil,
    HeapCell {
        cell: usize,
    },
    Local {
        frame_id: u64,
        register: usize,
    },
    Global {
        global: usize,
    },
    ProjectedField {
        base: Box<PointerTarget>,
        field: String,
    },
    ProjectedIndex {
        base: Box<PointerTarget>,
        index: Box<Value>,
    },
    LocalField {
        frame_id: u64,
        register: usize,
        field: String,
    },
    GlobalField {
        global: usize,
        field: String,
    },
    LocalIndex {
        frame_id: u64,
        register: usize,
        index: Box<Value>,
    },
    GlobalIndex {
        global: usize,
        index: Box<Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueData {
    Nil,
    Int(i64),
    Float(Float64),
    String(String),
    Bool(bool),
    Error(ErrorValue),
    Array(ArrayValue),
    Slice(SliceValue),
    Map(MapValue),
    Channel(ChannelValue),
    Pointer(PointerValue),
    Function(FunctionValue),
    Struct(Vec<(String, Value)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub typ: super::TypeId,
    pub data: ValueData,
}

impl Value {
    pub fn nil() -> Self {
        Self {
            typ: super::TYPE_NIL,
            data: ValueData::Nil,
        }
    }

    pub fn int(value: i64) -> Self {
        Self {
            typ: super::TYPE_INT,
            data: ValueData::Int(value),
        }
    }

    pub fn float(value: f64) -> Self {
        Self {
            typ: super::TYPE_FLOAT64,
            data: ValueData::Float(Float64(value)),
        }
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self {
            typ: super::TYPE_STRING,
            data: ValueData::String(value.into()),
        }
    }

    pub fn bool(value: bool) -> Self {
        Self {
            typ: super::TYPE_BOOL,
            data: ValueData::Bool(value),
        }
    }

    pub fn error(value: impl Into<String>) -> Self {
        Self {
            typ: super::TYPE_ERROR,
            data: ValueData::Error(ErrorValue {
                message: value.into(),
                kind_message: None,
                wrapped: None,
            }),
        }
    }

    pub fn error_with_kind(message: impl Into<String>, kind_message: impl Into<String>) -> Self {
        Self {
            typ: super::TYPE_ERROR,
            data: ValueData::Error(ErrorValue {
                message: message.into(),
                kind_message: Some(kind_message.into()),
                wrapped: None,
            }),
        }
    }

    pub fn wrapped_error(message: String, wrapped: Value) -> Self {
        Self {
            typ: super::TYPE_ERROR,
            data: ValueData::Error(ErrorValue {
                message,
                kind_message: None,
                wrapped: Some(Box::new(wrapped)),
            }),
        }
    }

    pub fn wrapped_error_with_kind(
        message: impl Into<String>,
        kind_message: impl Into<String>,
        wrapped: Value,
    ) -> Self {
        Self {
            typ: super::TYPE_ERROR,
            data: ValueData::Error(ErrorValue {
                message: message.into(),
                kind_message: Some(kind_message.into()),
                wrapped: Some(Box::new(wrapped)),
            }),
        }
    }

    pub fn array(values: Vec<Value>) -> Self {
        Self {
            typ: super::TYPE_ARRAY,
            data: ValueData::Array(ArrayValue {
                values: Rc::new(RefCell::new(values)),
                concrete_type: None,
            }),
        }
    }

    pub fn array_typed(values: Vec<Value>, concrete_type: ConcreteType) -> Self {
        Self {
            typ: super::TYPE_ARRAY,
            data: ValueData::Array(ArrayValue {
                values: Rc::new(RefCell::new(values)),
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn slice(values: Vec<Value>) -> Self {
        let cap = values.len();
        Self::slice_with_cap(values, cap)
    }

    pub fn slice_typed(values: Vec<Value>, concrete_type: ConcreteType) -> Self {
        let cap = values.len();
        Self::slice_with_cap_typed(values, cap, concrete_type)
    }

    pub fn slice_with_cap(values: Vec<Value>, cap: usize) -> Self {
        assert!(cap >= values.len(), "slice capacity must be >= length");
        let len = values.len();
        Self {
            typ: super::TYPE_SLICE,
            data: ValueData::Slice(SliceValue {
                values: Rc::new(RefCell::new(values)),
                start: 0,
                len,
                cap,
                is_nil: false,
                concrete_type: None,
            }),
        }
    }

    pub fn slice_with_cap_typed(
        values: Vec<Value>,
        cap: usize,
        concrete_type: ConcreteType,
    ) -> Self {
        assert!(cap >= values.len(), "slice capacity must be >= length");
        let len = values.len();
        Self {
            typ: super::TYPE_SLICE,
            data: ValueData::Slice(SliceValue {
                values: Rc::new(RefCell::new(values)),
                start: 0,
                len,
                cap,
                is_nil: false,
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn nil_slice() -> Self {
        Self {
            typ: super::TYPE_SLICE,
            data: ValueData::Slice(SliceValue {
                values: Rc::new(RefCell::new(Vec::new())),
                start: 0,
                len: 0,
                cap: 0,
                is_nil: true,
                concrete_type: None,
            }),
        }
    }

    pub fn nil_slice_typed(concrete_type: ConcreteType) -> Self {
        Self {
            typ: super::TYPE_SLICE,
            data: ValueData::Slice(SliceValue {
                values: Rc::new(RefCell::new(Vec::new())),
                start: 0,
                len: 0,
                cap: 0,
                is_nil: true,
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn map(entries: Vec<(Value, Value)>, zero_value: Value) -> Self {
        Self {
            typ: super::TYPE_MAP,
            data: ValueData::Map(MapValue::with_entries(entries, zero_value, None)),
        }
    }

    pub fn map_typed(
        entries: Vec<(Value, Value)>,
        zero_value: Value,
        concrete_type: ConcreteType,
    ) -> Self {
        Self {
            typ: super::TYPE_MAP,
            data: ValueData::Map(MapValue::with_entries(
                entries,
                zero_value,
                Some(concrete_type),
            )),
        }
    }

    pub fn nil_map(zero_value: Value) -> Self {
        Self {
            typ: super::TYPE_MAP,
            data: ValueData::Map(MapValue::nil(zero_value, None)),
        }
    }

    pub fn nil_map_typed(zero_value: Value, concrete_type: ConcreteType) -> Self {
        Self {
            typ: super::TYPE_MAP,
            data: ValueData::Map(MapValue::nil(zero_value, Some(concrete_type))),
        }
    }

    pub fn nil_pointer(typ: super::TypeId) -> Self {
        Self {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::Nil,
                concrete_type: None,
            }),
        }
    }

    pub fn nil_pointer_typed(typ: super::TypeId, concrete_type: ConcreteType) -> Self {
        Self {
            typ,
            data: ValueData::Pointer(PointerValue {
                target: PointerTarget::Nil,
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn nil_channel() -> Self {
        Self {
            typ: super::TYPE_CHANNEL,
            data: ValueData::Channel(ChannelValue {
                id: None,
                concrete_type: None,
            }),
        }
    }

    pub fn nil_channel_typed(concrete_type: ConcreteType) -> Self {
        Self {
            typ: super::TYPE_CHANNEL,
            data: ValueData::Channel(ChannelValue {
                id: None,
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn channel(id: u64) -> Self {
        Self {
            typ: super::TYPE_CHANNEL,
            data: ValueData::Channel(ChannelValue {
                id: Some(id),
                concrete_type: None,
            }),
        }
    }

    pub fn channel_typed(id: u64, concrete_type: ConcreteType) -> Self {
        Self {
            typ: super::TYPE_CHANNEL,
            data: ValueData::Channel(ChannelValue {
                id: Some(id),
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn function(function: usize, captures: Vec<Value>) -> Self {
        Self {
            typ: super::TYPE_FUNCTION,
            data: ValueData::Function(FunctionValue {
                function,
                captures,
                concrete_type: None,
            }),
        }
    }

    pub fn function_typed(
        function: usize,
        captures: Vec<Value>,
        concrete_type: ConcreteType,
    ) -> Self {
        Self {
            typ: super::TYPE_FUNCTION,
            data: ValueData::Function(FunctionValue {
                function,
                captures,
                concrete_type: Some(concrete_type),
            }),
        }
    }

    pub fn struct_value(typ: super::TypeId, fields: Vec<(String, Value)>) -> Self {
        Self {
            typ,
            data: ValueData::Struct(fields),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SetFieldError {
    InvalidTarget,
    UnknownField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SetIndexError {
    InvalidTarget,
    InvalidIndexValue,
    IndexOutOfBounds { index: i64, len: usize },
    NilMap,
}

pub(crate) fn format_value(value: &Value) -> String {
    if value.typ == super::TYPE_SYNC_WAIT_GROUP
        || value.typ == super::TYPE_SYNC_ONCE
        || value.typ == super::TYPE_SYNC_MUTEX
        || value.typ == super::TYPE_SYNC_RW_MUTEX
        || value.typ == super::TYPE_CONTEXT
    {
        return "{}".into();
    }
    if value.typ == super::TYPE_TIME_TIMER {
        if let ValueData::Struct(fields) = &value.data {
            return format_timer_struct(fields);
        }
        return "{}".into();
    }
    match &value.data {
        ValueData::Nil => "<nil>".into(),
        ValueData::Int(number) => number.to_string(),
        ValueData::Float(f) => f.to_string(),
        ValueData::String(text) => text.clone(),
        ValueData::Bool(value) => value.to_string(),
        ValueData::Error(err) => err.message.clone(),
        ValueData::Array(array) => format_collection(&array.values_snapshot()),
        ValueData::Slice(slice) => format_collection(&slice.values_snapshot()),
        ValueData::Map(map) => format_map(map),
        ValueData::Channel(channel) => {
            if channel.is_nil() {
                "<nil>".into()
            } else {
                "<chan>".into()
            }
        }
        ValueData::Pointer(pointer) => {
            if pointer.is_nil() {
                "<nil>".into()
            } else {
                "<pointer>".into()
            }
        }
        ValueData::Function(_) => "<func>".into(),
        ValueData::Struct(fields) => format_struct(fields),
    }
}

pub(crate) fn describe_value(value: &Value) -> String {
    format!(
        "{} value `{}`",
        value_kind(value),
        render_value_for_error(value)
    )
}

fn render_value_for_error(value: &Value) -> String {
    match &value.data {
        ValueData::String(text) => format!("{text:?}"),
        _ => format_value(value),
    }
}

fn value_kind(value: &Value) -> &'static str {
    match &value.data {
        ValueData::Nil => "nil",
        ValueData::Int(_) => "int",
        ValueData::Float(_) => "float64",
        ValueData::String(_) => "string",
        ValueData::Bool(_) => "bool",
        ValueData::Error(_) => "error",
        ValueData::Array(_) => "array",
        ValueData::Slice(_) => "slice",
        ValueData::Map(_) => "map",
        ValueData::Channel(_) => "channel",
        ValueData::Pointer(_) => "pointer",
        ValueData::Function(_) => "function",
        ValueData::Struct(_) => "struct",
    }
}

pub(crate) fn compare_eq(equal: bool, op: CompareOp) -> Option<bool> {
    match op {
        CompareOp::Equal => Some(equal),
        CompareOp::NotEqual => Some(!equal),
        CompareOp::Less | CompareOp::LessEqual | CompareOp::Greater | CompareOp::GreaterEqual => {
            None
        }
    }
}

pub(crate) fn compare_ord(ordering: std::cmp::Ordering, op: CompareOp) -> Option<bool> {
    match op {
        CompareOp::Equal => Some(ordering == std::cmp::Ordering::Equal),
        CompareOp::NotEqual => Some(ordering != std::cmp::Ordering::Equal),
        CompareOp::Less => Some(ordering == std::cmp::Ordering::Less),
        CompareOp::LessEqual => Some(ordering != std::cmp::Ordering::Greater),
        CompareOp::Greater => Some(ordering == std::cmp::Ordering::Greater),
        CompareOp::GreaterEqual => Some(ordering != std::cmp::Ordering::Less),
    }
}

pub(crate) fn compare_op_symbol(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Equal => "==",
        CompareOp::NotEqual => "!=",
        CompareOp::Less => "<",
        CompareOp::LessEqual => "<=",
        CompareOp::Greater => ">",
        CompareOp::GreaterEqual => ">=",
    }
}

pub(crate) fn set_struct_field(
    target: &mut Value,
    field: &str,
    value: Value,
) -> Result<(), SetFieldError> {
    let ValueData::Struct(fields) = &mut target.data else {
        return Err(SetFieldError::InvalidTarget);
    };

    let slot = fields
        .iter_mut()
        .find(|(name, _)| name == field)
        .ok_or(SetFieldError::UnknownField)?;
    slot.1 = value;
    Ok(())
}

pub(crate) fn set_index_value(
    target: &mut Value,
    index: &Value,
    value: Value,
) -> Result<(), SetIndexError> {
    match &mut target.data {
        ValueData::Array(array) => {
            let ValueData::Int(index) = index.data else {
                return Err(SetIndexError::InvalidIndexValue);
            };
            if index < 0 {
                return Err(SetIndexError::IndexOutOfBounds { index, len: 0 });
            }
            let index = index as usize;
            let len = array.len();
            if !array.set(index, value) {
                return Err(SetIndexError::IndexOutOfBounds {
                    index: index as i64,
                    len,
                });
            }
            Ok(())
        }
        ValueData::Slice(slice) => {
            let ValueData::Int(index) = index.data else {
                return Err(SetIndexError::InvalidIndexValue);
            };
            if index < 0 {
                return Err(SetIndexError::IndexOutOfBounds { index, len: 0 });
            }
            let index = index as usize;
            let len = slice.len();
            if !slice.set(index, value) {
                return Err(SetIndexError::IndexOutOfBounds {
                    index: index as i64,
                    len,
                });
            }
            Ok(())
        }
        ValueData::Map(map) => {
            if !map.insert(index.clone(), value) {
                return Err(SetIndexError::NilMap);
            }
            Ok(())
        }
        _ => Err(SetIndexError::InvalidTarget),
    }
}

fn format_collection(values: &[Value]) -> String {
    let parts: Vec<String> = values.iter().map(format_value).collect();
    format!("[{}]", parts.join(" "))
}

fn format_map(map: &MapValue) -> String {
    let Some(entries) = &map.entries else {
        return "map[]".into();
    };
    let entries = entries.borrow();
    let parts: Vec<String> = entries
        .iter()
        .map(|(key, value)| format!("{}:{}", format_value(key), format_value(value)))
        .collect();
    format!("map[{}]", parts.join(" "))
}

fn format_struct(fields: &[(String, Value)]) -> String {
    let parts: Vec<String> = fields
        .iter()
        .map(|(_, value)| format_value(value))
        .collect();
    format!("{{{}}}", parts.join(" "))
}

fn format_timer_struct(fields: &[(String, Value)]) -> String {
    let Some((_, channel)) = fields.iter().find(|(name, _)| name == "C") else {
        return "{}".into();
    };
    format!("{{{}}}", format_value(channel))
}
