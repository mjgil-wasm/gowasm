use crate::{
    program_type_inventory, ConcreteType, Program, RuntimeTypeKind, Value, TYPE_ANY, TYPE_STRING,
};

pub(super) fn target_is_interface(program: &Program, current: &Value) -> bool {
    current.typ == TYPE_ANY
        || program_type_inventory(program)
            .and_then(|inventory| inventory.type_info_for_type_id(current.typ))
            .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
}

pub(super) fn decode_json_interface(
    current: &Value,
    parsed: &serde_json::Value,
) -> Result<Value, String> {
    if parsed.is_null() {
        return Ok(interface_value(current.typ, Value::nil()));
    }
    let dynamic = decode_json_interface_dynamic(parsed)?;
    Ok(interface_value(current.typ, dynamic))
}

fn decode_json_interface_dynamic(parsed: &serde_json::Value) -> Result<Value, String> {
    match parsed {
        serde_json::Value::Null => Ok(Value::nil()),
        serde_json::Value::Bool(boolean) => Ok(Value::bool(*boolean)),
        serde_json::Value::Number(number) => Ok(Value::float(
            parse_json_float(number).map_err(|_| number_mismatch(number, "float64"))?,
        )),
        serde_json::Value::String(text) => Ok(Value::string(text.clone())),
        serde_json::Value::Array(items) => {
            let mut decoded = Vec::with_capacity(items.len());
            for item in items {
                decoded.push(interface_value(
                    TYPE_ANY,
                    decode_json_interface_dynamic(item)?,
                ));
            }
            Ok(Value::slice_typed(
                decoded,
                ConcreteType::Slice {
                    element: Box::new(ConcreteType::TypeId(TYPE_ANY)),
                },
            ))
        }
        serde_json::Value::Object(object) => {
            let mut decoded = Vec::with_capacity(object.len());
            for (member_name, member_value) in object {
                decoded.push((
                    Value::string(member_name.clone()),
                    interface_value(TYPE_ANY, decode_json_interface_dynamic(member_value)?),
                ));
            }
            Ok(Value::map_typed(
                decoded,
                interface_value(TYPE_ANY, Value::nil()),
                ConcreteType::Map {
                    key: Box::new(ConcreteType::TypeId(TYPE_STRING)),
                    value: Box::new(ConcreteType::TypeId(TYPE_ANY)),
                },
            ))
        }
    }
}

fn interface_value(interface_type: crate::TypeId, dynamic: Value) -> Value {
    let mut wrapped = dynamic;
    wrapped.typ = interface_type;
    wrapped
}

fn parse_json_float(number: &serde_json::Number) -> Result<f64, ()> {
    number
        .to_string()
        .parse::<f64>()
        .map_err(|_| ())
        .and_then(|value| {
            if value.is_finite() {
                Ok(value)
            } else {
                Err(())
            }
        })
}

fn number_mismatch(number: &serde_json::Number, target: &str) -> String {
    format!("json: cannot unmarshal number {number} into {target} target")
}
