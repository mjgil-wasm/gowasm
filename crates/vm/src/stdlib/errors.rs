use crate::{
    concrete_type_for_value, program_type_inventory, value::ErrorValue, Program, RuntimeTypeKind,
    TypeId, Value, ValueData, Vm, VmError, TYPE_ANY, TYPE_ERROR,
};

use super::StdlibFunction;

pub(super) const ERRORS_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: super::ERRORS_NEW,
        symbol: "New",
        returns_value: true,
        handler: errors_new,
    },
    StdlibFunction {
        id: super::ERRORS_JOIN,
        symbol: "Join",
        returns_value: true,
        handler: errors_join,
    },
    StdlibFunction {
        id: super::ERRORS_UNWRAP,
        symbol: "Unwrap",
        returns_value: true,
        handler: errors_unwrap,
    },
    StdlibFunction {
        id: super::ERRORS_IS,
        symbol: "Is",
        returns_value: true,
        handler: errors_is,
    },
    StdlibFunction {
        id: super::ERRORS_AS,
        symbol: "As",
        returns_value: true,
        handler: errors_as,
    },
];

pub(super) fn errors_new(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(message) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "errors.New".into(),
            expected: "a string argument".into(),
        });
    };

    Ok(Value::error(message.clone()))
}

pub(super) fn errors_join(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let mut messages = Vec::new();
    for arg in args {
        match &arg.data {
            ValueData::Nil => {}
            ValueData::Error(err) => messages.push(err.message.clone()),
            _ => {
                return Err(VmError::InvalidErrorValue {
                    function: vm.current_function_name(program)?,
                });
            }
        }
    }

    if messages.is_empty() {
        Ok(Value::nil())
    } else {
        Ok(joined_error_value(
            messages.join("\n"),
            filtered_error_args(args),
        ))
    }
}

pub(super) fn joined_error_value(message: String, children: Vec<Value>) -> Value {
    Value {
        typ: TYPE_ERROR,
        data: ValueData::Error(ErrorValue {
            message,
            kind_message: None,
            wrapped: Some(Box::new(Value::array(children))),
        }),
    }
}

fn errors_unwrap(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let err = &args[0];
    match &err.data {
        ValueData::Error(e) => match e.wrapped.as_deref() {
            Some(Value {
                data: ValueData::Array(_),
                ..
            }) => Ok(Value::nil()),
            Some(inner) => Ok(inner.clone()),
            None => Ok(Value::nil()),
        },
        ValueData::Nil => Ok(Value::nil()),
        _ => Ok(Value::nil()),
    }
}

fn errors_is(_vm: &mut Vm, _program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let err = &args[0];
    let target = &args[1];
    if matches!(target.data, ValueData::Nil) {
        return Ok(Value::bool(matches!(err.data, ValueData::Nil)));
    }
    Ok(Value::bool(error_chain_contains(err, target)))
}

fn errors_as(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let target_kind = classify_as_target(vm, program, &args[1])?;
    let Some(matched) = find_error_as_match(vm, program, &args[0], &target_kind) else {
        return Ok(Value::bool(false));
    };

    vm.store_indirect(program, &args[1], matched)?;
    Ok(Value::bool(true))
}

fn error_chain_contains(err: &Value, target: &Value) -> bool {
    walk_error_chain(err, &mut |current| {
        current_matches_is_target(current, target)
    })
}

fn current_matches_is_target(current: &Value, target: &Value) -> bool {
    if current.data == target.data {
        return true;
    }
    let (ValueData::Error(current_err), ValueData::Error(target_err)) =
        (&current.data, &target.data)
    else {
        return false;
    };
    error_matches_target(current_err, target_err)
}

fn error_matches_target(current: &ErrorValue, target: &ErrorValue) -> bool {
    if current.message == target.message {
        return true;
    }

    if let Some(kind) = current.kind_message.as_deref() {
        if kind == target.message {
            return true;
        }
        if let Some(target_kind) = target.kind_message.as_deref() {
            return kind == target_kind;
        }
    }

    false
}

fn filtered_error_args(args: &[Value]) -> Vec<Value> {
    args.iter()
        .filter(|arg| !matches!(arg.data, ValueData::Nil))
        .cloned()
        .collect()
}

fn walk_error_chain(err: &Value, visit: &mut impl FnMut(&Value) -> bool) -> bool {
    if matches!(err.data, ValueData::Nil) {
        return false;
    }
    if visit(err) {
        return true;
    }
    let ValueData::Error(error) = &err.data else {
        return false;
    };
    wrapped_error_children(error)
        .into_iter()
        .any(|child| walk_error_chain(&child, visit))
}

fn wrapped_error_children(error: &ErrorValue) -> Vec<Value> {
    match error.wrapped.as_deref() {
        Some(Value {
            data: ValueData::Array(array),
            ..
        }) => array.values_snapshot(),
        Some(wrapped) => vec![wrapped.clone()],
        None => Vec::new(),
    }
}

enum ErrorsAsTarget {
    ErrorInterface,
    Exact(TypeId),
}

fn classify_as_target(
    vm: &Vm,
    program: &Program,
    target: &Value,
) -> Result<ErrorsAsTarget, VmError> {
    let ValueData::Pointer(pointer) = &target.data else {
        return Err(errors_panic(
            vm,
            program,
            "target must be a non-nil pointer",
        ));
    };
    if pointer.is_nil() {
        return Err(errors_panic(
            vm,
            program,
            "target must be a non-nil pointer",
        ));
    }

    let target_slot = vm.deref_pointer(program, target)?;
    if target_slot.typ == TYPE_ERROR {
        return Ok(ErrorsAsTarget::ErrorInterface);
    }
    if target_slot.typ == TYPE_ANY || type_id_is_interface(program, target_slot.typ) {
        return Err(errors_panic(
            vm,
            program,
            "*target interface matching beyond `*error` is not supported in the current subset",
        ));
    }
    if type_implements_error(program, target_slot.typ) {
        return Ok(ErrorsAsTarget::Exact(target_slot.typ));
    }
    Err(errors_panic(
        vm,
        program,
        "*target must be `error` or an exact concrete type that implements error",
    ))
}

fn find_error_as_match(
    vm: &Vm,
    program: &Program,
    err: &Value,
    target: &ErrorsAsTarget,
) -> Option<Value> {
    let mut matched = None;
    walk_error_chain(err, &mut |current| {
        let value = match target {
            ErrorsAsTarget::ErrorInterface => {
                let mut value = current.clone();
                value.typ = TYPE_ERROR;
                value
            }
            ErrorsAsTarget::Exact(type_id) => {
                let current_type = exact_runtime_type_id(vm, program, current);
                if current_type != Some(*type_id) {
                    return false;
                }
                let mut value = current.clone();
                value.typ = *type_id;
                value
            }
        };
        matched = Some(value);
        true
    });
    matched
}

fn exact_runtime_type_id(vm: &Vm, program: &Program, value: &Value) -> Option<TypeId> {
    match concrete_type_for_value(vm, program, value) {
        Some(crate::ConcreteType::TypeId(type_id)) => Some(type_id),
        _ => None,
    }
}

fn type_id_is_interface(program: &Program, type_id: TypeId) -> bool {
    program_type_inventory(program)
        .and_then(|inventory| inventory.type_info_for_type_id(type_id))
        .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
}

fn type_implements_error(program: &Program, type_id: TypeId) -> bool {
    program.methods.iter().any(|binding| {
        binding.receiver_type == type_id
            && binding.name == "Error"
            && binding.param_types.is_empty()
            && binding.result_types == ["string".to_string()]
    })
}

fn errors_panic(vm: &Vm, program: &Program, detail: &str) -> VmError {
    VmError::UnhandledPanic {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        value: format!("errors: {detail}"),
    }
}
