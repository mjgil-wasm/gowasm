use std::collections::BTreeMap;

use crate::{
    describe_value, ConcreteType, MapValue, Program, Value, ValueData, Vm, VmError, TYPE_URL_VALUES,
};

use super::{query_escape_component, query_unescape_component};

pub(super) fn url_values_get(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, key) = values_key_args(vm, program, "url.Values.Get", args)?;
    let Some(entry) = values_entry_value(values, key) else {
        return Ok(Value::string(String::new()));
    };
    let ValueData::Slice(slice) = &entry.data else {
        return Err(invalid_values_argument(
            vm,
            program,
            "url.Values.Get",
            "values stored as []string",
        )?);
    };
    let values = slice.values_snapshot();
    let Some(first) = values.first() else {
        return Ok(Value::string(String::new()));
    };
    let ValueData::String(text) = &first.data else {
        return Err(invalid_values_argument(
            vm,
            program,
            "url.Values.Get",
            "values stored as []string",
        )?);
    };
    Ok(Value::string(text.clone()))
}

pub(super) fn url_values_set(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, key, value) = values_key_value_args(vm, program, "url.Values.Set", args)?;
    if values.is_nil() {
        return Err(VmError::AssignToNilMap {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[0]),
        });
    }
    let lookup_key = Value::string(key.to_string());
    let inserted = values.insert(lookup_key, single_values_value(value));
    debug_assert!(inserted, "non-nil url.Values receiver should stay writable");
    Ok(args[0].clone())
}

pub(super) fn url_values_add(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, key, value) = values_key_value_args(vm, program, "url.Values.Add", args)?;
    if values.is_nil() {
        return Err(VmError::AssignToNilMap {
            function: vm.current_function_name(program)?,
            target: describe_value(&args[0]),
        });
    }
    let lookup_key = Value::string(key.to_string());
    let updated_value = if let Some(existing) = values.get(&lookup_key) {
        let ValueData::Slice(slice) = &existing.data else {
            return Err(invalid_values_argument(
                vm,
                program,
                "url.Values.Add",
                "values stored as []string",
            )?);
        };
        let mut items = slice.values_snapshot();
        items.push(Value::string(value.to_string()));
        Value::slice(items)
    } else {
        single_values_value(value)
    };
    let inserted = values.insert(lookup_key, updated_value);
    debug_assert!(inserted, "non-nil url.Values receiver should stay writable");
    Ok(args[0].clone())
}

pub(super) fn url_values_del(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, key) = values_key_args(vm, program, "url.Values.Del", args)?;
    if values.is_nil() {
        return Ok(args[0].clone());
    }
    let removed = values.remove(&Value::string(key.to_string()));
    debug_assert!(
        removed,
        "non-nil url.Values receiver delete should not fail"
    );
    Ok(args[0].clone())
}

pub(super) fn url_values_has(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (values, key) = values_key_args(vm, program, "url.Values.Has", args)?;
    Ok(Value::bool(values_entry_value(values, key).is_some()))
}

pub(crate) fn url_values_encoded_text(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
    value: &Value,
) -> Result<String, VmError> {
    let ValueData::Map(values) = &value.data else {
        return Err(invalid_values_argument(vm, program, builtin, expected)?);
    };
    encode_values_map(vm, program, builtin, expected, values)
}

pub(super) fn parse_query_values(text: &str) -> (Value, Option<String>) {
    let mut entries = BTreeMap::<String, Vec<String>>::new();
    let mut first_error = None;

    for pair in text.split('&') {
        if pair.is_empty() {
            continue;
        }
        if pair.bytes().any(|byte| byte == b';') {
            if first_error.is_none() {
                first_error = Some("invalid semicolon separator in query".into());
            }
            continue;
        }

        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = match query_unescape_component(raw_key) {
            Ok(key) => key,
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
                continue;
            }
        };
        let value = match query_unescape_component(raw_value) {
            Ok(value) => value,
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
                continue;
            }
        };
        entries.entry(key).or_default().push(value);
    }

    (url_values_value(entries), first_error)
}

fn invalid_values_argument(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
) -> Result<VmError, VmError> {
    Ok(VmError::InvalidStringFunctionArgument {
        function: vm.current_function_name(program)?,
        builtin: builtin.into(),
        expected: expected.into(),
    })
}

fn values_key_args<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a MapValue, &'a str), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let values = values_map(vm, program, builtin, &args[0])?;
    let ValueData::String(key) = &args[1].data else {
        return Err(invalid_values_argument(
            vm,
            program,
            builtin,
            "a string key argument",
        )?);
    };
    Ok((values, key))
}

fn values_key_value_args<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a MapValue, &'a str, &'a str), VmError> {
    if args.len() != 3 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 3,
            actual: args.len(),
        });
    }
    let values = values_map(vm, program, builtin, &args[0])?;
    let ValueData::String(key) = &args[1].data else {
        return Err(invalid_values_argument(
            vm,
            program,
            builtin,
            "string key/value arguments",
        )?);
    };
    let ValueData::String(value) = &args[2].data else {
        return Err(invalid_values_argument(
            vm,
            program,
            builtin,
            "string key/value arguments",
        )?);
    };
    Ok((values, key, value))
}

fn values_map<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &'a Value,
) -> Result<&'a MapValue, VmError> {
    let ValueData::Map(map) = &value.data else {
        return Err(invalid_values_argument(
            vm,
            program,
            builtin,
            "a url.Values receiver",
        )?);
    };
    Ok(map)
}

fn values_entry_value(values: &MapValue, key: &str) -> Option<Value> {
    values.get(&Value::string(key.to_string()))
}

fn single_values_value(value: &str) -> Value {
    Value::slice(vec![Value::string(value.to_string())])
}

fn encode_values_map(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
    values: &MapValue,
) -> Result<String, VmError> {
    let entries = values.entries_snapshot();
    if entries.is_empty() && values.is_nil() {
        return Ok(String::new());
    }

    let mut encoded_entries = Vec::with_capacity(entries.len());
    for (key, value) in entries {
        let ValueData::String(key) = &key.data else {
            return Err(invalid_values_argument(vm, program, builtin, expected)?);
        };
        let ValueData::Slice(slice) = &value.data else {
            return Err(invalid_values_argument(vm, program, builtin, expected)?);
        };

        let visible = slice.values_snapshot();
        let mut values = Vec::with_capacity(visible.len());
        for item in &visible {
            let ValueData::String(text) = &item.data else {
                return Err(invalid_values_argument(vm, program, builtin, expected)?);
            };
            values.push(text.clone());
        }
        encoded_entries.push((key.clone(), values));
    }
    encoded_entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut parts = Vec::new();
    for (key, values) in encoded_entries {
        if values.is_empty() {
            continue;
        }

        let encoded_key = query_escape_component(&key);
        for value in values {
            parts.push(format!("{encoded_key}={}", query_escape_component(&value)));
        }
    }
    Ok(parts.join("&"))
}

fn url_values_value(entries: BTreeMap<String, Vec<String>>) -> Value {
    let mut values = Vec::with_capacity(entries.len());
    for (key, items) in entries {
        values.push((
            Value::string(key),
            Value::slice(items.into_iter().map(Value::string).collect()),
        ));
    }
    Value {
        typ: TYPE_URL_VALUES,
        data: ValueData::Map(MapValue::with_entries(
            values,
            Value::nil_slice(),
            Some(ConcreteType::TypeId(TYPE_URL_VALUES)),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Function;

    fn test_program() -> Program {
        Program {
            functions: vec![Function {
                name: "main".into(),
                param_count: 0,
                register_count: 0,
                code: vec![],
            }],
            methods: vec![],
            global_count: 0,
            entry_function: 0,
        }
    }

    fn value_strings(value: &Value) -> Vec<String> {
        let ValueData::Slice(slice) = &value.data else {
            panic!("expected []string value");
        };
        slice
            .values_snapshot()
            .iter()
            .map(|item| match &item.data {
                ValueData::String(text) => text.clone(),
                _ => panic!("expected string item"),
            })
            .collect()
    }

    #[test]
    fn url_values_methods_update_existing_keys_without_reordering_entries() {
        let (receiver, _) = parse_query_values("go=1&go=2&wasm=3");
        let program = test_program();
        let mut vm = Vm::new();

        let updated = url_values_set(
            &mut vm,
            &program,
            &[receiver.clone(), Value::string("go"), Value::string("4")],
        )
        .expect("url.Values.Set should succeed");
        let added = url_values_add(
            &mut vm,
            &program,
            &[updated.clone(), Value::string("wasm"), Value::string("5")],
        )
        .expect("url.Values.Add should succeed");

        let ValueData::Map(values) = &added.data else {
            unreachable!("url.Values helpers should keep a map receiver");
        };
        let entries = values.entries_snapshot();
        assert_eq!(entries[0].0, Value::string("go"));
        assert_eq!(entries[1].0, Value::string("wasm"));
        assert_eq!(value_strings(&entries[0].1), vec!["4".to_string()]);
        assert_eq!(
            value_strings(&entries[1].1),
            vec!["3".to_string(), "5".to_string()]
        );
        assert_eq!(
            url_values_get(&mut vm, &program, &[added.clone(), Value::string("go")])
                .expect("url.Values.Get should succeed"),
            Value::string("4"),
        );
        assert_eq!(
            url_values_has(&mut vm, &program, &[added.clone(), Value::string("wasm")])
                .expect("url.Values.Has should succeed"),
            Value::bool(true),
        );

        let deleted = url_values_del(&mut vm, &program, &[added.clone(), Value::string("go")])
            .expect("url.Values.Del should succeed");
        let ValueData::Map(values) = &deleted.data else {
            unreachable!("url.Values.Del should keep a map receiver");
        };
        let entries = values.entries_snapshot();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, Value::string("wasm"));
    }

    #[test]
    fn url_values_methods_mutate_shared_aliases_in_place() {
        let (receiver, _) = parse_query_values("go=1");
        let alias = receiver.clone();
        let program = test_program();
        let mut vm = Vm::new();

        url_values_add(
            &mut vm,
            &program,
            &[receiver.clone(), Value::string("go"), Value::string("2")],
        )
        .expect("url.Values.Add should succeed");
        url_values_set(
            &mut vm,
            &program,
            &[receiver.clone(), Value::string("wasm"), Value::string("3")],
        )
        .expect("url.Values.Set should succeed");
        url_values_del(&mut vm, &program, &[receiver, Value::string("go")])
            .expect("url.Values.Del should succeed");

        let ValueData::Map(values) = &alias.data else {
            unreachable!("url.Values aliases should stay map-backed");
        };
        let entries = values.entries_snapshot();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, Value::string("wasm"));
        assert_eq!(value_strings(&entries[0].1), vec!["3".to_string()]);
    }
}
