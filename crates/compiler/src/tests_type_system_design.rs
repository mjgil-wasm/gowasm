use gowasm_parser::{parse_source_file, TypeDeclKind};
use gowasm_vm::{
    program_type_inventory, value_runtime_type, ConcreteType, RuntimeTypeInfo, RuntimeTypeKind,
    SliceValue, Value, ValueData, Vm, TYPE_STRING,
};

use super::compile_source;

fn inventory_entry<'a>(
    inventory: &'a gowasm_vm::ProgramTypeInventory,
    display_name: &str,
) -> &'a RuntimeTypeInfo {
    inventory
        .types_by_id
        .values()
        .find(|info| info.display_name == display_name)
        .unwrap_or_else(|| panic!("runtime type inventory should contain `{display_name}`"))
}

#[test]
fn parser_compiler_and_runtime_type_identity_stay_aligned() {
    let source = r#"
package main

type Labels []string

type Reader interface {
    Read() string
}

type Box[T any] struct {
    Value T `json:"value"`
}

type Handler func(Labels) Box[int]

func main() {
    var labels Labels
    var reader Reader
    var box Box[int]
    var handler Handler
    _ = labels
    _ = reader
    _ = box
    _ = handler
}
"#;

    let parsed = parse_source_file(source).expect("source should parse");
    assert_eq!(parsed.types.len(), 4);
    match &parsed.types[0].kind {
        TypeDeclKind::Alias { underlying } => assert_eq!(underlying, "[]string"),
        other => panic!("expected alias type, got {other:?}"),
    }
    match &parsed.types[1].kind {
        TypeDeclKind::Interface { methods, embeds } => {
            assert!(embeds.is_empty());
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "Read");
            assert_eq!(methods[0].result_types, vec!["string".to_string()]);
        }
        other => panic!("expected interface type, got {other:?}"),
    }
    match &parsed.types[2].kind {
        TypeDeclKind::Struct { fields } => {
            assert_eq!(parsed.types[2].type_params[0].name, "T");
            assert_eq!(fields[0].name, "Value");
            assert_eq!(fields[0].typ, "T");
            assert_eq!(fields[0].tag.as_deref(), Some(r#"json:"value""#));
        }
        other => panic!("expected struct type, got {other:?}"),
    }
    match &parsed.types[3].kind {
        TypeDeclKind::Alias { underlying } => {
            assert_eq!(underlying, "__gowasm_func__(Labels)->(Box[int])");
        }
        other => panic!("expected function alias type, got {other:?}"),
    }

    let program = compile_source(source).expect("program should compile");
    let inventory =
        program_type_inventory(&program).expect("compiled program should register type inventory");

    let labels = inventory_entry(&inventory, "Labels");
    let labels_type_id = labels.type_id.expect("Labels should have a type id");
    assert_eq!(labels.kind, RuntimeTypeKind::Slice);
    assert_eq!(
        labels.underlying,
        Some(Box::new(ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
        }))
    );

    let reader = inventory_entry(&inventory, "Reader");
    let reader_type_id = reader.type_id.expect("Reader should have a type id");
    assert_eq!(reader.kind, RuntimeTypeKind::Interface);
    assert_eq!(reader.package_path.as_deref(), Some("main"));

    let boxed = inventory_entry(&inventory, "Box[int]");
    let boxed_type_id = boxed.type_id.expect("Box[int] should have a type id");
    assert_eq!(boxed.kind, RuntimeTypeKind::Struct);
    assert_eq!(boxed.fields.len(), 1);
    assert_eq!(boxed.fields[0].tag.as_deref(), Some(r#"json:"value""#));

    let handler = inventory_entry(&inventory, "Handler");
    let handler_type_id = handler.type_id.expect("Handler should have a type id");
    assert_eq!(handler.kind, RuntimeTypeKind::Function);
    assert_eq!(handler.params, vec![ConcreteType::TypeId(labels_type_id)]);
    assert_eq!(handler.results, vec![ConcreteType::TypeId(boxed_type_id)]);

    let labels_value = Value {
        typ: labels_type_id,
        data: ValueData::Slice(SliceValue {
            values: Rc::new(RefCell::new(vec![Value::string("x")])),
            start: 0,
            len: 1,
            cap: 1,
            is_nil: false,
            concrete_type: Some(ConcreteType::Slice {
                element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
            }),
        }),
    };
    let labels_runtime = value_runtime_type(&program, &Vm::new(), &labels_value)
        .expect("runtime identity should resolve Labels");
    assert_eq!(labels_runtime.display_name, "Labels");

    let reader_runtime = value_runtime_type(
        &program,
        &Vm::new(),
        &Value {
            typ: reader_type_id,
            data: ValueData::Nil,
        },
    )
    .expect("runtime identity should preserve typed nil interfaces");
    assert_eq!(reader_runtime.display_name, "Reader");

    let boxed_runtime = value_runtime_type(
        &program,
        &Vm::new(),
        &Value::struct_value(boxed_type_id, vec![("Value".into(), Value::int(7))]),
    )
    .expect("runtime identity should resolve Box[int]");
    assert_eq!(boxed_runtime.display_name, "Box[int]");

    let handler_runtime = value_runtime_type(
        &program,
        &Vm::new(),
        &Value::function_typed(0, Vec::new(), ConcreteType::TypeId(handler_type_id)),
    )
    .expect("runtime identity should resolve Handler");
    assert_eq!(handler_runtime.display_name, "Handler");
}

#[test]
fn reflect_and_json_type_identity_match_compiled_inventory() {
    let source = r#"
package main

import (
    "encoding/json"
    "fmt"
    "reflect"
)

type Labels []string

type Box[T any] struct {
    Value T `json:"value"`
}

type Handler func(Labels) Box[int]

func makeBox(labels Labels) Box[int] {
    var result Box[int]
    result.Value = len(labels)
    return result
}

func main() {
    var labels Labels
    labels = append(labels, "x")
    var boxValue Box[int]
    boxValue.Value = 7
    labelsType := reflect.TypeOf(labels)
    boxType := reflect.TypeOf(boxValue)
    ptrType := reflect.TypeOf(&boxValue)
    handler := Handler(makeBox)
    handlerType := reflect.TypeOf(handler)
    payload, err := json.Marshal(boxValue)

    fmt.Println(labelsType.Kind().String(), labelsType.Elem().String())
    fmt.Println(boxType.String(), boxType.Field(0).Type.String(), boxType.Field(0).Tag)
    fmt.Println(ptrType.String(), ptrType.Elem().String())
    fmt.Println(handlerType.NumIn(), handlerType.In(0).String(), handlerType.NumOut(), handlerType.Out(0).String())
    fmt.Println(string(payload), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "slice string\n",
            "main.Box[int] int json:\"value\"\n",
            "*main.Box[int] main.Box[int]\n",
            "1 main.Labels 1 main.Box[int]\n",
            "{\"value\":7} <nil>\n",
        )
    );
}
use std::cell::RefCell;
use std::rc::Rc;
