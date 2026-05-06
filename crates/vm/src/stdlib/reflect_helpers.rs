use super::*;

pub(super) fn reflect_inventory(
    vm: &Vm,
    program: &Program,
) -> Result<ProgramTypeInventory, VmError> {
    program_type_inventory(program)
        .ok_or_else(|| reflect_panic(vm, program, "missing type inventory"))
}

pub(super) fn build_reflect_type_value(
    info: &RuntimeTypeInfo,
    inventory: &ProgramTypeInventory,
    _vm: &Vm,
    _program: &Program,
) -> Result<Value, VmError> {
    let bits = type_bits(info);
    Ok(Value::struct_value(
        TYPE_REFLECT_RTYPE,
        vec![
            (
                TYPE_STRING_FIELD.into(),
                Value::string(render_type_string(info, inventory)),
            ),
            (
                TYPE_NAME_FIELD.into(),
                Value::string(render_type_name(info)),
            ),
            (
                TYPE_PKG_PATH_FIELD.into(),
                Value::string(render_type_pkg_path(info)),
            ),
            (
                TYPE_KIND_FIELD.into(),
                reflect_kind_value(kind_code_for(&info.kind)),
            ),
            (
                TYPE_ELEM_FIELD.into(),
                encode_optional_concrete_type_ref(info.elem.as_deref()),
            ),
            (
                TYPE_KEY_FIELD.into(),
                encode_optional_concrete_type_ref(info.key.as_deref()),
            ),
            (
                TYPE_LEN_FIELD.into(),
                Value::int(info.len.unwrap_or_default() as i64),
            ),
            (TYPE_HAS_LEN_FIELD.into(), Value::bool(info.len.is_some())),
            (
                TYPE_FIELDS_FIELD.into(),
                Value::slice(reflect_struct_fields(info)),
            ),
            (TYPE_BITS_FIELD.into(), Value::int(bits.unwrap_or_default())),
            (TYPE_HAS_BITS_FIELD.into(), Value::bool(bits.is_some())),
            (
                TYPE_COMPARABLE_FIELD.into(),
                Value::bool(type_is_comparable(info, inventory)),
            ),
            (
                TYPE_PARAMS_FIELD.into(),
                Value::slice(encode_concrete_type_refs(&info.params)),
            ),
            (
                TYPE_RESULTS_FIELD.into(),
                Value::slice(encode_concrete_type_refs(&info.results)),
            ),
        ],
    ))
}

pub(super) fn build_reflect_value_from_dynamic(
    value: &Value,
    can_interface: bool,
    inventory: &ProgramTypeInventory,
    vm: &Vm,
    program: &Program,
) -> Result<Value, VmError> {
    if matches!(&value.data, ValueData::Nil)
        && (value.typ == TYPE_NIL || is_nil_interface_value(inventory, value))
    {
        return Ok(build_invalid_reflect_value());
    }
    let info = inventory
        .value_type_info(vm, program, value)
        .ok_or_else(|| reflect_panic(vm, program, "ValueOf cannot describe this value"))?;
    build_reflect_value_with_type_info(value.clone(), &info, can_interface, inventory, vm, program)
}

pub(super) fn build_reflect_value_with_type_info(
    value: Value,
    info: &RuntimeTypeInfo,
    can_interface: bool,
    inventory: &ProgramTypeInventory,
    vm: &Vm,
    program: &Program,
) -> Result<Value, VmError> {
    let type_value = build_reflect_type_value(info, inventory, vm, program)?;
    Ok(build_reflect_value_with_type_value(
        value,
        type_value,
        can_interface,
    ))
}

pub(super) fn build_reflect_value_with_type_value(
    value: Value,
    type_value: Value,
    can_interface: bool,
) -> Value {
    Value::struct_value(
        TYPE_REFLECT_RVALUE,
        vec![
            (VALUE_VALUE_FIELD.into(), value),
            (VALUE_TYPE_FIELD.into(), type_value),
            (VALUE_VALID_FIELD.into(), Value::bool(true)),
            (VALUE_CAN_INTERFACE_FIELD.into(), Value::bool(can_interface)),
        ],
    )
}

pub(super) fn build_invalid_reflect_value() -> Value {
    Value::struct_value(
        TYPE_REFLECT_RVALUE,
        vec![
            (VALUE_VALUE_FIELD.into(), Value::nil()),
            (VALUE_TYPE_FIELD.into(), Value::nil()),
            (VALUE_VALID_FIELD.into(), Value::bool(false)),
            (VALUE_CAN_INTERFACE_FIELD.into(), Value::bool(false)),
        ],
    )
}

pub(super) fn type_value_from_type_id(
    type_id: crate::TypeId,
    inventory: &ProgramTypeInventory,
    vm: &Vm,
    program: &Program,
) -> Result<Value, VmError> {
    let info = inventory
        .type_info_for_type_id(type_id)
        .ok_or_else(|| reflect_panic(vm, program, "missing type metadata"))?;
    build_reflect_type_value(&info, inventory, vm, program)
}

fn type_bits(info: &RuntimeTypeInfo) -> Option<i64> {
    match info.kind {
        RuntimeTypeKind::Int | RuntimeTypeKind::Float64 => Some(64),
        _ => None,
    }
}

fn type_is_comparable(info: &RuntimeTypeInfo, inventory: &ProgramTypeInventory) -> bool {
    match info.kind {
        RuntimeTypeKind::Bool
        | RuntimeTypeKind::Int
        | RuntimeTypeKind::Float64
        | RuntimeTypeKind::String
        | RuntimeTypeKind::Interface
        | RuntimeTypeKind::Pointer
        | RuntimeTypeKind::Channel => true,
        RuntimeTypeKind::Array => info
            .elem
            .as_deref()
            .and_then(|elem| inventory.resolve_concrete_type(elem))
            .is_some_and(|elem| type_is_comparable(&elem, inventory)),
        RuntimeTypeKind::Struct => info.fields.iter().all(|field| {
            inventory
                .resolve_concrete_type(&field.typ)
                .is_some_and(|field_info| type_is_comparable(&field_info, inventory))
        }),
        RuntimeTypeKind::Nil
        | RuntimeTypeKind::Slice
        | RuntimeTypeKind::Map
        | RuntimeTypeKind::Function => false,
    }
}

pub(super) fn extract_kind_receiver(
    vm: &Vm,
    program: &Program,
    args: &[Value],
    function_name: &str,
) -> Result<i64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    if args[0].typ != TYPE_REFLECT_KIND {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{function_name} expects a reflect.Kind receiver"),
        ));
    }
    let ValueData::Int(kind) = &args[0].data else {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{function_name} expects a reflect.Kind receiver"),
        ));
    };
    Ok(*kind)
}

pub(super) fn extract_reflect_type_receiver<'a>(
    vm: &Vm,
    program: &Program,
    args: &'a [Value],
    function_name: &str,
) -> Result<&'a Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    extract_reflect_type_value(vm, program, &args[0], function_name)
}

pub(super) fn extract_reflect_type_value<'a>(
    vm: &Vm,
    program: &Program,
    value: &'a Value,
    function_name: &str,
) -> Result<&'a Value, VmError> {
    if value.typ != TYPE_REFLECT_RTYPE {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{function_name} expects a reflect.Type receiver"),
        ));
    }
    Ok(value)
}

pub(super) fn extract_reflect_value_receiver<'a>(
    vm: &Vm,
    program: &Program,
    args: &'a [Value],
    function_name: &str,
) -> Result<&'a Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    extract_reflect_value_value(vm, program, &args[0], function_name)
}

pub(super) fn extract_reflect_value_value<'a>(
    vm: &Vm,
    program: &Program,
    value: &'a Value,
    function_name: &str,
) -> Result<&'a Value, VmError> {
    if value.typ != TYPE_REFLECT_RVALUE {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{function_name} expects a reflect.Value receiver"),
        ));
    }
    Ok(value)
}

pub(super) fn reflect_value_valid(receiver: &Value) -> Result<bool, VmError> {
    hidden_bool(receiver, VALUE_VALID_FIELD)
}

pub(super) fn reflect_value_can_interface(receiver: &Value) -> Result<bool, VmError> {
    hidden_bool(receiver, VALUE_CAN_INTERFACE_FIELD)
}

pub(super) fn reflect_value_underlying(receiver: &Value) -> Result<Value, VmError> {
    hidden_field(receiver, VALUE_VALUE_FIELD)
}

pub(super) fn reflect_value_type_required(
    vm: &Vm,
    program: &Program,
    receiver: &Value,
    method: &str,
) -> Result<Value, VmError> {
    if !reflect_value_valid(receiver)? {
        return Err(reflect_invalid_value_panic(vm, program, method));
    }
    let type_value = hidden_field(receiver, VALUE_TYPE_FIELD)?;
    if matches!(&type_value.data, ValueData::Nil) {
        return Err(reflect_invalid_value_panic(vm, program, method));
    }
    Ok(type_value)
}

pub(super) fn hidden_field(receiver: &Value, field: &str) -> Result<Value, VmError> {
    let ValueData::Struct(fields) = &receiver.data else {
        return Err(VmError::InvalidFieldTarget {
            function: "<reflect>".into(),
            target: "non-struct reflect metadata".into(),
        });
    };
    fields
        .iter()
        .find_map(|(name, value)| (name == field).then(|| value.clone()))
        .ok_or(VmError::UnknownField {
            function: "<reflect>".into(),
            field: field.into(),
        })
}

pub(super) fn hidden_bool(receiver: &Value, field: &str) -> Result<bool, VmError> {
    match hidden_field(receiver, field)?.data {
        ValueData::Bool(value) => Ok(value),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "bool".into(),
        }),
    }
}

pub(super) fn hidden_kind(receiver: &Value) -> Result<i64, VmError> {
    match hidden_field(receiver, TYPE_KIND_FIELD)?.data {
        ValueData::Int(kind) => Ok(kind),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "reflect.Kind".into(),
        }),
    }
}

pub(super) fn hidden_slice(receiver: &Value, field: &str) -> Result<Vec<Value>, VmError> {
    match hidden_field(receiver, field)?.data {
        ValueData::Slice(slice) => Ok(slice.values_snapshot()),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "slice".into(),
        }),
    }
}

pub(super) fn hidden_string(receiver: &Value, field: &str) -> Result<String, VmError> {
    match hidden_field(receiver, field)?.data {
        ValueData::String(value) => Ok(value),
        _ => Err(VmError::TypeMismatch {
            function: "<reflect>".into(),
            expected: "string".into(),
        }),
    }
}

pub(super) fn reflect_type_member(
    vm: &Vm,
    program: &Program,
    receiver: &Value,
    field: &str,
    method: &str,
) -> Result<Value, VmError> {
    let value = hidden_field(receiver, field)?;
    if matches!(&value.data, ValueData::Nil) {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} of unsupported type"),
        ));
    }
    let inventory = reflect_inventory(vm, program)?;
    resolve_concrete_type_ref_value(&value, &inventory, vm, program)
}

fn render_type_string(info: &RuntimeTypeInfo, inventory: &ProgramTypeInventory) -> String {
    if let Some(name) = render_named_type_name(info) {
        if info.display_name.contains('.') || is_predeclared_named_type(info.display_name.as_str())
        {
            return info.display_name.clone();
        }
        if let Some(package_path) = info.package_path.as_deref() {
            return format!("{}.{}", short_package_name(package_path), name);
        }
        return name;
    }

    if let Some(display_name) = render_unnamed_type_syntax(info.display_name.as_str()) {
        return display_name;
    }

    match info.kind {
        RuntimeTypeKind::Array => format!(
            "[{}]{}",
            info.len.unwrap_or_default(),
            render_concrete_string(info.elem.as_deref(), inventory)
        ),
        RuntimeTypeKind::Slice => {
            format!(
                "[]{}",
                render_concrete_string(info.elem.as_deref(), inventory)
            )
        }
        RuntimeTypeKind::Map => format!(
            "map[{}]{}",
            render_concrete_string(info.key.as_deref(), inventory),
            render_concrete_string(info.elem.as_deref(), inventory)
        ),
        RuntimeTypeKind::Pointer => {
            format!(
                "*{}",
                render_concrete_string(info.elem.as_deref(), inventory)
            )
        }
        RuntimeTypeKind::Function => render_function_string(info, inventory),
        RuntimeTypeKind::Channel => render_channel_string(info, inventory),
        _ => info.display_name.clone(),
    }
}

fn render_unnamed_type_syntax(display_name: &str) -> Option<String> {
    if let Some(body) = display_name.strip_prefix("interface{") {
        return Some(format!("interface {{{body}"));
    }
    if let Some(body) = display_name.strip_prefix("struct{") {
        return Some(format!("struct {{{body}"));
    }
    None
}

fn render_function_string(info: &RuntimeTypeInfo, inventory: &ProgramTypeInventory) -> String {
    let params = info
        .params
        .iter()
        .map(|typ| render_concrete_string(Some(typ), inventory))
        .collect::<Vec<_>>()
        .join(", ");
    match info.results.as_slice() {
        [] => format!("func({params})"),
        [result] => format!(
            "func({params}) {}",
            render_concrete_string(Some(result), inventory)
        ),
        many => format!(
            "func({params}) ({})",
            many.iter()
                .map(|typ| render_concrete_string(Some(typ), inventory))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_channel_string(info: &RuntimeTypeInfo, inventory: &ProgramTypeInventory) -> String {
    let element = render_concrete_string(info.elem.as_deref(), inventory);
    match info.channel_direction {
        Some(RuntimeChannelDirection::SendOnly) => format!("chan<- {element}"),
        Some(RuntimeChannelDirection::ReceiveOnly) => format!("<-chan {element}"),
        _ => format!("chan {element}"),
    }
}

fn render_concrete_string(typ: Option<&ConcreteType>, inventory: &ProgramTypeInventory) -> String {
    typ.and_then(|typ| inventory.resolve_concrete_type(typ))
        .map(|info| render_type_string(&info, inventory))
        .unwrap_or_else(|| "<nil>".into())
}

fn render_type_name(info: &RuntimeTypeInfo) -> String {
    render_named_type_name(info).unwrap_or_default()
}

fn render_named_type_name(info: &RuntimeTypeInfo) -> Option<String> {
    let display_name = info.display_name.as_str();
    if display_name == "interface{}"
        || display_name == "struct{}"
        || display_name.starts_with("struct{")
        || display_name.starts_with("interface{")
        || display_name.starts_with('[')
        || display_name.starts_with("[]")
        || display_name.starts_with("map[")
        || display_name.starts_with("chan ")
        || display_name.starts_with("chan<- ")
        || display_name.starts_with("<-chan ")
        || display_name.starts_with('*')
        || display_name.starts_with("__gowasm_func__(")
    {
        return None;
    }
    Some(
        display_name
            .rsplit_once('.')
            .map(|(_, tail)| tail)
            .unwrap_or(display_name)
            .to_string(),
    )
}

fn render_type_pkg_path(info: &RuntimeTypeInfo) -> String {
    if render_named_type_name(info).is_none()
        || is_predeclared_named_type(info.display_name.as_str())
    {
        return String::new();
    }
    info.package_path.clone().unwrap_or_default()
}

pub(super) fn struct_field_pkg_path(owner: &RuntimeTypeInfo, field: &RuntimeTypeField) -> String {
    let Some(first) = field.name.chars().next() else {
        return String::new();
    };
    if first.is_uppercase() {
        return String::new();
    }
    owner.package_path.clone().unwrap_or_default()
}

fn short_package_name(package_path: &str) -> &str {
    if package_path == "." {
        return "main";
    }
    package_path.rsplit('/').next().unwrap_or(package_path)
}

fn is_predeclared_named_type(display_name: &str) -> bool {
    matches!(
        display_name,
        "bool" | "error" | "float64" | "int" | "string"
    )
}

fn kind_code_for(kind: &RuntimeTypeKind) -> i64 {
    match kind {
        RuntimeTypeKind::Nil => KIND_INVALID,
        RuntimeTypeKind::Int => KIND_INT,
        RuntimeTypeKind::Float64 => KIND_FLOAT64,
        RuntimeTypeKind::String => KIND_STRING,
        RuntimeTypeKind::Bool => KIND_BOOL,
        RuntimeTypeKind::Array => KIND_ARRAY,
        RuntimeTypeKind::Slice => KIND_SLICE,
        RuntimeTypeKind::Map => KIND_MAP,
        RuntimeTypeKind::Struct => KIND_STRUCT,
        RuntimeTypeKind::Interface => KIND_INTERFACE,
        RuntimeTypeKind::Pointer => KIND_PTR,
        RuntimeTypeKind::Function => KIND_FUNC,
        RuntimeTypeKind::Channel => KIND_CHAN,
    }
}

pub(super) fn reflect_kind_name(kind: i64) -> String {
    match kind {
        KIND_INVALID => "invalid".into(),
        KIND_BOOL => "bool".into(),
        KIND_INT => "int".into(),
        KIND_FLOAT64 => "float64".into(),
        KIND_ARRAY => "array".into(),
        KIND_CHAN => "chan".into(),
        KIND_FUNC => "func".into(),
        KIND_INTERFACE => "interface".into(),
        KIND_MAP => "map".into(),
        KIND_PTR => "ptr".into(),
        KIND_SLICE => "slice".into(),
        KIND_STRING => "string".into(),
        KIND_STRUCT => "struct".into(),
        other => format!("kind({other})"),
    }
}

pub(super) fn reflect_kind_value(kind: i64) -> Value {
    Value {
        typ: TYPE_REFLECT_KIND,
        data: ValueData::Int(kind),
    }
}

pub(super) fn is_nil_interface_value(inventory: &ProgramTypeInventory, value: &Value) -> bool {
    matches!(&value.data, ValueData::Nil)
        && (matches!(value.typ, TYPE_ANY | TYPE_ERROR)
            || inventory
                .type_info_for_type_id(value.typ)
                .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface)))
}

pub(super) fn reflect_panic(vm: &Vm, program: &Program, detail: &str) -> VmError {
    VmError::UnhandledPanic {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<unknown>".into()),
        value: format!("reflect: {detail}"),
    }
}

pub(super) fn reflect_invalid_value_panic(vm: &Vm, program: &Program, method: &str) -> VmError {
    reflect_panic(
        vm,
        program,
        &format!("call of reflect.Value.{method} on zero Value"),
    )
}
