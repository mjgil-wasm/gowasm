use std::cell::RefCell;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::{
    value::ErrorValue, ChannelValue, ConcreteType, Float64, FunctionValue, MapValue, PointerTarget,
    PointerValue, RuntimeChannelDirection, TypeId, Value, ValueData,
};

pub(crate) type MapIndex = HashMap<u64, Vec<usize>>;
pub(crate) type SharedMapEntries = Rc<RefCell<Vec<(Value, Value)>>>;
pub(crate) type SharedMapIndex = Rc<RefCell<MapIndex>>;

pub(crate) fn map_key_is_comparable(value: &Value) -> bool {
    match &value.data {
        ValueData::Nil => false,
        ValueData::Int(_)
        | ValueData::Float(_)
        | ValueData::String(_)
        | ValueData::Bool(_)
        | ValueData::Pointer(_)
        | ValueData::Channel(_) => true,
        ValueData::Error(error) => match error.wrapped.as_deref() {
            Some(wrapped) => map_key_is_comparable(wrapped),
            None => true,
        },
        ValueData::Array(array) => array.values_snapshot().iter().all(map_key_is_comparable),
        ValueData::Struct(fields) => fields
            .iter()
            .all(|(_, field_value)| map_key_is_comparable(field_value)),
        ValueData::Slice(_) | ValueData::Map(_) | ValueData::Function(_) => false,
    }
}

impl MapValue {
    pub(crate) fn with_entries(
        entries: Vec<(Value, Value)>,
        zero_value: Value,
        concrete_type: Option<ConcreteType>,
    ) -> Self {
        let index = build_map_index(&entries);
        Self {
            entries: Some(Rc::new(RefCell::new(entries))),
            index: Some(Rc::new(RefCell::new(index))),
            zero_value: Box::new(zero_value),
            concrete_type,
        }
    }

    pub(crate) fn nil(zero_value: Value, concrete_type: Option<ConcreteType>) -> Self {
        Self {
            entries: None,
            index: None,
            zero_value: Box::new(zero_value),
            concrete_type,
        }
    }

    pub(crate) fn is_nil(&self) -> bool {
        self.entries.is_none()
    }

    pub(crate) fn len(&self) -> usize {
        self.entries
            .as_ref()
            .map(|entries| entries.borrow().len())
            .unwrap_or(0)
    }

    pub(crate) fn entries_snapshot(&self) -> Vec<(Value, Value)> {
        self.entries
            .as_ref()
            .map(|entries| entries.borrow().clone())
            .unwrap_or_default()
    }

    pub(crate) fn get(&self, key: &Value) -> Option<Value> {
        let entries = self.entries.as_ref()?;
        let index = self.index.as_ref()?;
        let entries = entries.borrow();
        let index = index.borrow();
        lookup_entry_index(&entries, &index, key).map(|entry_index| entries[entry_index].1.clone())
    }

    pub(crate) fn contains_key(&self, key: &Value) -> bool {
        self.entry_index(key).is_some()
    }

    pub(crate) fn entry_index(&self, key: &Value) -> Option<usize> {
        let entries = self.entries.as_ref()?;
        let index = self.index.as_ref()?;
        let entries = entries.borrow();
        let index = index.borrow();
        lookup_entry_index(&entries, &index, key)
    }

    pub(crate) fn insert(&self, key: Value, value: Value) -> bool {
        if self.entries.is_none() {
            return false;
        }
        if self.index.is_none() {
            return false;
        }

        let existing_index = {
            let entries = self.entries.as_ref().expect("entries should exist");
            let index = self.index.as_ref().expect("index should exist");
            let entries = entries.borrow();
            let index = index.borrow();
            lookup_entry_index(&entries, &index, &key)
        };
        if let Some(existing_index) = existing_index {
            let entries = self.entries.as_ref().expect("entries should exist");
            let mut entries = entries.borrow_mut();
            entries[existing_index].1 = value;
            return true;
        }

        let hash = map_key_hash(&key);
        let entries = self.entries.as_ref().expect("entries should exist");
        let mut entries = entries.borrow_mut();
        let entry_index = entries.len();
        entries.push((key, value));
        drop(entries);
        self.index
            .as_ref()
            .expect("index should exist")
            .borrow_mut()
            .entry(hash)
            .or_default()
            .push(entry_index);
        true
    }

    pub(crate) fn remove(&self, key: &Value) -> bool {
        if self.is_nil() {
            return false;
        }
        let Some(entry_index) = self.entry_index(key) else {
            return true;
        };
        let entries = self.entries.as_ref().expect("entries should exist");
        let mut entries = entries.borrow_mut();
        entries.remove(entry_index);
        let rebuilt = build_map_index(&entries);
        drop(entries);
        let index = self.index.as_ref().expect("index should exist");
        *index.borrow_mut() = rebuilt;
        true
    }

    pub(crate) fn clear(&self) -> bool {
        let Some(entries) = &self.entries else {
            return false;
        };
        entries.borrow_mut().clear();
        if let Some(index) = &self.index {
            index.borrow_mut().clear();
        }
        true
    }
}

pub(crate) fn build_map_index(entries: &[(Value, Value)]) -> MapIndex {
    let mut index: MapIndex = HashMap::with_capacity(entries.len());
    for (entry_index, (key, _)) in entries.iter().enumerate() {
        index
            .entry(map_key_hash(key))
            .or_default()
            .push(entry_index);
    }
    index
}

fn lookup_entry_index(entries: &[(Value, Value)], index: &MapIndex, key: &Value) -> Option<usize> {
    let hash = map_key_hash(key);
    index.get(&hash).and_then(|bucket| {
        bucket.iter().copied().find(|entry_index| {
            entries
                .get(*entry_index)
                .map(|(existing_key, _)| existing_key == key)
                .unwrap_or(false)
        })
    })
}

fn map_key_hash(value: &Value) -> u64 {
    let mut state = DefaultHasher::new();
    hash_value(&mut state, value);
    state.finish()
}

fn hash_value<H: Hasher>(state: &mut H, value: &Value) {
    hash_type_id(state, value.typ);
    match &value.data {
        ValueData::Nil => 0_u8.hash(state),
        ValueData::Int(number) => {
            1_u8.hash(state);
            number.hash(state);
        }
        ValueData::Float(number) => {
            2_u8.hash(state);
            hash_float64(state, *number);
        }
        ValueData::String(text) => {
            3_u8.hash(state);
            text.hash(state);
        }
        ValueData::Bool(boolean) => {
            4_u8.hash(state);
            boolean.hash(state);
        }
        ValueData::Error(error) => {
            5_u8.hash(state);
            hash_error_value(state, error);
        }
        ValueData::Array(array) => {
            6_u8.hash(state);
            hash_optional_concrete_type(state, array.concrete_type.as_ref());
            hash_values(state, &array.values_snapshot());
        }
        ValueData::Slice(slice) => {
            7_u8.hash(state);
            hash_optional_concrete_type(state, slice.concrete_type.as_ref());
            slice.cap.hash(state);
            slice.is_nil.hash(state);
            hash_values(state, &slice.values_snapshot());
        }
        ValueData::Map(map) => {
            8_u8.hash(state);
            hash_optional_concrete_type(state, map.concrete_type.as_ref());
            match &map.entries {
                Some(entries) => {
                    let entries = entries.borrow();
                    true.hash(state);
                    entries.len().hash(state);
                    for (key, value) in entries.iter() {
                        hash_value(state, key);
                        hash_value(state, value);
                    }
                }
                None => false.hash(state),
            }
            hash_value(state, &map.zero_value);
        }
        ValueData::Channel(channel) => {
            9_u8.hash(state);
            hash_channel_value(state, channel);
        }
        ValueData::Pointer(pointer) => {
            10_u8.hash(state);
            hash_pointer_value(state, pointer);
        }
        ValueData::Function(function) => {
            11_u8.hash(state);
            hash_function_value(state, function);
        }
        ValueData::Struct(fields) => {
            12_u8.hash(state);
            fields.len().hash(state);
            for (name, field_value) in fields {
                name.hash(state);
                hash_value(state, field_value);
            }
        }
    }
}

fn hash_values<H: Hasher>(state: &mut H, values: &[Value]) {
    values.len().hash(state);
    for value in values {
        hash_value(state, value);
    }
}

fn hash_error_value<H: Hasher>(state: &mut H, value: &ErrorValue) {
    value.message.hash(state);
    value.kind_message.hash(state);
    match &value.wrapped {
        Some(wrapped) => {
            true.hash(state);
            hash_value(state, wrapped);
        }
        None => false.hash(state),
    }
}

fn hash_channel_value<H: Hasher>(state: &mut H, value: &ChannelValue) {
    value.id.hash(state);
    hash_optional_concrete_type(state, value.concrete_type.as_ref());
}

fn hash_function_value<H: Hasher>(state: &mut H, value: &FunctionValue) {
    value.function.hash(state);
    hash_values(state, &value.captures);
    hash_optional_concrete_type(state, value.concrete_type.as_ref());
}

fn hash_pointer_value<H: Hasher>(state: &mut H, value: &PointerValue) {
    hash_pointer_target(state, &value.target);
    hash_optional_concrete_type(state, value.concrete_type.as_ref());
}

fn hash_pointer_target<H: Hasher>(state: &mut H, target: &PointerTarget) {
    match target {
        PointerTarget::Nil => 0_u8.hash(state),
        PointerTarget::HeapCell { cell } => {
            1_u8.hash(state);
            cell.hash(state);
        }
        PointerTarget::Local { frame_id, register } => {
            2_u8.hash(state);
            frame_id.hash(state);
            register.hash(state);
        }
        PointerTarget::Global { global } => {
            3_u8.hash(state);
            global.hash(state);
        }
        PointerTarget::ProjectedField { base, field } => {
            4_u8.hash(state);
            hash_pointer_target(state, base);
            field.hash(state);
        }
        PointerTarget::ProjectedIndex { base, index } => {
            5_u8.hash(state);
            hash_pointer_target(state, base);
            hash_value(state, index);
        }
        PointerTarget::LocalField {
            frame_id,
            register,
            field,
        } => {
            6_u8.hash(state);
            frame_id.hash(state);
            register.hash(state);
            field.hash(state);
        }
        PointerTarget::GlobalField { global, field } => {
            7_u8.hash(state);
            global.hash(state);
            field.hash(state);
        }
        PointerTarget::LocalIndex {
            frame_id,
            register,
            index,
        } => {
            8_u8.hash(state);
            frame_id.hash(state);
            register.hash(state);
            hash_value(state, index);
        }
        PointerTarget::GlobalIndex { global, index } => {
            9_u8.hash(state);
            global.hash(state);
            hash_value(state, index);
        }
    }
}

fn hash_optional_concrete_type<H: Hasher>(state: &mut H, concrete_type: Option<&ConcreteType>) {
    match concrete_type {
        Some(concrete_type) => {
            true.hash(state);
            hash_concrete_type(state, concrete_type);
        }
        None => false.hash(state),
    }
}

fn hash_concrete_type<H: Hasher>(state: &mut H, concrete_type: &ConcreteType) {
    match concrete_type {
        ConcreteType::TypeId(type_id) => {
            0_u8.hash(state);
            hash_type_id(state, *type_id);
        }
        ConcreteType::Array { len, element } => {
            1_u8.hash(state);
            len.hash(state);
            hash_concrete_type(state, element);
        }
        ConcreteType::Slice { element } => {
            2_u8.hash(state);
            hash_concrete_type(state, element);
        }
        ConcreteType::Map { key, value } => {
            3_u8.hash(state);
            hash_concrete_type(state, key);
            hash_concrete_type(state, value);
        }
        ConcreteType::Pointer { element } => {
            4_u8.hash(state);
            hash_concrete_type(state, element);
        }
        ConcreteType::Function { params, results } => {
            5_u8.hash(state);
            hash_concrete_types(state, params);
            hash_concrete_types(state, results);
        }
        ConcreteType::Channel { direction, element } => {
            6_u8.hash(state);
            hash_runtime_channel_direction(state, *direction);
            hash_concrete_type(state, element);
        }
    }
}

fn hash_concrete_types<H: Hasher>(state: &mut H, concrete_types: &[ConcreteType]) {
    concrete_types.len().hash(state);
    for concrete_type in concrete_types {
        hash_concrete_type(state, concrete_type);
    }
}

fn hash_type_id<H: Hasher>(state: &mut H, type_id: TypeId) {
    type_id.0.hash(state);
}

fn hash_float64<H: Hasher>(state: &mut H, number: Float64) {
    number.0.to_bits().hash(state);
}

fn hash_runtime_channel_direction<H: Hasher>(state: &mut H, direction: RuntimeChannelDirection) {
    match direction {
        RuntimeChannelDirection::Bidirectional => 0_u8.hash(state),
        RuntimeChannelDirection::SendOnly => 1_u8.hash(state),
        RuntimeChannelDirection::ReceiveOnly => 2_u8.hash(state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_maps_share_backing_storage() {
        let original = Value::map(vec![(Value::string("go"), Value::int(1))], Value::int(0));
        let alias = original.clone();

        let ValueData::Map(original_map) = &original.data else {
            unreachable!("Value::map should produce a map");
        };
        let ValueData::Map(alias_map) = &alias.data else {
            unreachable!("Value::map clone should stay a map");
        };

        assert!(alias_map.insert(Value::string("wasm"), Value::int(2)));
        assert_eq!(
            original_map.get(&Value::string("wasm")),
            Some(Value::int(2))
        );

        assert!(original_map.remove(&Value::string("go")));
        assert_eq!(alias_map.get(&Value::string("go")), None);
    }

    #[test]
    fn lookup_entry_index_scans_bucket_entries_by_equality() {
        let key = Value::string("go");
        let other = Value::string("wasm");
        let entries = vec![(key.clone(), Value::int(1)), (other, Value::int(2))];
        let mut index = HashMap::new();
        index.insert(map_key_hash(&key), vec![1, 0]);

        assert_eq!(lookup_entry_index(&entries, &index, &key), Some(0));
    }
}
