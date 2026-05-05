use super::{
    unsupported_multi_result_stdlib, StdlibConstant, StdlibConstantValue, StdlibFunction,
    StdlibMethod, REFLECT_KIND_STRING, REFLECT_STRUCT_TAG_GET, REFLECT_STRUCT_TAG_LOOKUP,
    REFLECT_TYPE_BITS, REFLECT_TYPE_COMPARABLE, REFLECT_TYPE_ELEM, REFLECT_TYPE_FIELD,
    REFLECT_TYPE_IN, REFLECT_TYPE_KEY, REFLECT_TYPE_KIND, REFLECT_TYPE_LEN, REFLECT_TYPE_NAME,
    REFLECT_TYPE_NUM_FIELD, REFLECT_TYPE_NUM_IN, REFLECT_TYPE_NUM_OUT, REFLECT_TYPE_OF,
    REFLECT_TYPE_OUT, REFLECT_TYPE_PKG_PATH, REFLECT_TYPE_STRING, REFLECT_VALUE_BOOL,
    REFLECT_VALUE_CAN_INTERFACE, REFLECT_VALUE_ELEM, REFLECT_VALUE_FIELD, REFLECT_VALUE_FLOAT,
    REFLECT_VALUE_INDEX, REFLECT_VALUE_INT, REFLECT_VALUE_INTERFACE, REFLECT_VALUE_IS_NIL,
    REFLECT_VALUE_IS_VALID, REFLECT_VALUE_KIND, REFLECT_VALUE_LEN, REFLECT_VALUE_MAP_INDEX,
    REFLECT_VALUE_MAP_KEYS, REFLECT_VALUE_NUM_FIELD, REFLECT_VALUE_OF, REFLECT_VALUE_STRING,
    REFLECT_VALUE_TYPE,
};
use crate::{
    program_type_inventory, ConcreteType, Program, ProgramTypeInventory, RuntimeChannelDirection,
    RuntimeTypeField, RuntimeTypeInfo, RuntimeTypeKind, TypeId, Value, ValueData, Vm, VmError,
    TYPE_ANY, TYPE_EMPTY_STRUCT, TYPE_ERROR, TYPE_INT, TYPE_NIL, TYPE_REFLECT_KIND,
    TYPE_REFLECT_RTYPE, TYPE_REFLECT_RVALUE, TYPE_REFLECT_STRUCT_FIELD, TYPE_REFLECT_VALUE,
};

#[path = "reflect_helpers.rs"]
mod helpers;
#[path = "reflect_struct_tag.rs"]
mod struct_tag_surface;
#[path = "reflect_type_refs.rs"]
mod type_refs;
#[path = "reflect_value.rs"]
mod value_surface;

use helpers::*;
use struct_tag_surface::reflect_struct_tag_get;
use type_refs::*;
use value_surface::{
    reflect_value_bool, reflect_value_can_interface_method, reflect_value_elem,
    reflect_value_field, reflect_value_float, reflect_value_index, reflect_value_int,
    reflect_value_interface, reflect_value_is_nil, reflect_value_is_valid, reflect_value_kind,
    reflect_value_len, reflect_value_map_index, reflect_value_map_keys, reflect_value_num_field,
    reflect_value_of, reflect_value_string, reflect_value_type,
};

pub(super) const REFLECT_KIND_RECEIVER: &str = "reflect.Kind";
pub(super) const REFLECT_STRUCT_TAG_RECEIVER: &str = "reflect.StructTag";
pub(super) const REFLECT_TYPE_RECEIVER: &str = "reflect.__type";
pub(super) const REFLECT_VALUE_RECEIVER: &str = "reflect.__value";
pub(super) use super::strconv_helpers_impl::unquote as stdlib_unquote;
pub(super) use struct_tag_surface::reflect_struct_tag_lookup;

pub(super) const TYPE_STRING_FIELD: &str = "__string";
pub(super) const TYPE_NAME_FIELD: &str = "__name";
pub(super) const TYPE_PKG_PATH_FIELD: &str = "__pkgPath";
pub(super) const TYPE_KIND_FIELD: &str = "__kind";
pub(super) const TYPE_ELEM_FIELD: &str = "__elem";
pub(super) const TYPE_KEY_FIELD: &str = "__key";
pub(super) const TYPE_LEN_FIELD: &str = "__len";
pub(super) const TYPE_HAS_LEN_FIELD: &str = "__hasLen";
pub(super) const TYPE_FIELDS_FIELD: &str = "__fields";
pub(super) const TYPE_BITS_FIELD: &str = "__bits";
pub(super) const TYPE_HAS_BITS_FIELD: &str = "__hasBits";
pub(super) const TYPE_COMPARABLE_FIELD: &str = "__comparable";
pub(super) const TYPE_PARAMS_FIELD: &str = "__params";
pub(super) const TYPE_RESULTS_FIELD: &str = "__results";

pub(super) const VALUE_VALUE_FIELD: &str = "__value";
pub(super) const VALUE_TYPE_FIELD: &str = "__type";
pub(super) const VALUE_VALID_FIELD: &str = "__valid";
pub(super) const VALUE_CAN_INTERFACE_FIELD: &str = "__canInterface";

pub(super) const KIND_INVALID: i64 = 0;
pub(super) const KIND_BOOL: i64 = 1;
pub(super) const KIND_INT: i64 = 2;
pub(super) const KIND_FLOAT64: i64 = 14;
pub(super) const KIND_ARRAY: i64 = 17;
pub(super) const KIND_CHAN: i64 = 18;
pub(super) const KIND_FUNC: i64 = 19;
pub(super) const KIND_INTERFACE: i64 = 20;
pub(super) const KIND_MAP: i64 = 21;
pub(super) const KIND_PTR: i64 = 22;
pub(super) const KIND_SLICE: i64 = 23;
pub(super) const KIND_STRING: i64 = 24;
pub(super) const KIND_STRUCT: i64 = 25;

pub(super) const REFLECT_CONSTANTS: &[StdlibConstant] = &[
    reflect_kind_constant("Invalid", KIND_INVALID),
    reflect_kind_constant("Bool", KIND_BOOL),
    reflect_kind_constant("Int", KIND_INT),
    reflect_kind_constant("Float64", KIND_FLOAT64),
    reflect_kind_constant("Array", KIND_ARRAY),
    reflect_kind_constant("Chan", KIND_CHAN),
    reflect_kind_constant("Func", KIND_FUNC),
    reflect_kind_constant("Interface", KIND_INTERFACE),
    reflect_kind_constant("Map", KIND_MAP),
    reflect_kind_constant("Ptr", KIND_PTR),
    reflect_kind_constant("Slice", KIND_SLICE),
    reflect_kind_constant("String", KIND_STRING),
    reflect_kind_constant("Struct", KIND_STRUCT),
];

pub(super) const REFLECT_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: REFLECT_TYPE_OF,
        symbol: "TypeOf",
        returns_value: true,
        handler: reflect_type_of,
    },
    StdlibFunction {
        id: REFLECT_VALUE_OF,
        symbol: "ValueOf",
        returns_value: true,
        handler: reflect_value_of,
    },
];

pub(super) const REFLECT_METHODS: &[StdlibMethod] = &[
    StdlibMethod {
        receiver_type: REFLECT_KIND_RECEIVER,
        method: "String",
        function: REFLECT_KIND_STRING,
    },
    StdlibMethod {
        receiver_type: REFLECT_STRUCT_TAG_RECEIVER,
        method: "Get",
        function: REFLECT_STRUCT_TAG_GET,
    },
    StdlibMethod {
        receiver_type: REFLECT_STRUCT_TAG_RECEIVER,
        method: "Lookup",
        function: REFLECT_STRUCT_TAG_LOOKUP,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Kind",
        function: REFLECT_TYPE_KIND,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "String",
        function: REFLECT_TYPE_STRING,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Name",
        function: REFLECT_TYPE_NAME,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "PkgPath",
        function: REFLECT_TYPE_PKG_PATH,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Elem",
        function: REFLECT_TYPE_ELEM,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Key",
        function: REFLECT_TYPE_KEY,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Len",
        function: REFLECT_TYPE_LEN,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "NumField",
        function: REFLECT_TYPE_NUM_FIELD,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Field",
        function: REFLECT_TYPE_FIELD,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Comparable",
        function: REFLECT_TYPE_COMPARABLE,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Bits",
        function: REFLECT_TYPE_BITS,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "NumIn",
        function: REFLECT_TYPE_NUM_IN,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "In",
        function: REFLECT_TYPE_IN,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "NumOut",
        function: REFLECT_TYPE_NUM_OUT,
    },
    StdlibMethod {
        receiver_type: REFLECT_TYPE_RECEIVER,
        method: "Out",
        function: REFLECT_TYPE_OUT,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Kind",
        function: REFLECT_VALUE_KIND,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Type",
        function: REFLECT_VALUE_TYPE,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "IsValid",
        function: REFLECT_VALUE_IS_VALID,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "CanInterface",
        function: REFLECT_VALUE_CAN_INTERFACE,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "IsNil",
        function: REFLECT_VALUE_IS_NIL,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Len",
        function: REFLECT_VALUE_LEN,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "NumField",
        function: REFLECT_VALUE_NUM_FIELD,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Bool",
        function: REFLECT_VALUE_BOOL,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Int",
        function: REFLECT_VALUE_INT,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Float",
        function: REFLECT_VALUE_FLOAT,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "String",
        function: REFLECT_VALUE_STRING,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Elem",
        function: REFLECT_VALUE_ELEM,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Index",
        function: REFLECT_VALUE_INDEX,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Field",
        function: REFLECT_VALUE_FIELD,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "MapIndex",
        function: REFLECT_VALUE_MAP_INDEX,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "MapKeys",
        function: REFLECT_VALUE_MAP_KEYS,
    },
    StdlibMethod {
        receiver_type: REFLECT_VALUE_RECEIVER,
        method: "Interface",
        function: REFLECT_VALUE_INTERFACE,
    },
];

pub(super) const REFLECT_METHOD_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: REFLECT_KIND_STRING,
        symbol: "String",
        returns_value: true,
        handler: reflect_kind_string,
    },
    StdlibFunction {
        id: REFLECT_STRUCT_TAG_GET,
        symbol: "Get",
        returns_value: true,
        handler: reflect_struct_tag_get,
    },
    StdlibFunction {
        id: REFLECT_STRUCT_TAG_LOOKUP,
        symbol: "Lookup",
        returns_value: false,
        handler: unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: REFLECT_TYPE_KIND,
        symbol: "Kind",
        returns_value: true,
        handler: reflect_type_kind,
    },
    StdlibFunction {
        id: REFLECT_TYPE_STRING,
        symbol: "String",
        returns_value: true,
        handler: reflect_type_string,
    },
    StdlibFunction {
        id: REFLECT_TYPE_NAME,
        symbol: "Name",
        returns_value: true,
        handler: reflect_type_name,
    },
    StdlibFunction {
        id: REFLECT_TYPE_PKG_PATH,
        symbol: "PkgPath",
        returns_value: true,
        handler: reflect_type_pkg_path,
    },
    StdlibFunction {
        id: REFLECT_TYPE_ELEM,
        symbol: "Elem",
        returns_value: true,
        handler: reflect_type_elem,
    },
    StdlibFunction {
        id: REFLECT_TYPE_KEY,
        symbol: "Key",
        returns_value: true,
        handler: reflect_type_key,
    },
    StdlibFunction {
        id: REFLECT_TYPE_LEN,
        symbol: "Len",
        returns_value: true,
        handler: reflect_type_len,
    },
    StdlibFunction {
        id: REFLECT_TYPE_NUM_FIELD,
        symbol: "NumField",
        returns_value: true,
        handler: reflect_type_num_field,
    },
    StdlibFunction {
        id: REFLECT_TYPE_FIELD,
        symbol: "Field",
        returns_value: true,
        handler: reflect_type_field,
    },
    StdlibFunction {
        id: REFLECT_TYPE_COMPARABLE,
        symbol: "Comparable",
        returns_value: true,
        handler: reflect_type_comparable,
    },
    StdlibFunction {
        id: REFLECT_TYPE_BITS,
        symbol: "Bits",
        returns_value: true,
        handler: reflect_type_bits,
    },
    StdlibFunction {
        id: REFLECT_TYPE_NUM_IN,
        symbol: "NumIn",
        returns_value: true,
        handler: reflect_type_num_in,
    },
    StdlibFunction {
        id: REFLECT_TYPE_IN,
        symbol: "In",
        returns_value: true,
        handler: reflect_type_in,
    },
    StdlibFunction {
        id: REFLECT_TYPE_NUM_OUT,
        symbol: "NumOut",
        returns_value: true,
        handler: reflect_type_num_out,
    },
    StdlibFunction {
        id: REFLECT_TYPE_OUT,
        symbol: "Out",
        returns_value: true,
        handler: reflect_type_out,
    },
    StdlibFunction {
        id: REFLECT_VALUE_KIND,
        symbol: "Kind",
        returns_value: true,
        handler: reflect_value_kind,
    },
    StdlibFunction {
        id: REFLECT_VALUE_TYPE,
        symbol: "Type",
        returns_value: true,
        handler: reflect_value_type,
    },
    StdlibFunction {
        id: REFLECT_VALUE_IS_VALID,
        symbol: "IsValid",
        returns_value: true,
        handler: reflect_value_is_valid,
    },
    StdlibFunction {
        id: REFLECT_VALUE_CAN_INTERFACE,
        symbol: "CanInterface",
        returns_value: true,
        handler: reflect_value_can_interface_method,
    },
    StdlibFunction {
        id: REFLECT_VALUE_IS_NIL,
        symbol: "IsNil",
        returns_value: true,
        handler: reflect_value_is_nil,
    },
    StdlibFunction {
        id: REFLECT_VALUE_LEN,
        symbol: "Len",
        returns_value: true,
        handler: reflect_value_len,
    },
    StdlibFunction {
        id: REFLECT_VALUE_NUM_FIELD,
        symbol: "NumField",
        returns_value: true,
        handler: reflect_value_num_field,
    },
    StdlibFunction {
        id: REFLECT_VALUE_BOOL,
        symbol: "Bool",
        returns_value: true,
        handler: reflect_value_bool,
    },
    StdlibFunction {
        id: REFLECT_VALUE_INT,
        symbol: "Int",
        returns_value: true,
        handler: reflect_value_int,
    },
    StdlibFunction {
        id: REFLECT_VALUE_FLOAT,
        symbol: "Float",
        returns_value: true,
        handler: reflect_value_float,
    },
    StdlibFunction {
        id: REFLECT_VALUE_STRING,
        symbol: "String",
        returns_value: true,
        handler: reflect_value_string,
    },
    StdlibFunction {
        id: REFLECT_VALUE_ELEM,
        symbol: "Elem",
        returns_value: true,
        handler: reflect_value_elem,
    },
    StdlibFunction {
        id: REFLECT_VALUE_INDEX,
        symbol: "Index",
        returns_value: true,
        handler: reflect_value_index,
    },
    StdlibFunction {
        id: REFLECT_VALUE_FIELD,
        symbol: "Field",
        returns_value: true,
        handler: reflect_value_field,
    },
    StdlibFunction {
        id: REFLECT_VALUE_MAP_INDEX,
        symbol: "MapIndex",
        returns_value: true,
        handler: reflect_value_map_index,
    },
    StdlibFunction {
        id: REFLECT_VALUE_MAP_KEYS,
        symbol: "MapKeys",
        returns_value: true,
        handler: reflect_value_map_keys,
    },
    StdlibFunction {
        id: REFLECT_VALUE_INTERFACE,
        symbol: "Interface",
        returns_value: true,
        handler: reflect_value_interface,
    },
];

pub(super) fn reflect_type_of(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let inventory = reflect_inventory(vm, program)?;
    let value = &args[0];
    if matches!(&value.data, ValueData::Nil)
        && (value.typ == TYPE_NIL || is_nil_interface_value(&inventory, value))
    {
        return Ok(Value::nil());
    }
    let info = inventory
        .value_type_info(vm, program, value)
        .ok_or_else(|| reflect_panic(vm, program, "TypeOf cannot describe this value"))?;
    build_reflect_type_value(&info, &inventory, vm, program)
}

fn reflect_kind_string(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let kind = extract_kind_receiver(vm, program, args, "reflect.Kind.String")?;
    Ok(Value::string(reflect_kind_name(kind)))
}

fn reflect_type_kind(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Kind")?;
    hidden_field(receiver, TYPE_KIND_FIELD)
}

fn reflect_type_string(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.String")?;
    hidden_field(receiver, TYPE_STRING_FIELD)
}

fn reflect_type_name(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Name")?;
    hidden_field(receiver, TYPE_NAME_FIELD)
}

fn reflect_type_pkg_path(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.PkgPath")?;
    hidden_field(receiver, TYPE_PKG_PATH_FIELD)
}

fn reflect_type_elem(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Elem")?;
    reflect_type_member(vm, program, receiver, TYPE_ELEM_FIELD, "Elem")
}

fn reflect_type_key(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Key")?;
    reflect_type_member(vm, program, receiver, TYPE_KEY_FIELD, "Key")
}

fn reflect_type_len(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Len")?;
    if !hidden_bool(receiver, TYPE_HAS_LEN_FIELD)? {
        return Err(reflect_panic(vm, program, "Len of non-array type"));
    }
    hidden_field(receiver, TYPE_LEN_FIELD)
}

fn reflect_type_num_field(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.NumField")?;
    if hidden_kind(receiver)? != KIND_STRUCT {
        return Err(reflect_panic(vm, program, "NumField of non-struct type"));
    }
    Ok(Value::int(
        hidden_slice(receiver, TYPE_FIELDS_FIELD)?.len() as i64
    ))
}

fn reflect_type_field(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let receiver = extract_reflect_type_value(vm, program, &args[0], "reflect.Type.Field")?;
    if hidden_kind(receiver)? != KIND_STRUCT {
        return Err(reflect_panic(vm, program, "Field of non-struct type"));
    }
    let ValueData::Int(index) = &args[1].data else {
        return Err(reflect_panic(vm, program, "Field index must be an int"));
    };
    let fields = hidden_slice(receiver, TYPE_FIELDS_FIELD)?;
    if *index < 0 || *index as usize >= fields.len() {
        return Err(reflect_panic(vm, program, "Field index out of range"));
    }
    materialize_reflect_struct_field(vm, program, &fields[*index as usize])
}

fn reflect_type_comparable(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Comparable")?;
    hidden_field(receiver, TYPE_COMPARABLE_FIELD)
}

fn reflect_type_bits(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.Bits")?;
    if !hidden_bool(receiver, TYPE_HAS_BITS_FIELD)? {
        return Err(reflect_panic(vm, program, "Bits of non-arithmetic type"));
    }
    hidden_field(receiver, TYPE_BITS_FIELD)
}

fn reflect_type_num_in(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.NumIn")?;
    if hidden_kind(receiver)? != KIND_FUNC {
        return Err(reflect_panic(vm, program, "NumIn of non-func type"));
    }
    Ok(Value::int(
        hidden_slice(receiver, TYPE_PARAMS_FIELD)?.len() as i64
    ))
}

fn reflect_type_in(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    reflect_type_signature_member(vm, program, args, TYPE_PARAMS_FIELD, "In")
}

fn reflect_type_num_out(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let receiver = extract_reflect_type_receiver(vm, program, args, "reflect.Type.NumOut")?;
    if hidden_kind(receiver)? != KIND_FUNC {
        return Err(reflect_panic(vm, program, "NumOut of non-func type"));
    }
    Ok(Value::int(
        hidden_slice(receiver, TYPE_RESULTS_FIELD)?.len() as i64,
    ))
}

fn reflect_type_out(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    reflect_type_signature_member(vm, program, args, TYPE_RESULTS_FIELD, "Out")
}

fn reflect_type_signature_member(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
    field: &str,
    method: &str,
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let receiver =
        extract_reflect_type_value(vm, program, &args[0], &format!("reflect.Type.{method}"))?;
    if hidden_kind(receiver)? != KIND_FUNC {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} of non-func type"),
        ));
    }
    let ValueData::Int(index) = &args[1].data else {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} index must be an int"),
        ));
    };
    if *index < 0 {
        return Err(reflect_panic(
            vm,
            program,
            &format!("{method} index out of range"),
        ));
    }
    reflect_type_signature_item(vm, program, receiver, field, *index as usize, method)
}

const fn reflect_kind_constant(symbol: &'static str, value: i64) -> StdlibConstant {
    StdlibConstant {
        symbol,
        typ: "reflect.Kind",
        value: StdlibConstantValue::Int(value),
    }
}
