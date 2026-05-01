use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_zero_value_empty_interface_vars() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func main() {
    var value Any
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "<nil>\n");
}

#[test]
fn compiles_and_runs_nil_interface_literals() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func main() {
    var value Any = nil
    fmt.Println(value == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn compiles_and_runs_package_level_empty_interface_vars() {
    let source = r#"
package main
import "fmt"

type Any interface {}
var value Any

func main() {
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "<nil>\n");
}

#[test]
fn compiles_and_runs_predeclared_error_nil_checks() {
    let source = r#"
package main
import "fmt"
import "strconv"

func main() {
    var err error
    var ignored int
    fmt.Println(err == nil)
    ignored, err = strconv.Atoi("bad")
    fmt.Println(err != nil, err, err.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue strconv.Atoi: parsing \"bad\": invalid syntax strconv.Atoi: parsing \"bad\": invalid syntax\n"
    );
}

#[test]
fn compiles_and_runs_typed_nil_pointer_inside_interface_as_non_nil() {
    let source = r#"
package main
import "fmt"

type Describer interface {
    Describe() string
}

type box struct{}

func (*box) Describe() string {
    return "box"
}

func main() {
    var pointer *box
    var value Describer = pointer
    fmt.Println(pointer == nil)
    fmt.Println(value == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n");
}

#[test]
fn compiles_and_runs_typed_nil_pointer_inside_empty_interface_as_non_nil() {
    let source = r#"
package main
import "fmt"

func main() {
    var pointer *int
    var value interface{} = pointer
    fmt.Println(pointer == nil)
    fmt.Println(value == nil)
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n<nil>\n");
}

#[test]
fn compiles_with_non_empty_interface_declarations() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    area() int
}

func main() {
    fmt.Println(1)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn compiles_and_runs_single_value_type_assertions() {
    let source = r#"
package main
import "fmt"

type Any interface {}

func main() {
    var value Any
    value = 7
    number := value.(int)
    fmt.Println(number)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn type_assertions_fail_at_runtime_for_wrong_types() {
    let source = r#"
package main

type Any interface {}

func main() {
    var value Any
    value = 7
    value = value.(string)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("type assertion should fail");
    assert!(error
        .to_string()
        .contains("type assertion to `string` failed"));
}

#[test]
fn compiles_and_runs_interface_target_type_assertions() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Shape interface {
    area() int
}
type Point struct { x int }

func (point Point) area() int {
    return point.x
}

func main() {
    var value Any
    value = Point{x: 7}
    shape := value.(Shape)
    fmt.Println(shape.area())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn compiles_and_runs_comma_ok_interface_target_assertions() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Shape interface {
    area() int
}
type Point struct { x int }

func (point Point) area() int {
    return point.x
}

func main() {
    var value Any
    value = Point{x: 9}
    shape, ok := value.(Shape)
    value = 3
    missing, missingOk := value.(Shape)
    fmt.Println(shape.area(), ok, missing, missingOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9 true <nil> false\n");
}

#[test]
fn compiles_and_runs_interface_typed_method_calls() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    area() int
}
type Point struct { x int }

var global Shape

func (point Point) area() int {
    return point.x
}

func render(shape Shape) int {
    return shape.area()
}

func main() {
    global = Point{x: 5}
    var local Shape
    local = Point{x: 6}
    fmt.Println(render(global), local.area(), global.area())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "5 6 5\n");
}

#[test]
fn compiles_and_runs_pointer_receiver_interface_calls() {
    let source = r#"
package main
import "fmt"

type Incer interface {
    inc() int
}
type Counter struct { value int }

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func main() {
    counter := Counter{value: 1}
    var incer Incer
    incer = &counter
    fmt.Println(incer.inc(), counter.value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 2\n");
}

#[test]
fn rejects_methods_not_in_interface_method_sets() {
    let source = r#"
package main

type Shape interface {
    area() int
}
type Point struct {}

func (point Point) area() int {
    return 1
}

func (point Point) perimeter() int {
    return 2
}

func main() {
    var shape Shape
    shape = Point{}
    shape.perimeter()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("method `perimeter` is not part of interface `Shape`"));
}

#[test]
fn rejects_assigning_non_satisfying_values_to_interface_vars() {
    let source = r#"
package main

type Shape interface {
    area() int
}

func main() {
    var shape Shape
    shape = 7
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("constant of type `int` is not assignable to `Shape`"));
}

#[test]
fn rejects_non_satisfying_package_interface_initializers() {
    let source = r#"
package main

type Shape interface {
    area() int
}

var shape Shape = 7

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("constant of type `int` is not assignable to `Shape`"));
}

#[test]
fn allows_assigning_satisfying_values_to_interface_vars() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    area() int
}
type Point struct { x int }

func (point Point) area() int {
    return point.x
}

func main() {
    var shape Shape
    shape = Point{x: 8}
    fmt.Println(shape.area())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8\n");
}

#[test]
fn allows_assigning_pointer_values_to_value_receiver_interfaces() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    area() int
}
type Point struct { x int }

func (point Point) area() int {
    return point.x
}

func main() {
    point := Point{x: 8}
    var shape Shape
    shape = &point
    fmt.Println(shape.area())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "8\n");
}

#[test]
fn rejects_assigning_values_to_pointer_receiver_interfaces() {
    let source = r#"
package main

type Incer interface {
    inc() int
}
type Counter struct { value int }

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func main() {
    var incer Incer
    counter := Counter{value: 1}
    incer = counter
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("type `Counter` does not satisfy interface `Incer`"));
}

#[test]
fn compiles_and_runs_interface_valued_returns() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    area() int
}
type Point struct { x int }

func (point Point) area() int {
    return point.x
}

func makeShape() Shape {
    return Point{x: 4}
}

func main() {
    value := makeShape()
    fmt.Println(value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "{4}\n");
}

#[test]
fn compiles_and_runs_interface_typed_short_decls_from_function_calls() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    area() int
}
type Point struct { x int }

func (point Point) area() int {
    return point.x
}

func makeShape() Shape {
    return Point{x: 11}
}

func main() {
    shape := makeShape()
    fmt.Println(shape.area(), makeShape().area())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "11 11\n");
}

#[test]
fn rejects_methods_not_in_interface_sets_for_interface_returning_calls() {
    let source = r#"
package main

type Shape interface {
    area() int
}
type Point struct {}

func (point Point) area() int {
    return 1
}

func (point Point) perimeter() int {
    return 2
}

func makeShape() Shape {
    return Point{}
}

func main() {
    makeShape().perimeter()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("method `perimeter` is not part of interface `Shape`"));
}

#[test]
fn rejects_non_satisfying_interface_assignments_from_function_calls() {
    let source = r#"
package main

type Shape interface {
    area() int
}

func makeNumber() int {
    return 7
}

func main() {
    var shape Shape
    shape = makeNumber()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    let detail = error.to_string();
    assert!(detail.contains("type `int`"));
    assert!(detail.contains("does not satisfy interface"));
    assert!(detail.contains("Shape"));
}

#[test]
fn rejects_non_satisfying_interface_valued_returns() {
    let source = r#"
package main

type Shape interface {
    area() int
}

func makeShape() Shape {
    return 7
}

func main() {}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("constant of type `int` is not assignable to `Shape`"));
}

#[test]
fn compiles_and_runs_comma_ok_type_assertions() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Point struct { x int }

func main() {
    var value Any
    value = Point{x: 7}
    point, ok := value.(Point)
    value = 3
    missing, missingOk := value.(Point)
    fmt.Println(point, ok, missing, missingOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "{7} true {0} false\n");
}

#[test]
fn compiles_and_runs_pointer_interface_target_assertions() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Incer interface {
    inc() int
}
type Counter struct { value int }

func (counter *Counter) inc() int {
    counter.value = counter.value + 1
    return counter.value
}

func main() {
    counter := Counter{value: 1}
    var value Any
    value = &counter
    incer := value.(Incer)
    fmt.Println(incer.inc(), counter.value)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "2 2\n");
}

#[test]
fn compiles_and_runs_assignment_form_comma_ok_type_assertions() {
    let source = r#"
package main
import "fmt"

type Any interface {}
type Point struct { x int }

func main() {
    var value Any
    var point Point
    var ok bool
    var missing Point
    var missingOk bool
    value = Point{x: 9}
    point, ok = value.(Point)
    value = true
    missing, missingOk = value.(Point)
    fmt.Println(point, ok, missing, missingOk)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "{9} true {0} false\n");
}
