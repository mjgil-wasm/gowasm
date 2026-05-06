use super::compile_source;
use gowasm_vm::{
    program_type_inventory, ConcreteType, RuntimeChannelDirection, RuntimeTypeInfo,
    RuntimeTypeKind, TYPE_ANY, TYPE_BOOL, TYPE_ERROR, TYPE_INT, TYPE_INT64, TYPE_STRING,
};

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
fn compile_source_registers_runtime_type_inventory_for_named_and_generic_types() {
    let source = r#"
package main

type Config struct {
    Name string `json:"name"`
    Values []string `json:"values,omitempty"`
    Meta map[string]int
    Any interface{}
}

type Labels []string

type Box[T any] struct {
    Value T
}

func main() {
    var labels Labels
    var box Box[int]
    _, _ = labels, box
}
"#;

    let program = compile_source(source).expect("program should compile");
    let inventory =
        program_type_inventory(&program).expect("compiled program should register type inventory");

    let config = inventory_entry(&inventory, "Config");
    assert_eq!(config.kind, RuntimeTypeKind::Struct);
    assert_eq!(config.fields.len(), 4);
    assert_eq!(config.fields[0].tag.as_deref(), Some(r#"json:"name""#));
    assert_eq!(
        config.fields[1].typ,
        ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
        }
    );
    assert_eq!(
        config.fields[1].tag.as_deref(),
        Some(r#"json:"values,omitempty""#)
    );
    assert_eq!(
        config.fields[2].typ,
        ConcreteType::Map {
            key: Box::new(ConcreteType::TypeId(TYPE_STRING)),
            value: Box::new(ConcreteType::TypeId(TYPE_INT)),
        }
    );
    assert_eq!(config.fields[2].tag, None);
    assert_eq!(config.fields[3].typ, ConcreteType::TypeId(TYPE_ANY));

    let labels = inventory_entry(&inventory, "Labels");
    assert_eq!(labels.kind, RuntimeTypeKind::Slice);
    assert_eq!(
        labels.underlying,
        Some(Box::new(ConcreteType::Slice {
            element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
        }))
    );

    let boxed = inventory_entry(&inventory, "Box[int]");
    assert_eq!(boxed.kind, RuntimeTypeKind::Struct);
    assert_eq!(boxed.fields.len(), 1);
    assert_eq!(boxed.fields[0].typ, ConcreteType::TypeId(TYPE_INT));
}

#[test]
fn compile_source_registers_runtime_type_inventory_for_alias_interface_and_recursive_generic_edges()
{
    let source = r#"
package main

type Labels []string

type Notifier interface {
    Notify() string
}

type Box[T any] struct {
    Value T
    Any interface{}
    Next *Box[T]
}

func main() {
    var labels Labels
    var notifier Notifier
    var box Box[Labels]
    _, _, _ = labels, notifier, box
}
"#;

    let program = compile_source(source).expect("program should compile");
    let inventory =
        program_type_inventory(&program).expect("compiled program should register type inventory");

    let labels_type_id = inventory_entry(&inventory, "Labels")
        .type_id
        .expect("named alias should have a type id");

    let notifier = inventory_entry(&inventory, "Notifier");
    assert_eq!(notifier.kind, RuntimeTypeKind::Interface);
    assert_eq!(notifier.package_path.as_deref(), Some("main"));

    let boxed = inventory_entry(&inventory, "Box[Labels]");
    let boxed_pointer_type_id = inventory_entry(&inventory, "*Box[Labels]")
        .type_id
        .expect("recursive generic pointer type should have a registered type id");
    assert_eq!(boxed.kind, RuntimeTypeKind::Struct);
    assert_eq!(boxed.package_path.as_deref(), Some("main"));
    assert_eq!(boxed.fields.len(), 3);
    assert_eq!(boxed.fields[0].typ, ConcreteType::TypeId(labels_type_id));
    assert_eq!(boxed.fields[1].typ, ConcreteType::TypeId(TYPE_ANY));
    assert_eq!(
        boxed.fields[2].typ,
        ConcreteType::TypeId(boxed_pointer_type_id)
    );
}

#[test]
fn compile_source_registers_runtime_type_inventory_for_function_and_channel_shapes() {
    let source = r#"
package main

type Handler func(string, int) (bool, error)
type Feed chan int
type Sink <-chan string

func main() {
    var handler Handler
    var feed Feed
    var sink Sink
    _, _, _ = handler, feed, sink
}
"#;

    let program = compile_source(source).expect("program should compile");
    let inventory =
        program_type_inventory(&program).expect("compiled program should register type inventory");

    let handler = inventory_entry(&inventory, "Handler");
    assert_eq!(handler.kind, RuntimeTypeKind::Function);
    assert_eq!(
        handler.underlying,
        Some(Box::new(ConcreteType::Function {
            params: vec![
                ConcreteType::TypeId(TYPE_STRING),
                ConcreteType::TypeId(TYPE_INT),
            ],
            results: vec![
                ConcreteType::TypeId(TYPE_BOOL),
                ConcreteType::TypeId(TYPE_ERROR),
            ],
        }))
    );

    let feed = inventory_entry(&inventory, "Feed");
    assert_eq!(feed.kind, RuntimeTypeKind::Channel);
    assert_eq!(
        feed.underlying,
        Some(Box::new(ConcreteType::Channel {
            direction: RuntimeChannelDirection::Bidirectional,
            element: Box::new(ConcreteType::TypeId(TYPE_INT)),
        }))
    );

    let sink = inventory_entry(&inventory, "Sink");
    assert_eq!(sink.kind, RuntimeTypeKind::Channel);
    assert_eq!(
        sink.underlying,
        Some(Box::new(ConcreteType::Channel {
            direction: RuntimeChannelDirection::ReceiveOnly,
            element: Box::new(ConcreteType::TypeId(TYPE_STRING)),
        }))
    );
}

#[test]
fn compile_source_registers_runtime_type_inventory_for_imported_interfaces() {
    let source = r#"
package main

import "net/http"

func main() {
    req, _ := http.NewRequest("POST", "https://example.com/upload", nil)
    var body = req.Body
    var client = http.DefaultClient
    _, _ = body, client
}
"#;

    let program = compile_source(source).expect("program should compile");
    let inventory =
        program_type_inventory(&program).expect("compiled program should register type inventory");

    let reader = inventory_entry(&inventory, "io.Reader");
    assert_eq!(reader.kind, RuntimeTypeKind::Interface);
    assert_eq!(reader.package_path.as_deref(), Some("io"));
}

#[test]
fn compile_source_registers_runtime_type_inventory_for_int64_scalars() {
    let source = r#"
package main

type Stamp struct {
    Value int64
}

func main() {
    var stamp Stamp
    _ = stamp
}
"#;

    let program = compile_source(source).expect("program should compile");
    let inventory =
        program_type_inventory(&program).expect("compiled program should register type inventory");

    let int64_type = inventory_entry(&inventory, "int64");
    assert_eq!(int64_type.kind, RuntimeTypeKind::Int);
    assert_eq!(int64_type.type_id, Some(TYPE_INT64));

    let stamp = inventory_entry(&inventory, "Stamp");
    assert_eq!(stamp.kind, RuntimeTypeKind::Struct);
    assert_eq!(stamp.fields.len(), 1);
    assert_eq!(stamp.fields[0].typ, ConcreteType::TypeId(TYPE_INT64));
}
