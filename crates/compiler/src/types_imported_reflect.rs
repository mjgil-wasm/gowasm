use std::collections::HashMap;

use gowasm_parser::{InterfaceMethodDecl, Parameter, TypeFieldDecl};
use gowasm_vm::{TYPE_REFLECT_STRUCT_FIELD, TYPE_REFLECT_TYPE, TYPE_REFLECT_VALUE};

use super::{InterfaceTypeDef, StructTypeDef};

pub(super) fn extend_reflect_structs(structs: &mut HashMap<String, StructTypeDef>) {
    structs.extend([(
        "reflect.StructField".into(),
        StructTypeDef {
            type_id: TYPE_REFLECT_STRUCT_FIELD,
            fields: vec![
                TypeFieldDecl {
                    name: "Name".into(),
                    typ: "string".into(),
                    embedded: false,
                    tag: None,
                },
                TypeFieldDecl {
                    name: "PkgPath".into(),
                    typ: "string".into(),
                    embedded: false,
                    tag: None,
                },
                TypeFieldDecl {
                    name: "Type".into(),
                    typ: "reflect.Type".into(),
                    embedded: false,
                    tag: None,
                },
                TypeFieldDecl {
                    name: "Tag".into(),
                    typ: "reflect.StructTag".into(),
                    embedded: false,
                    tag: None,
                },
                TypeFieldDecl {
                    name: "Anonymous".into(),
                    typ: "bool".into(),
                    embedded: false,
                    tag: None,
                },
            ],
        },
    )]);
}

pub(super) fn extend_reflect_interfaces(interfaces: &mut HashMap<String, InterfaceTypeDef>) {
    interfaces.extend([
        (
            "reflect.Type".into(),
            InterfaceTypeDef {
                type_id: TYPE_REFLECT_TYPE,
                methods: vec![
                    no_arg_method("Kind", vec!["reflect.Kind".into()]),
                    no_arg_method("String", vec!["string".into()]),
                    no_arg_method("Name", vec!["string".into()]),
                    no_arg_method("PkgPath", vec!["string".into()]),
                    no_arg_method("Elem", vec!["reflect.Type".into()]),
                    no_arg_method("Key", vec!["reflect.Type".into()]),
                    no_arg_method("Len", vec!["int".into()]),
                    no_arg_method("NumField", vec!["int".into()]),
                    no_arg_method("Comparable", vec!["bool".into()]),
                    no_arg_method("Bits", vec!["int".into()]),
                    no_arg_method("NumIn", vec!["int".into()]),
                    indexed_type_method("In"),
                    no_arg_method("NumOut", vec!["int".into()]),
                    indexed_type_method("Out"),
                    InterfaceMethodDecl {
                        name: "Field".into(),
                        params: vec![Parameter {
                            name: "i".into(),
                            typ: "int".into(),
                            variadic: false,
                        }],
                        result_types: vec!["reflect.StructField".into()],
                    },
                ],
            },
        ),
        (
            "reflect.Value".into(),
            InterfaceTypeDef {
                type_id: TYPE_REFLECT_VALUE,
                methods: vec![
                    no_arg_method("Kind", vec!["reflect.Kind".into()]),
                    no_arg_method("Type", vec!["reflect.Type".into()]),
                    no_arg_method("IsValid", vec!["bool".into()]),
                    no_arg_method("CanInterface", vec!["bool".into()]),
                    no_arg_method("IsNil", vec!["bool".into()]),
                    no_arg_method("Len", vec!["int".into()]),
                    no_arg_method("NumField", vec!["int".into()]),
                    no_arg_method("Bool", vec!["bool".into()]),
                    no_arg_method("Int", vec!["int".into()]),
                    no_arg_method("Float", vec!["float64".into()]),
                    no_arg_method("String", vec!["string".into()]),
                    no_arg_method("Elem", vec!["reflect.Value".into()]),
                    InterfaceMethodDecl {
                        name: "Index".into(),
                        params: vec![Parameter {
                            name: "i".into(),
                            typ: "int".into(),
                            variadic: false,
                        }],
                        result_types: vec!["reflect.Value".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Field".into(),
                        params: vec![Parameter {
                            name: "i".into(),
                            typ: "int".into(),
                            variadic: false,
                        }],
                        result_types: vec!["reflect.Value".into()],
                    },
                    InterfaceMethodDecl {
                        name: "MapIndex".into(),
                        params: vec![Parameter {
                            name: "key".into(),
                            typ: "interface{}".into(),
                            variadic: false,
                        }],
                        result_types: vec!["reflect.Value".into()],
                    },
                    no_arg_method("MapKeys", vec!["[]reflect.Value".into()]),
                    no_arg_method("Interface", vec!["interface{}".into()]),
                ],
            },
        ),
    ]);
}

fn no_arg_method(name: &str, result_types: Vec<String>) -> InterfaceMethodDecl {
    InterfaceMethodDecl {
        name: name.into(),
        params: Vec::new(),
        result_types,
    }
}

fn indexed_type_method(name: &str) -> InterfaceMethodDecl {
    InterfaceMethodDecl {
        name: name.into(),
        params: vec![Parameter {
            name: "i".into(),
            typ: "int".into(),
            variadic: false,
        }],
        result_types: vec!["reflect.Type".into()],
    }
}
