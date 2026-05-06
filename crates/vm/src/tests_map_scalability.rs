use super::value::{format_value, set_index_value, SetIndexError};
use super::{
    ConcreteType, PointerTarget, PointerValue, TypeId, Value, ValueData, TYPE_INT, TYPE_POINTER,
};

fn typed_array_key(values: Vec<Value>) -> Value {
    Value::array_typed(
        values,
        ConcreteType::Array {
            len: 2,
            element: Box::new(ConcreteType::TypeId(TYPE_INT)),
        },
    )
}

fn struct_key(name: &str, count: i64) -> Value {
    Value::struct_value(
        TypeId(90_001),
        vec![
            ("name".into(), Value::string(name)),
            ("count".into(), Value::int(count)),
        ],
    )
}

fn pointer_key(global: usize) -> Value {
    Value {
        typ: TYPE_POINTER,
        data: ValueData::Pointer(PointerValue {
            target: PointerTarget::Global { global },
            concrete_type: None,
        }),
    }
}

#[test]
fn indexed_maps_lookup_supported_key_shapes() {
    let keys = vec![
        Value::int(7),
        Value::float(-3.5),
        Value::string("gowasm"),
        Value::bool(true),
        typed_array_key(vec![Value::int(1), Value::int(2)]),
        struct_key("compiler", 2),
        pointer_key(11),
        Value::channel(21),
        Value::function(3, vec![Value::string("capture")]),
        Value::wrapped_error_with_kind("boom", "kind", Value::int(4)),
    ];

    let entries: Vec<(Value, Value)> = keys
        .iter()
        .enumerate()
        .map(|(index, key)| (key.clone(), Value::string(format!("value-{index}"))))
        .collect();
    let map = Value::map(entries, Value::string(""));
    let ValueData::Map(map) = &map.data else {
        unreachable!("Value::map should produce a map");
    };

    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            map.get(key),
            Some(Value::string(format!("value-{index}"))),
            "key index {index} should resolve through the map index",
        );
        assert!(map.contains_key(key), "key index {index} should exist");
    }

    assert!(!map.contains_key(&Value::int(999)));
    assert!(map.get(&struct_key("compiler", 3)).is_none());
}

#[test]
fn indexed_maps_preserve_insertion_order_for_large_updates() {
    let mut map = Value::map(vec![], Value::string(""));

    for index in 0..2048 {
        set_index_value(
            &mut map,
            &Value::int(index),
            Value::string(format!("value-{index}")),
        )
        .expect("large map insert should succeed");
    }

    for index in (0..2048).step_by(3) {
        set_index_value(
            &mut map,
            &Value::int(index),
            Value::string(format!("updated-{index}")),
        )
        .expect("large map update should succeed");
    }

    let ValueData::Map(map_value) = &map.data else {
        unreachable!("Value::map should produce a map");
    };
    let entries = map_value.entries_snapshot();

    assert_eq!(entries.len(), 2048);
    assert_eq!(entries[0].0, Value::int(0));
    assert_eq!(entries[1].0, Value::int(1));
    assert_eq!(entries[2].0, Value::int(2));
    assert_eq!(entries[2047].0, Value::int(2047));
    assert_eq!(
        map_value.get(&Value::int(0)),
        Some(Value::string("updated-0"))
    );
    assert_eq!(
        map_value.get(&Value::int(1)),
        Some(Value::string("value-1"))
    );
    assert_eq!(
        map_value.get(&Value::int(1536)),
        Some(Value::string("updated-1536"))
    );
    assert_eq!(
        map_value.get(&Value::int(2047)),
        Some(Value::string("value-2047"))
    );

    let rendered = format_value(&map);
    assert!(rendered.starts_with("map[0:updated-0 1:value-1 2:value-2"));
}

#[test]
fn indexed_maps_handle_dense_string_prefix_workloads() {
    let mut map = Value::map(vec![], Value::int(-1));

    for index in 0..1536 {
        let key = Value::string(format!("prefix/segment/shared/{:04}-{}", index % 64, index));
        set_index_value(&mut map, &key, Value::int(index)).expect("insert should succeed");
    }

    for index in (0..1536).rev().step_by(11) {
        let key = Value::string(format!("prefix/segment/shared/{:04}-{}", index % 64, index));
        let ValueData::Map(map_value) = &map.data else {
            unreachable!("Value::map should produce a map");
        };
        assert_eq!(map_value.get(&key), Some(Value::int(index)));
        assert!(map_value.contains_key(&key));
    }

    let mut nil_map = Value::nil_map(Value::int(0));
    let result = set_index_value(&mut nil_map, &Value::int(1), Value::int(2));
    assert_eq!(result, Err(SetIndexError::NilMap));
}
