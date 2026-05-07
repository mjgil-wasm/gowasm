use super::super::{
    StdlibFunction, StdlibMethod, NET_HTTP_HEADER_ADD, NET_HTTP_HEADER_CLONE, NET_HTTP_HEADER_DEL,
    NET_HTTP_HEADER_GET, NET_HTTP_HEADER_SET, NET_HTTP_HEADER_VALUES,
};
use super::core::canonicalize_header_key_text;
use crate::{MapValue, Program, Value, ValueData, Vm, VmError};

pub(crate) const NET_HTTP_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: "http.Header",
        method: "Get",
        function: NET_HTTP_HEADER_GET,
    },
    StdlibMethod {
        receiver_type: "http.Header",
        method: "Values",
        function: NET_HTTP_HEADER_VALUES,
    },
    StdlibMethod {
        receiver_type: "http.Header",
        method: "Set",
        function: NET_HTTP_HEADER_SET,
    },
    StdlibMethod {
        receiver_type: "http.Header",
        method: "Add",
        function: NET_HTTP_HEADER_ADD,
    },
    StdlibMethod {
        receiver_type: "http.Header",
        method: "Del",
        function: NET_HTTP_HEADER_DEL,
    },
    StdlibMethod {
        receiver_type: "http.Header",
        method: "Clone",
        function: NET_HTTP_HEADER_CLONE,
    },
];

pub(crate) const NET_HTTP_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: NET_HTTP_HEADER_GET,
        symbol: "Get",
        returns_value: true,
        handler: header_get,
    },
    StdlibFunction {
        id: NET_HTTP_HEADER_VALUES,
        symbol: "Values",
        returns_value: true,
        handler: header_values,
    },
    StdlibFunction {
        id: NET_HTTP_HEADER_SET,
        symbol: "Set",
        returns_value: false,
        handler: header_set,
    },
    StdlibFunction {
        id: NET_HTTP_HEADER_ADD,
        symbol: "Add",
        returns_value: false,
        handler: header_add,
    },
    StdlibFunction {
        id: NET_HTTP_HEADER_DEL,
        symbol: "Del",
        returns_value: false,
        handler: header_del,
    },
    StdlibFunction {
        id: NET_HTTP_HEADER_CLONE,
        symbol: "Clone",
        returns_value: true,
        handler: header_clone,
    },
];

fn header_get(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (header, key) = header_key_args(vm, program, "http.Header.Get", args)?;
    let canonical = canonicalize_header_key_text(key);
    let Some(entry) = header_entry_value(header, &canonical) else {
        return Ok(Value::string(String::new()));
    };
    let ValueData::Slice(slice) = &entry.data else {
        return Err(invalid_header_argument(
            vm,
            program,
            "http.Header.Get",
            "header values stored as []string",
        )?);
    };
    let values = slice.values_snapshot();
    let Some(first) = values.first() else {
        return Ok(Value::string(String::new()));
    };
    let ValueData::String(text) = &first.data else {
        return Err(invalid_header_argument(
            vm,
            program,
            "http.Header.Get",
            "header values stored as []string",
        )?);
    };
    Ok(Value::string(text.clone()))
}

fn header_values(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (header, key) = header_key_args(vm, program, "http.Header.Values", args)?;
    let canonical = canonicalize_header_key_text(key);
    let Some(entry) = header_entry_value(header, &canonical) else {
        return Ok(Value::nil_slice());
    };
    let ValueData::Slice(_) = &entry.data else {
        return Err(invalid_header_argument(
            vm,
            program,
            "http.Header.Values",
            "header values stored as []string",
        )?);
    };
    Ok(entry)
}

fn header_set(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (header, key, value) = header_key_value_args(vm, program, "http.Header.Set", args)?;
    if header.is_nil() {
        return Err(VmError::AssignToNilMap {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[0]),
        });
    }
    let canonical = canonicalize_header_key_text(key);
    let lookup_key = Value::string(canonical.clone());
    let inserted = header.insert(lookup_key, single_header_value(value));
    debug_assert!(
        inserted,
        "non-nil http.Header receiver should stay writable"
    );
    Ok(args[0].clone())
}

fn header_add(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (header, key, value) = header_key_value_args(vm, program, "http.Header.Add", args)?;
    if header.is_nil() {
        return Err(VmError::AssignToNilMap {
            function: vm.current_function_name(program)?,
            target: crate::describe_value(&args[0]),
        });
    }
    let canonical = canonicalize_header_key_text(key);
    let lookup_key = Value::string(canonical.clone());
    let updated_value = if let Some(existing) = header.get(&lookup_key) {
        let ValueData::Slice(slice) = &existing.data else {
            return Err(invalid_header_argument(
                vm,
                program,
                "http.Header.Add",
                "header values stored as []string",
            )?);
        };
        let mut values = slice.values_snapshot();
        values.push(Value::string(value.to_string()));
        Value::slice(values)
    } else {
        single_header_value(value)
    };
    let inserted = header.insert(lookup_key, updated_value);
    debug_assert!(
        inserted,
        "non-nil http.Header receiver should stay writable"
    );
    Ok(args[0].clone())
}

fn header_del(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (header, key) = header_key_args(vm, program, "http.Header.Del", args)?;
    if header.is_nil() {
        return Ok(args[0].clone());
    }
    let canonical = canonicalize_header_key_text(key);
    let removed = header.remove(&Value::string(canonical));
    debug_assert!(
        removed,
        "non-nil http.Header receiver delete should not fail"
    );
    Ok(args[0].clone())
}

fn header_clone(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let header = header_map(vm, program, "http.Header.Clone", &args[0])?;
    let mut cloned = args[0].clone();
    cloned.data = ValueData::Map(if header.is_nil() {
        MapValue::nil((*header.zero_value).clone(), header.concrete_type.clone())
    } else {
        MapValue::with_entries(
            header.entries_snapshot(),
            (*header.zero_value).clone(),
            header.concrete_type.clone(),
        )
    });
    Ok(cloned)
}

fn header_key_args<'a>(
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
    let header = header_map(vm, program, builtin, &args[0])?;
    let ValueData::String(key) = &args[1].data else {
        return Err(invalid_header_argument(
            vm,
            program,
            builtin,
            "a string key argument",
        )?);
    };
    Ok((header, key))
}

fn header_key_value_args<'a>(
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
    let header = header_map(vm, program, builtin, &args[0])?;
    let ValueData::String(key) = &args[1].data else {
        return Err(invalid_header_argument(
            vm,
            program,
            builtin,
            "string key/value arguments",
        )?);
    };
    let ValueData::String(value) = &args[2].data else {
        return Err(invalid_header_argument(
            vm,
            program,
            builtin,
            "string key/value arguments",
        )?);
    };
    Ok((header, key, value))
}

fn header_map<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    value: &'a Value,
) -> Result<&'a MapValue, VmError> {
    let ValueData::Map(map) = &value.data else {
        return Err(invalid_header_argument(
            vm,
            program,
            builtin,
            "an http.Header receiver",
        )?);
    };
    Ok(map)
}

fn header_entry_value(header: &MapValue, canonical: &str) -> Option<Value> {
    header.get(&Value::string(canonical.to_string()))
}

fn single_header_value(value: &str) -> Value {
    Value::slice(vec![Value::string(value.to_string())])
}

fn invalid_header_argument(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConcreteType, Function, TYPE_HTTP_HEADER};

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

    fn header_value(entries: Vec<(Value, Value)>) -> Value {
        Value {
            typ: TYPE_HTTP_HEADER,
            data: ValueData::Map(MapValue::with_entries(
                entries,
                Value::nil_slice(),
                Some(ConcreteType::TypeId(TYPE_HTTP_HEADER)),
            )),
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
    fn header_methods_update_existing_keys_without_reordering_entries() {
        let receiver = header_value(vec![
            (
                Value::string("Content-Type"),
                Value::slice(vec![Value::string("text/plain")]),
            ),
            (
                Value::string("X-Test"),
                Value::slice(vec![Value::string("one")]),
            ),
        ]);
        let program = test_program();
        let mut vm = Vm::new();

        let updated = header_set(
            &mut vm,
            &program,
            &[
                receiver.clone(),
                Value::string("content-type"),
                Value::string("application/json"),
            ],
        )
        .expect("http.Header.Set should succeed");
        let added = header_add(
            &mut vm,
            &program,
            &[
                updated.clone(),
                Value::string("x-test"),
                Value::string("two"),
            ],
        )
        .expect("http.Header.Add should succeed");

        let ValueData::Map(header) = &added.data else {
            unreachable!("header helpers should keep a map receiver");
        };
        let entries = header.entries_snapshot();
        assert_eq!(entries[0].0, Value::string("Content-Type"));
        assert_eq!(entries[1].0, Value::string("X-Test"));
        assert_eq!(
            value_strings(&entries[0].1),
            vec!["application/json".to_string()]
        );
        assert_eq!(
            value_strings(&entries[1].1),
            vec!["one".to_string(), "two".to_string()]
        );
        assert_eq!(
            header_get(
                &mut vm,
                &program,
                &[added.clone(), Value::string("content-type")],
            )
            .expect("http.Header.Get should succeed"),
            Value::string("application/json"),
        );

        let deleted = header_del(
            &mut vm,
            &program,
            &[added.clone(), Value::string("CONTENT-TYPE")],
        )
        .expect("http.Header.Del should succeed");
        let ValueData::Map(header) = &deleted.data else {
            unreachable!("header delete should keep a map receiver");
        };
        let entries = header.entries_snapshot();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, Value::string("X-Test"));
    }

    #[test]
    fn header_methods_mutate_shared_aliases_in_place() {
        let receiver = header_value(vec![(
            Value::string("Content-Type"),
            Value::slice(vec![Value::string("text/plain")]),
        )]);
        let alias = receiver.clone();
        let program = test_program();
        let mut vm = Vm::new();

        header_add(
            &mut vm,
            &program,
            &[
                receiver.clone(),
                Value::string("content-type"),
                Value::string("charset=utf-8"),
            ],
        )
        .expect("http.Header.Add should succeed");
        header_set(
            &mut vm,
            &program,
            &[
                receiver.clone(),
                Value::string("x-test"),
                Value::string("one"),
            ],
        )
        .expect("http.Header.Set should succeed");
        header_del(
            &mut vm,
            &program,
            &[receiver, Value::string("content-type")],
        )
        .expect("http.Header.Del should succeed");

        let ValueData::Map(header) = &alias.data else {
            unreachable!("http.Header aliases should stay map-backed");
        };
        let entries = header.entries_snapshot();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, Value::string("X-Test"));
        assert_eq!(value_strings(&entries[0].1), vec!["one".to_string()]);
    }
}
