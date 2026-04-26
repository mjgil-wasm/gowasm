use super::{
    resolve_stdlib_constant, resolve_stdlib_function, resolve_stdlib_method,
    resolve_stdlib_runtime_method, stdlib_function_param_types, stdlib_function_result_count,
    stdlib_function_result_types, stdlib_function_returns_value,
};
use crate::{TYPE_REFLECT_KIND, TYPE_REFLECT_RTYPE, TYPE_REFLECT_RVALUE, TYPE_REFLECT_STRUCT_TAG};

#[test]
fn reflect_registry_exposes_first_type_kind_and_struct_tag_surface() {
    let type_of = resolve_stdlib_function("reflect", "TypeOf").expect("reflect.TypeOf exists");
    assert!(stdlib_function_returns_value(type_of));
    assert_eq!(stdlib_function_result_count(type_of), 1);
    assert_eq!(
        stdlib_function_param_types(type_of),
        Some(&["interface{}"][..])
    );
    assert_eq!(
        stdlib_function_result_types(type_of),
        Some(&["reflect.Type"][..])
    );

    let value_of = resolve_stdlib_function("reflect", "ValueOf").expect("reflect.ValueOf exists");
    assert!(stdlib_function_returns_value(value_of));
    assert_eq!(stdlib_function_result_count(value_of), 1);
    assert_eq!(
        stdlib_function_param_types(value_of),
        Some(&["interface{}"][..])
    );
    assert_eq!(
        stdlib_function_result_types(value_of),
        Some(&["reflect.Value"][..])
    );

    let kind_string =
        resolve_stdlib_method("reflect.Kind", "String").expect("reflect.Kind.String exists");
    assert_eq!(
        stdlib_function_param_types(kind_string),
        Some(&["reflect.Kind"][..])
    );
    assert_eq!(
        stdlib_function_result_types(kind_string),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_KIND, "String"),
        Some(kind_string)
    );

    let tag_get =
        resolve_stdlib_method("reflect.StructTag", "Get").expect("reflect.StructTag.Get exists");
    assert!(stdlib_function_returns_value(tag_get));
    assert_eq!(
        stdlib_function_param_types(tag_get),
        Some(&["reflect.StructTag", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(tag_get), Some(&["string"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_STRUCT_TAG, "Get"),
        Some(tag_get)
    );

    let tag_lookup = resolve_stdlib_method("reflect.StructTag", "Lookup")
        .expect("reflect.StructTag.Lookup exists");
    assert!(!stdlib_function_returns_value(tag_lookup));
    assert_eq!(stdlib_function_result_count(tag_lookup), 2);
    assert_eq!(
        stdlib_function_param_types(tag_lookup),
        Some(&["reflect.StructTag", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(tag_lookup),
        Some(&["string", "bool"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_STRUCT_TAG, "Lookup"),
        Some(tag_lookup)
    );

    let field =
        resolve_stdlib_method("reflect.__type", "Field").expect("reflect.Type.Field exists");
    assert_eq!(stdlib_function_result_count(field), 1);
    assert_eq!(
        stdlib_function_param_types(field),
        Some(&["reflect.__type", "int"][..])
    );
    assert_eq!(
        stdlib_function_result_types(field),
        Some(&["reflect.StructField"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "Field"),
        Some(field)
    );

    let comparable = resolve_stdlib_method("reflect.__type", "Comparable")
        .expect("reflect.Type.Comparable exists");
    assert!(stdlib_function_returns_value(comparable));
    assert_eq!(
        stdlib_function_result_types(comparable),
        Some(&["bool"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "Comparable"),
        Some(comparable)
    );

    let bits = resolve_stdlib_method("reflect.__type", "Bits").expect("reflect.Type.Bits exists");
    assert!(stdlib_function_returns_value(bits));
    assert_eq!(stdlib_function_result_types(bits), Some(&["int"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "Bits"),
        Some(bits)
    );

    let num_in =
        resolve_stdlib_method("reflect.__type", "NumIn").expect("reflect.Type.NumIn exists");
    assert!(stdlib_function_returns_value(num_in));
    assert_eq!(stdlib_function_result_types(num_in), Some(&["int"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "NumIn"),
        Some(num_in)
    );

    let in_method = resolve_stdlib_method("reflect.__type", "In").expect("reflect.Type.In exists");
    assert!(stdlib_function_returns_value(in_method));
    assert_eq!(
        stdlib_function_param_types(in_method),
        Some(&["reflect.__type", "int"][..])
    );
    assert_eq!(
        stdlib_function_result_types(in_method),
        Some(&["reflect.Type"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "In"),
        Some(in_method)
    );

    let num_out =
        resolve_stdlib_method("reflect.__type", "NumOut").expect("reflect.Type.NumOut exists");
    assert!(stdlib_function_returns_value(num_out));
    assert_eq!(stdlib_function_result_types(num_out), Some(&["int"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "NumOut"),
        Some(num_out)
    );

    let out_method =
        resolve_stdlib_method("reflect.__type", "Out").expect("reflect.Type.Out exists");
    assert!(stdlib_function_returns_value(out_method));
    assert_eq!(
        stdlib_function_param_types(out_method),
        Some(&["reflect.__type", "int"][..])
    );
    assert_eq!(
        stdlib_function_result_types(out_method),
        Some(&["reflect.Type"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RTYPE, "Out"),
        Some(out_method)
    );

    let interface = resolve_stdlib_method("reflect.__value", "Interface")
        .expect("reflect.Value.Interface exists");
    assert_eq!(
        stdlib_function_param_types(interface),
        Some(&["reflect.__value"][..])
    );
    assert_eq!(
        stdlib_function_result_types(interface),
        Some(&["interface{}"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RVALUE, "Interface"),
        Some(interface)
    );

    let map_keys =
        resolve_stdlib_method("reflect.__value", "MapKeys").expect("reflect.Value.MapKeys exists");
    assert_eq!(
        stdlib_function_result_types(map_keys),
        Some(&["[]reflect.Value"][..])
    );

    let can_interface = resolve_stdlib_method("reflect.__value", "CanInterface")
        .expect("reflect.Value.CanInterface exists");
    assert!(stdlib_function_returns_value(can_interface));
    assert_eq!(
        stdlib_function_param_types(can_interface),
        Some(&["reflect.__value"][..])
    );
    assert_eq!(
        stdlib_function_result_types(can_interface),
        Some(&["bool"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RVALUE, "CanInterface"),
        Some(can_interface)
    );

    let num_field = resolve_stdlib_method("reflect.__value", "NumField")
        .expect("reflect.Value.NumField exists");
    assert!(stdlib_function_returns_value(num_field));
    assert_eq!(
        stdlib_function_param_types(num_field),
        Some(&["reflect.__value"][..])
    );
    assert_eq!(stdlib_function_result_types(num_field), Some(&["int"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_REFLECT_RVALUE, "NumField"),
        Some(num_field)
    );

    let struct_kind = resolve_stdlib_constant("reflect", "Struct").expect("reflect.Struct exists");
    assert_eq!(struct_kind.typ, "reflect.Kind");
    let ptr_kind = resolve_stdlib_constant("reflect", "Ptr").expect("reflect.Ptr exists");
    assert_eq!(ptr_kind.typ, "reflect.Kind");
}
