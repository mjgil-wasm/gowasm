use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::{maybe_truncate, FormatFlags};
use crate::{
    format_value, program_type_inventory, value_runtime_type, FunctionValue, PointerTarget,
    PointerValue, Program, RuntimeTypeInfo, RuntimeTypeKind, Value, ValueData, Vm, VmError,
    TYPE_ANY, TYPE_BASE64_ENCODING, TYPE_BOOL, TYPE_CONTEXT, TYPE_ERROR, TYPE_FLOAT64,
    TYPE_FS_DIR_ENTRY, TYPE_FS_FILE, TYPE_FS_FILE_INFO, TYPE_FUNCTION, TYPE_HTTP_CLIENT,
    TYPE_HTTP_HEADER, TYPE_HTTP_REQUEST, TYPE_HTTP_REQUEST_BODY, TYPE_HTTP_RESPONSE,
    TYPE_HTTP_RESPONSE_BODY, TYPE_INT, TYPE_INT64, TYPE_OS_DIR_FS, TYPE_REGEXP, TYPE_STRING,
    TYPE_STRINGS_REPLACER, TYPE_SYNC_MUTEX, TYPE_SYNC_ONCE, TYPE_SYNC_RW_MUTEX,
    TYPE_SYNC_WAIT_GROUP, TYPE_TIME, TYPE_TIME_TIMER, TYPE_URL, TYPE_URL_VALUES,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueFormatStyle {
    Default,
    Plus,
    Sharp,
}

#[derive(Default)]
struct RenderState {
    active_pointer_targets: Vec<PointerTarget>,
}

impl RenderState {
    fn enter_pointer_target(&mut self, target: &PointerTarget) -> bool {
        if self.active_pointer_targets.contains(target) {
            return false;
        }
        self.active_pointer_targets.push(target.clone());
        true
    }

    fn exit_pointer_target(&mut self) {
        self.active_pointer_targets.pop();
    }
}

pub(crate) fn render_print_value(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
) -> Result<String, VmError> {
    let mut state = RenderState::default();
    if let Some(text) = render_display_text(vm, program, value)? {
        return Ok(text);
    }
    render_reflective_value(&mut state, vm, program, value, ValueFormatStyle::Default)
}

pub(super) fn render_verb_value(
    vm: &mut Vm,
    program: &Program,
    verb: char,
    value: &Value,
    flags: &FormatFlags,
) -> Result<String, VmError> {
    let mut state = RenderState::default();
    match verb {
        'v' => {
            if flags.alternate {
                return render_reflective_value(
                    &mut state,
                    vm,
                    program,
                    value,
                    ValueFormatStyle::Sharp,
                );
            }
            if let Some(text) = render_display_text(vm, program, value)? {
                return Ok(text);
            }
            if flags.plus_sign {
                render_reflective_value(&mut state, vm, program, value, ValueFormatStyle::Plus)
            } else {
                render_reflective_value(&mut state, vm, program, value, ValueFormatStyle::Default)
            }
        }
        's' => match &value.data {
            ValueData::String(text) if value.typ == crate::TYPE_STRING => {
                Ok(maybe_truncate(text, flags.precision))
            }
            ValueData::String(text) => {
                if let Some(rendered) = render_display_text(vm, program, value)? {
                    Ok(maybe_truncate(&rendered, flags.precision))
                } else {
                    Ok(maybe_truncate(text, flags.precision))
                }
            }
            _ => {
                let rendered = render_string_like_text(&mut state, vm, program, value)?;
                Ok(maybe_truncate(&rendered, flags.precision))
            }
        },
        'q' => match &value.data {
            ValueData::String(text) if value.typ == crate::TYPE_STRING => Ok(format!("{text:?}")),
            ValueData::String(text) => {
                if let Some(rendered) = render_display_text(vm, program, value)? {
                    Ok(format!("{rendered:?}"))
                } else {
                    Ok(format!("{text:?}"))
                }
            }
            ValueData::Int(number) => {
                if let Some(ch) = char::from_u32(*number as u32) {
                    Ok(format!("'{ch}'"))
                } else {
                    Ok(format!("'\\x{number:02x}'"))
                }
            }
            _ => {
                if let Some(text) = render_display_text(vm, program, value)? {
                    return Ok(format!("{text:?}"));
                }
                Ok(format!("%!q({})", render_print_value(vm, program, value)?))
            }
        },
        _ => Ok(format_value(value)),
    }
}

pub(super) fn render_type_verb(vm: &Vm, program: &Program, value: &Value) -> String {
    if matches!(&value.data, ValueData::Nil) && nil_interface_wrapper(program, value.typ) {
        return "<nil>".into();
    }
    if let Some(info) = value_runtime_type(program, vm, value) {
        if let Some(inventory) = program_type_inventory(program) {
            return render_full_type_string(&info, &inventory);
        }
        return sharp_type_name(&info);
    }
    match value.typ {
        TYPE_INT => "int".into(),
        TYPE_INT64 => "int64".into(),
        TYPE_FLOAT64 => "float64".into(),
        TYPE_STRING => "string".into(),
        TYPE_BOOL => "bool".into(),
        TYPE_ERROR => "error".into(),
        TYPE_FUNCTION => "func()".into(),
        TYPE_TIME => "time.Time".into(),
        TYPE_TIME_TIMER => "time.Timer".into(),
        TYPE_CONTEXT => "context.Context".into(),
        TYPE_SYNC_WAIT_GROUP => "sync.WaitGroup".into(),
        TYPE_SYNC_ONCE => "sync.Once".into(),
        TYPE_SYNC_MUTEX => "sync.Mutex".into(),
        TYPE_SYNC_RW_MUTEX => "sync.RWMutex".into(),
        TYPE_HTTP_HEADER => "http.Header".into(),
        TYPE_HTTP_REQUEST => "http.Request".into(),
        TYPE_HTTP_RESPONSE => "http.Response".into(),
        TYPE_HTTP_CLIENT => "http.Client".into(),
        TYPE_HTTP_REQUEST_BODY => "http.__requestBody".into(),
        TYPE_HTTP_RESPONSE_BODY => "http.__responseBody".into(),
        TYPE_URL => "url.URL".into(),
        TYPE_URL_VALUES => "url.Values".into(),
        TYPE_FS_FILE => "fs.File".into(),
        TYPE_FS_FILE_INFO => "fs.FileInfo".into(),
        TYPE_FS_DIR_ENTRY => "fs.DirEntry".into(),
        TYPE_OS_DIR_FS => "fs.FS".into(),
        TYPE_REGEXP => "regexp.Regexp".into(),
        TYPE_STRINGS_REPLACER => "strings.Replacer".into(),
        TYPE_BASE64_ENCODING => "base64.Encoding".into(),
        _ => "<nil>".into(),
    }
}

pub(super) fn render_pointer_verb(value: &Value) -> Option<String> {
    match &value.data {
        ValueData::Pointer(pointer) => Some(render_pointer_identity(pointer)),
        ValueData::Slice(slice) => Some(if slice.is_nil {
            "0x0".into()
        } else {
            format!(
                "0x{:x}",
                (std::rc::Rc::as_ptr(&slice.values) as usize) ^ slice.start
            )
        }),
        ValueData::Map(map) => Some(if let Some(entries) = &map.entries {
            format!("0x{:x}", std::rc::Rc::as_ptr(entries) as usize)
        } else {
            "0x0".into()
        }),
        ValueData::Channel(channel) => Some(match channel.id {
            Some(id) => format!("0x{id:x}"),
            None => "0x0".into(),
        }),
        ValueData::Function(function) => Some(format!("0x{:x}", function_identity(function))),
        _ => None,
    }
}

fn render_display_text(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
) -> Result<Option<String>, VmError> {
    if let ValueData::Error(err) = &value.data {
        return Ok(Some(err.message.clone()));
    }
    if let Some(text) = try_text_method(vm, program, value, "Error")? {
        return Ok(Some(text));
    }
    try_text_method(vm, program, value, "String")
}

fn render_string_like_text(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    value: &Value,
) -> Result<String, VmError> {
    if let Some(text) = render_display_text(vm, program, value)? {
        return Ok(text);
    }
    render_reflective_value(state, vm, program, value, ValueFormatStyle::Default)
}

fn try_text_method(
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    method: &str,
) -> Result<Option<String>, VmError> {
    if matches!(&value.data, ValueData::Nil) && nil_interface_wrapper(program, value.typ) {
        return Ok(None);
    }
    match vm.invoke_method(program, value.clone(), method, Vec::new()) {
        Ok(rendered) => match rendered.data {
            ValueData::String(text) => Ok(Some(text)),
            _ => Ok(None),
        },
        Err(
            VmError::UnknownMethod { .. }
            | VmError::WrongArgumentCount { .. }
            | VmError::ReturnValueCountMismatch { .. },
        ) => Ok(None),
        Err(error) => Err(error),
    }
}

fn nil_interface_wrapper(program: &Program, typ: crate::TypeId) -> bool {
    typ == crate::TYPE_NIL
        || typ == TYPE_ANY
        || typ == TYPE_ERROR
        || program_type_inventory(program)
            .and_then(|inventory| inventory.type_info_for_type_id(typ))
            .is_some_and(|info| matches!(info.kind, RuntimeTypeKind::Interface))
}

fn render_pointer_identity(pointer: &PointerValue) -> String {
    if pointer.is_nil() {
        return "0x0".into();
    }
    format!("0x{:x}", pointer_target_identity(&pointer.target))
}

fn function_identity(function: &FunctionValue) -> u64 {
    let mut hasher = DefaultHasher::new();
    function.function.hash(&mut hasher);
    for capture in &function.captures {
        capture.typ.hash(&mut hasher);
        format_value(capture).hash(&mut hasher);
    }
    hasher.finish()
}

fn pointer_target_identity(target: &PointerTarget) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash_pointer_target(target, &mut hasher);
    hasher.finish()
}

fn hash_pointer_target(target: &PointerTarget, hasher: &mut DefaultHasher) {
    match target {
        PointerTarget::Nil => {
            "nil".hash(hasher);
        }
        PointerTarget::HeapCell { cell } => {
            "heap".hash(hasher);
            cell.hash(hasher);
        }
        PointerTarget::Local { frame_id, register } => {
            "local".hash(hasher);
            frame_id.hash(hasher);
            register.hash(hasher);
        }
        PointerTarget::Global { global } => {
            "global".hash(hasher);
            global.hash(hasher);
        }
        PointerTarget::ProjectedField { base, field } => {
            "projected_field".hash(hasher);
            hash_pointer_target(base, hasher);
            field.hash(hasher);
        }
        PointerTarget::ProjectedIndex { base, index } => {
            "projected_index".hash(hasher);
            hash_pointer_target(base, hasher);
            index.typ.hash(hasher);
            format_value(index).hash(hasher);
        }
        PointerTarget::LocalField {
            frame_id,
            register,
            field,
        } => {
            "local_field".hash(hasher);
            frame_id.hash(hasher);
            register.hash(hasher);
            field.hash(hasher);
        }
        PointerTarget::GlobalField { global, field } => {
            "global_field".hash(hasher);
            global.hash(hasher);
            field.hash(hasher);
        }
        PointerTarget::LocalIndex {
            frame_id,
            register,
            index,
        } => {
            "local_index".hash(hasher);
            frame_id.hash(hasher);
            register.hash(hasher);
            index.typ.hash(hasher);
            format_value(index).hash(hasher);
        }
        PointerTarget::GlobalIndex { global, index } => {
            "global_index".hash(hasher);
            global.hash(hasher);
            index.typ.hash(hasher);
            format_value(index).hash(hasher);
        }
    }
}

fn render_reflective_value(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    style: ValueFormatStyle,
) -> Result<String, VmError> {
    if matches!(
        value.typ,
        TYPE_SYNC_WAIT_GROUP | TYPE_SYNC_ONCE | TYPE_SYNC_MUTEX | TYPE_SYNC_RW_MUTEX | TYPE_CONTEXT
    ) || value.typ == TYPE_TIME_TIMER
    {
        return Ok(format_value(value));
    }
    let Some(info) = value_runtime_type(program, vm, value) else {
        return Ok(format_value(value));
    };
    match (&value.data, &info.kind) {
        (ValueData::Array(array), RuntimeTypeKind::Array) => {
            render_array_like(state, vm, program, &array.values_snapshot(), &info, style)
        }
        (ValueData::Slice(slice), RuntimeTypeKind::Slice) => {
            if style == ValueFormatStyle::Sharp && slice.is_nil {
                Ok(format!("{}(nil)", sharp_type_name(&info)))
            } else {
                render_array_like(state, vm, program, &slice.values_snapshot(), &info, style)
            }
        }
        (ValueData::Map(map), RuntimeTypeKind::Map) => {
            render_map_like(state, vm, program, map, &info, style)
        }
        (ValueData::Struct(fields), RuntimeTypeKind::Struct) => {
            render_struct_like(state, vm, program, fields, &info, style)
        }
        (ValueData::Pointer(pointer), RuntimeTypeKind::Pointer) => {
            render_pointer_like(state, vm, program, value, pointer, &info, style)
        }
        _ => Ok(render_scalar_like(value, &info, style)),
    }
}

fn render_array_like(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    values: &[Value],
    info: &RuntimeTypeInfo,
    style: ValueFormatStyle,
) -> Result<String, VmError> {
    let rendered = render_items(state, vm, program, values, style)?;
    match style {
        ValueFormatStyle::Sharp => Ok(format!(
            "{}{{{}}}",
            sharp_type_name(info),
            rendered.join(", ")
        )),
        ValueFormatStyle::Default | ValueFormatStyle::Plus => {
            Ok(format!("[{}]", rendered.join(" ")))
        }
    }
}

fn render_map_like(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    map: &crate::MapValue,
    info: &RuntimeTypeInfo,
    style: ValueFormatStyle,
) -> Result<String, VmError> {
    let Some(entries) = &map.entries else {
        return Ok(match style {
            ValueFormatStyle::Sharp => format!("{}(nil)", sharp_type_name(info)),
            ValueFormatStyle::Default | ValueFormatStyle::Plus => "map[]".into(),
        });
    };
    let entries = entries.borrow();
    let mut rendered = Vec::with_capacity(entries.len());
    for (key, value) in entries.iter() {
        let key = render_reflective_value(state, vm, program, key, style)?;
        let value = render_reflective_value(state, vm, program, value, style)?;
        rendered.push((key, value));
    }
    let joined = match style {
        ValueFormatStyle::Sharp => rendered
            .iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join(", "),
        ValueFormatStyle::Default | ValueFormatStyle::Plus => rendered
            .iter()
            .map(|(key, value)| format!("{key}:{value}"))
            .collect::<Vec<_>>()
            .join(" "),
    };
    Ok(match style {
        ValueFormatStyle::Sharp => format!("{}{{{joined}}}", sharp_type_name(info)),
        ValueFormatStyle::Default | ValueFormatStyle::Plus => format!("map[{joined}]"),
    })
}

fn render_struct_like(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    fields: &[(String, Value)],
    info: &RuntimeTypeInfo,
    style: ValueFormatStyle,
) -> Result<String, VmError> {
    let mut rendered = Vec::with_capacity(fields.len());
    for (name, value) in fields {
        let value = render_reflective_value(state, vm, program, value, style)?;
        rendered.push((name.as_str(), value));
    }
    let joined = match style {
        ValueFormatStyle::Default => rendered
            .iter()
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>()
            .join(" "),
        ValueFormatStyle::Plus => rendered
            .iter()
            .map(|(name, value)| format!("{name}:{value}"))
            .collect::<Vec<_>>()
            .join(" "),
        ValueFormatStyle::Sharp => rendered
            .iter()
            .map(|(name, value)| format!("{name}:{value}"))
            .collect::<Vec<_>>()
            .join(", "),
    };
    Ok(match style {
        ValueFormatStyle::Sharp => format!("{}{{{joined}}}", sharp_type_name(info)),
        ValueFormatStyle::Default | ValueFormatStyle::Plus => format!("{{{joined}}}"),
    })
}

fn render_pointer_like(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    value: &Value,
    pointer: &crate::PointerValue,
    info: &RuntimeTypeInfo,
    style: ValueFormatStyle,
) -> Result<String, VmError> {
    if pointer.is_nil() {
        return Ok(match style {
            ValueFormatStyle::Sharp => format!("({})(nil)", sharp_type_name(info)),
            ValueFormatStyle::Default | ValueFormatStyle::Plus => "<nil>".into(),
        });
    }
    let dereferenced = vm.deref_pointer(program, value)?;
    let Some(target_info) = value_runtime_type(program, vm, &dereferenced) else {
        return Ok(format_value(value));
    };
    if !matches!(
        &target_info.kind,
        RuntimeTypeKind::Array
            | RuntimeTypeKind::Slice
            | RuntimeTypeKind::Map
            | RuntimeTypeKind::Struct
    ) {
        return Ok(format_value(value));
    }
    if !state.enter_pointer_target(&pointer.target) {
        return Ok(render_recursive_pointer(&target_info, style));
    }
    let rendered = render_reflective_value(state, vm, program, &dereferenced, style);
    state.exit_pointer_target();
    Ok(format!("&{}", rendered?))
}

fn render_recursive_pointer(info: &RuntimeTypeInfo, style: ValueFormatStyle) -> String {
    match style {
        ValueFormatStyle::Sharp => format!("&{}{{...}}", sharp_type_name(info)),
        ValueFormatStyle::Default | ValueFormatStyle::Plus => match &info.kind {
            RuntimeTypeKind::Struct => "&{...}".into(),
            RuntimeTypeKind::Array | RuntimeTypeKind::Slice => "&[...]".into(),
            RuntimeTypeKind::Map => "&map[...]".into(),
            _ => "&<recursive>".into(),
        },
    }
}

fn render_scalar_like(value: &Value, info: &RuntimeTypeInfo, style: ValueFormatStyle) -> String {
    if style == ValueFormatStyle::Sharp {
        if let ValueData::String(text) = &value.data {
            return format!("{text:?}");
        }
        if matches!(&value.data, ValueData::Nil)
            && matches!(
                &info.kind,
                RuntimeTypeKind::Slice | RuntimeTypeKind::Map | RuntimeTypeKind::Pointer
            )
        {
            return format!("{}(nil)", sharp_type_name(info));
        }
    }
    format_value(value)
}

fn sharp_type_name(info: &RuntimeTypeInfo) -> String {
    if info.display_name.contains('.') {
        return info.display_name.clone();
    }
    if let Some(pointer_name) = info.display_name.strip_prefix('*') {
        if !pointer_name.contains('.') {
            if let Some(package_path) = info.package_path.as_deref() {
                return format!("*{}.{}", short_package_name(package_path), pointer_name);
            }
        }
    }
    if let Some(package_path) = info.package_path.as_deref() {
        return format!("{}.{}", short_package_name(package_path), info.display_name);
    }
    info.display_name.clone()
}

fn render_full_type_string(
    info: &RuntimeTypeInfo,
    inventory: &crate::ProgramTypeInventory,
) -> String {
    if is_named_type_name(info.display_name.as_str()) {
        if info.display_name.contains('.') || is_predeclared_named_type(info.display_name.as_str())
        {
            return info.display_name.clone();
        }
        if let Some(package_path) = info.package_path.as_deref() {
            return format!("{}.{}", short_package_name(package_path), info.display_name);
        }
        return info.display_name.clone();
    }

    match info.kind {
        RuntimeTypeKind::Array => format!(
            "[{}]{}",
            info.len.unwrap_or_default(),
            render_concrete_type_string(info.elem.as_deref(), inventory)
        ),
        RuntimeTypeKind::Slice => {
            format!(
                "[]{}",
                render_concrete_type_string(info.elem.as_deref(), inventory)
            )
        }
        RuntimeTypeKind::Map => format!(
            "map[{}]{}",
            render_concrete_type_string(info.key.as_deref(), inventory),
            render_concrete_type_string(info.elem.as_deref(), inventory)
        ),
        RuntimeTypeKind::Pointer => format!(
            "*{}",
            render_concrete_type_string(info.elem.as_deref(), inventory)
        ),
        RuntimeTypeKind::Function => render_function_type_string(info, inventory),
        RuntimeTypeKind::Channel => render_channel_type_string(info, inventory),
        _ => info.display_name.clone(),
    }
}

fn render_concrete_type_string(
    typ: Option<&crate::ConcreteType>,
    inventory: &crate::ProgramTypeInventory,
) -> String {
    typ.and_then(|typ| inventory.resolve_concrete_type(typ))
        .map(|info| render_full_type_string(&info, inventory))
        .unwrap_or_else(|| "<nil>".into())
}

fn render_function_type_string(
    info: &RuntimeTypeInfo,
    inventory: &crate::ProgramTypeInventory,
) -> String {
    let params = info
        .params
        .iter()
        .map(|typ| render_concrete_type_string(Some(typ), inventory))
        .collect::<Vec<_>>()
        .join(", ");
    match info.results.as_slice() {
        [] => format!("func({params})"),
        [result] => format!(
            "func({params}) {}",
            render_concrete_type_string(Some(result), inventory)
        ),
        many => format!(
            "func({params}) ({})",
            many.iter()
                .map(|typ| render_concrete_type_string(Some(typ), inventory))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_channel_type_string(
    info: &RuntimeTypeInfo,
    inventory: &crate::ProgramTypeInventory,
) -> String {
    let element = render_concrete_type_string(info.elem.as_deref(), inventory);
    match info.channel_direction {
        Some(crate::RuntimeChannelDirection::SendOnly) => format!("chan<- {element}"),
        Some(crate::RuntimeChannelDirection::ReceiveOnly) => format!("<-chan {element}"),
        _ => format!("chan {element}"),
    }
}

fn is_named_type_name(display_name: &str) -> bool {
    !(display_name == "interface{}"
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
        || display_name.starts_with("__gowasm_func__("))
}

fn is_predeclared_named_type(display_name: &str) -> bool {
    matches!(
        display_name,
        "bool" | "error" | "float64" | "int" | "string"
    )
}

fn short_package_name(package_path: &str) -> &str {
    if package_path == "." {
        return "main";
    }
    package_path.rsplit('/').next().unwrap_or(package_path)
}

fn render_items(
    state: &mut RenderState,
    vm: &mut Vm,
    program: &Program,
    values: &[Value],
    style: ValueFormatStyle,
) -> Result<Vec<String>, VmError> {
    let mut rendered = Vec::with_capacity(values.len());
    for value in values {
        rendered.push(render_reflective_value(state, vm, program, value, style)?);
    }
    Ok(rendered)
}
