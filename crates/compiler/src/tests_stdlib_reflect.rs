use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn reflect_type_surface_reports_kinds_names_and_shape_metadata() {
    let source = r#"
package main

import "fmt"
import "reflect"
import "time"

type Inner struct {
    Value int
}

type Labels []string
type Lookup map[string]int

type Config struct {
    Name string `json:"name"`
    hidden int `json:"hidden"`
    Inner `json:"inner"`
}

func main() {
    var any interface{}
    var err error
    var ptr *Config
    var labels Labels
    var lookup Lookup
    var array [2]int

    anyType := reflect.TypeOf(any)
    errType := reflect.TypeOf(err)
    ptrType := reflect.TypeOf(ptr)

    configType := reflect.TypeOf(Config{})
    labelsType := reflect.TypeOf(labels)
    lookupType := reflect.TypeOf(lookup)
    arrayType := reflect.TypeOf(array)
    durationType := reflect.TypeOf(time.Duration(5))

    fmt.Println(anyType == nil)
    fmt.Println(errType == nil)
    fmt.Println(ptrType == nil)
    fmt.Println(ptrType.Kind() == reflect.Ptr)
    fmt.Println(ptrType.Kind().String())
    fmt.Println(configType.Kind() == reflect.Struct)
    fmt.Println(configType.Name())
    fmt.Println(configType.PkgPath())
    fmt.Println(configType.NumField())
    field0 := configType.Field(0)
    field1 := configType.Field(1)
    field2 := configType.Field(2)
    fmt.Println(field0.Name, field0.PkgPath, field0.Type.String(), field0.Tag, field0.Anonymous)
    fmt.Println(field1.Name, field1.PkgPath, field1.Type.Kind().String(), field1.Tag, field1.Anonymous)
    fmt.Println(field2.Name, field2.PkgPath, field2.Type.Name(), field2.Tag, field2.Anonymous)
    fmt.Println(reflect.TypeOf(field0.Tag).String())
    fmt.Println(labelsType.Name())
    fmt.Println(labelsType.PkgPath())
    fmt.Println(labelsType.Kind() == reflect.Slice)
    fmt.Println(labelsType.Elem().String())
    fmt.Println(lookupType.Kind() == reflect.Map)
    fmt.Println(lookupType.Key().String())
    fmt.Println(lookupType.Elem().String())
    fmt.Println(arrayType.Len())
    fmt.Println(durationType.String())
    fmt.Println(durationType.Name())
    fmt.Println(durationType.PkgPath())
    fmt.Println(reflect.Struct.String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "true\n",
            "true\n",
            "false\n",
            "true\n",
            "ptr\n",
            "true\n",
            "Config\n",
            "main\n",
            "3\n",
            "Name  string json:\"name\" false\n",
            "hidden main int json:\"hidden\" false\n",
            "Inner  Inner json:\"inner\" true\n",
            "reflect.StructTag\n",
            "Labels\n",
            "main\n",
            "true\n",
            "string\n",
            "true\n",
            "string\n",
            "int\n",
            "2\n",
            "time.Duration\n",
            "Duration\n",
            "time\n",
            "struct\n",
        )
    );
}

#[test]
fn reflect_value_surface_reports_validity_shape_and_readers() {
    let source = r#"
package main

import "fmt"
import "reflect"

type Box struct {
    Name string
    Count int
    Ratio float64
    Flag bool
    hidden int
    Any interface{}
}

func main() {
    var any interface{}
    var ptr *Box

    invalid := reflect.ValueOf(any)
    ptrValue := reflect.ValueOf(ptr)
    boxValue := reflect.ValueOf(Box{Name: "go", Count: 42, Ratio: 1.5, Flag: true, hidden: 9, Any: "inner"})
    nilAny := reflect.ValueOf(Box{}).Field(5)
    sliceValue := reflect.ValueOf([]string{"x", "y"})
    mapValue := reflect.ValueOf(map[string]int{"only": 7})
    stringValue := reflect.ValueOf("hi")

    fmt.Println(invalid.IsValid())
    fmt.Println(invalid.Kind() == reflect.Invalid)

    fmt.Println(ptrValue.Kind() == reflect.Ptr)
    fmt.Println(ptrValue.IsNil())
    fmt.Println(ptrValue.Elem().IsValid())

    fmt.Println(boxValue.Kind() == reflect.Struct)
    fmt.Println(boxValue.Type().String())
    fmt.Println(boxValue.Field(0).String())
    fmt.Println(boxValue.Field(1).Int())
    fmt.Println(boxValue.Field(2).Float())
    fmt.Println(boxValue.Field(3).Bool())
    fmt.Println(boxValue.Field(5).Kind() == reflect.Interface)
    fmt.Println(boxValue.Field(5).Type().String())
    fmt.Println(boxValue.Field(5).IsNil())
    fmt.Println(boxValue.Field(5).Elem().String())
    fmt.Println(boxValue.Field(5).Elem().Interface())

    fmt.Println(nilAny.IsNil())
    fmt.Println(nilAny.Elem().IsValid())

    fmt.Println(sliceValue.Len())
    fmt.Println(sliceValue.Index(1).String())

    fmt.Println(mapValue.Len())
    fmt.Println(mapValue.MapIndex("only").Int())
    fmt.Println(mapValue.MapIndex("missing").IsValid())
    keys := mapValue.MapKeys()
    fmt.Println(len(keys))
    fmt.Println(keys[0].String())

    fmt.Println(stringValue.Len())
    fmt.Println(stringValue.Index(1).Int())
    fmt.Println(stringValue.Interface())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "false\n",
            "true\n",
            "true\n",
            "true\n",
            "false\n",
            "true\n",
            "main.Box\n",
            "go\n",
            "42\n",
            "1.5\n",
            "true\n",
            "true\n",
            "interface {}\n",
            "false\n",
            "inner\n",
            "inner\n",
            "true\n",
            "false\n",
            "2\n",
            "y\n",
            "1\n",
            "7\n",
            "false\n",
            "1\n",
            "only\n",
            "2\n",
            "105\n",
            "hi\n",
        )
    );
}

#[test]
fn reflect_value_surface_reports_struct_iteration_and_interface_capability() {
    let source = r#"
package main

import "fmt"
import "reflect"

type Node struct {
    Name string
    hidden int
    Next *Node
}

func printPanicked(fn func()) {
    defer func() {
        fmt.Println(recover() != nil)
    }()
    fn()
}

func main() {
    nodeValue := reflect.ValueOf(Node{Name: "go", hidden: 9})
    ptrValue := reflect.ValueOf(&Node{Name: "go"})

    fmt.Println(nodeValue.NumField())
    fmt.Println(nodeValue.Field(0).CanInterface())
    fmt.Println(nodeValue.Field(1).CanInterface())
    fmt.Println(ptrValue.Elem().NumField())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), concat!("3\n", "true\n", "false\n", "3\n",));
}

#[test]
fn reflect_struct_tag_helpers_expose_lookup_without_manual_parsing() {
    let source = r#"
package main

import "fmt"
import "reflect"

type Config struct {
    Name string `json:"name,omitempty" note:"say \"hi\""`
    Empty string `empty:""`
    Broken string `json`
}

func main() {
    configType := reflect.TypeOf(Config{})
    nameField := configType.Field(0)
    emptyField := configType.Field(1)
    brokenField := configType.Field(2)

    fmt.Println(nameField.Tag.Get("json"))
    fmt.Println(nameField.Tag.Get("note"))
    renamed, renamedOk := nameField.Tag.Lookup("json")
    fmt.Println(renamed, renamedOk)
    missing, missingOk := nameField.Tag.Lookup("missing")
    fmt.Println(missing == "", missingOk)
    empty, emptyOk := emptyField.Tag.Lookup("empty")
    fmt.Println(empty == "", emptyOk)
    fmt.Println(brokenField.Tag.Get("json") == "")
    broken, brokenOk := brokenField.Tag.Lookup("json")
    fmt.Println(broken == "", brokenOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "name,omitempty\n",
            "say \"hi\"\n",
            "name,omitempty true\n",
            "true false\n",
            "true true\n",
            "true\n",
            "true false\n",
        )
    );
}

#[test]
fn reflect_type_surface_reports_function_bits_and_comparability() {
    let source = r#"
package main

import "fmt"
import "reflect"

type Count int
type Labels [2]string

type Pair struct {
    Left int
    Right string
}

type Bucket struct {
    Items []int
}

func main() {
    var labels Labels
    countType := reflect.TypeOf(Count(3))
    floatType := reflect.TypeOf(1.5)
    pairType := reflect.TypeOf(Pair{})
    bucketType := reflect.TypeOf(Bucket{})
    labelsType := reflect.TypeOf(labels)
    mapType := reflect.TypeOf(map[string]int{})
    funcType := reflect.TypeOf(func(name string, count int) (bool, error) {
        return count > 0, nil
    })

    fmt.Println(countType.Bits())
    fmt.Println(floatType.Bits())
    fmt.Println(pairType.Comparable())
    fmt.Println(bucketType.Comparable())
    fmt.Println(labelsType.Comparable())
    fmt.Println(mapType.Comparable())
    fmt.Println(funcType.NumIn())
    fmt.Println(funcType.In(0).String(), funcType.In(1).String())
    fmt.Println(funcType.NumOut())
    fmt.Println(funcType.Out(0).String(), funcType.Out(1).String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "64\n",
            "64\n",
            "true\n",
            "false\n",
            "true\n",
            "false\n",
            "2\n",
            "string int\n",
            "2\n",
            "bool error\n",
        )
    );
}

#[test]
fn reflect_runtime_model_stays_stable_for_nil_alias_interface_and_generic_edges() {
    let source = r#"
package main

import "fmt"
import "reflect"

type Labels []string

type Notifier interface {
    Notify() string
}

type Event struct {
    Name string
}

func (e Event) Notify() string {
    return e.Name
}

type Box[T any] struct {
    Value T
    Any interface{}
    Next *Box[T]
}

func main() {
    var notifier Notifier
    var labels Labels
    labels = append(labels, "x", "y")
    zero := Box[Labels]{}
    box := Box[Labels]{
        Value: labels,
        Any: Event{Name: "go"},
    }

    fmt.Println(reflect.TypeOf(notifier) == nil)
    fmt.Println(reflect.ValueOf(notifier).IsValid())

    labelsType := reflect.TypeOf(labels)
    fmt.Println(labelsType.String())
    fmt.Println(labelsType.Kind() == reflect.Slice)
    fmt.Println(labelsType.Elem().String())

    boxType := reflect.TypeOf(box)
    fmt.Println(boxType.String())
    fmt.Println(boxType.Field(0).Type.String())
    fmt.Println(boxType.Field(1).Type.String())
    fmt.Println(boxType.Field(2).Type.String())

    zeroValue := reflect.ValueOf(zero)
    fmt.Println(zeroValue.Field(1).Kind() == reflect.Interface)
    fmt.Println(zeroValue.Field(1).IsNil())
    fmt.Println(zeroValue.Field(1).Elem().IsValid())
    fmt.Println(zeroValue.Field(2).IsNil())
    fmt.Println(zeroValue.Field(2).Type().String())

    boxValue := reflect.ValueOf(box)
    fmt.Println(boxValue.Field(0).Type().String())
    fmt.Println(boxValue.Field(1).Kind() == reflect.Interface)
    fmt.Println(boxValue.Field(1).Elem().Type().String())
    fmt.Println(boxValue.Field(1).Elem().Field(0).String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        concat!(
            "true\n",
            "false\n",
            "main.Labels\n",
            "true\n",
            "string\n",
            "main.Box[Labels]\n",
            "main.Labels\n",
            "interface {}\n",
            "*main.Box[Labels]\n",
            "true\n",
            "true\n",
            "false\n",
            "true\n",
            "*main.Box[Labels]\n",
            "main.Labels\n",
            "true\n",
            "main.Event\n",
            "go\n",
        )
    );
}

#[test]
fn reflect_invalid_value_interface_operation_surfaces_runtime_panic() {
    let source = r#"
package main

import "reflect"

type Box struct {
    hidden int
}

func main() {
    _ = reflect.ValueOf(Box{hidden: 9}).Field(0).Interface()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("reflect: cannot Interface unexported value"));
}

#[test]
fn reflect_invalid_type_signature_queries_surface_runtime_panics() {
    let source = r#"
package main

import "reflect"

type Pair struct {
    Left int
    Right string
}

func main() {
    _ = reflect.TypeOf(Pair{}).NumIn()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("reflect: NumIn of non-func type"));
}

#[test]
fn rejects_reflect_mutation_methods_outside_supported_boundary() {
    let cases = [
        (
            "SetInt",
            r#"
package main

import "reflect"

func main() {
    value := 1
    reflect.ValueOf(&value).Elem().SetInt(3)
}
"#,
            "method `SetInt` is not part of interface `reflect.Value` in the current subset",
        ),
        (
            "Addr",
            r#"
package main

import "reflect"

func main() {
    value := 1
    _ = reflect.ValueOf(value).Addr()
}
"#,
            "method `Addr` is not part of interface `reflect.Value` in the current subset",
        ),
        (
            "Call",
            r#"
package main

import "reflect"

func main() {
    fn := func(value int) int { return value + 1 }
    reflect.ValueOf(fn).Call(nil)
}
"#,
            "method `Call` is not part of interface `reflect.Value` in the current subset",
        ),
    ];

    for (method, source, expected) in cases {
        let error = compile_source(source).expect_err("program should fail to compile");
        assert!(
            error.to_string().contains(expected),
            "unexpected diagnostic for reflect.Value.{method}: {error}",
        );
    }
}

#[test]
fn rejects_reflect_construction_helpers_outside_supported_boundary() {
    let cases = [
        (
            "New",
            r#"
package main

import "reflect"

func main() {
    _ = reflect.New(reflect.TypeOf(1))
}
"#,
            "package selector `reflect.New` is not supported in the current subset",
        ),
        (
            "MakeSlice",
            r#"
package main

import "reflect"

func main() {
    _ = reflect.MakeSlice(reflect.TypeOf([]int{}), 2, 2)
}
"#,
            "package selector `reflect.MakeSlice` is not supported in the current subset",
        ),
        (
            "MakeMap",
            r#"
package main

import "reflect"

func main() {
    _ = reflect.MakeMap(reflect.TypeOf(map[string]int{}))
}
"#,
            "package selector `reflect.MakeMap` is not supported in the current subset",
        ),
    ];

    for (helper, source, expected) in cases {
        let error = compile_source(source).expect_err("program should fail to compile");
        assert!(
            error.to_string().contains(expected),
            "unexpected diagnostic for reflect.{helper}: {error}",
        );
    }
}
