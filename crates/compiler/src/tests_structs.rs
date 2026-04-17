use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_struct_field_reads() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    point := Point{x: 1, y: 2}
    fmt.Println(point.x, point.y, point)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2 {1 2}\n");
}

#[test]
fn compiles_zero_value_struct_locals() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    var point Point
    fmt.Println(point)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "{0 0}\n");
}

#[test]
fn fills_missing_struct_literal_fields_with_zero_values() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    point := Point{x: 7}
    fmt.Println(point)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "{7 0}\n");
}

#[test]
fn compiles_and_runs_struct_field_assignment() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    point := Point{x: 1, y: 2}
    point.x = 9
    fmt.Println(point.x, point)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "9 {9 2}\n");
}

#[test]
fn compiles_and_runs_value_receiver_methods() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func (point Point) sum() int {
    return point.x + point.y
}

func main() {
    point := Point{x: 2, y: 5}
    fmt.Println(point.sum(), Point{x: 1, y: 4}.sum())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 5\n");
}

#[test]
fn compiles_and_runs_direct_concrete_method_calls_without_runtime_dispatch() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func (point Point) sum() int {
    return point.x + point.y
}

func total(point Point) int {
    return point.sum()
}

func main() {
    point := Point{x: 3, y: 4}
    fmt.Println(total(point), point.sum())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7 7\n");
}

#[test]
fn rejects_method_calls_with_the_wrong_arity() {
    let source = r#"
package main

type Point struct {
    x int
}

func (point Point) move(dx int) {}

func main() {
    point := Point{x: 1}
    point.move()
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("method `Point.move` expects 1 argument(s), found 0"));
}

#[test]
fn rejects_method_calls_with_the_wrong_argument_types() {
    let source = r#"
package main

type Point struct {
    x int
}

func (point Point) move(dx int) {}

func main() {
    point := Point{x: 1}
    point.move("bad")
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("argument 1 to method `Point.move` has type `string`, expected `int`"));
}

#[test]
fn compiles_and_runs_struct_embedding_field_promotion() {
    let source = r#"
package main
import "fmt"

type Animal struct {
    Name string
    Sound string
}

type Dog struct {
    Animal
    Breed string
}

func main() {
    d := Dog{Animal: Animal{Name: "Rex", Sound: "woof"}, Breed: "Lab"}
    fmt.Println(d.Name)
    fmt.Println(d.Sound)
    fmt.Println(d.Breed)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Rex\nwoof\nLab\n");
}

#[test]
fn compiles_and_runs_struct_embedding_method_promotion() {
    let source = r#"
package main
import "fmt"

type Base struct {
    Value int
}

func (b Base) Describe() string {
    return "base"
}

type Derived struct {
    Base
    Extra string
}

func main() {
    d := Derived{Base: Base{Value: 42}, Extra: "hello"}
    fmt.Println(d.Describe())
    fmt.Println(d.Value)
    fmt.Println(d.Extra)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "base\n42\nhello\n");
}

#[test]
fn compiles_and_runs_struct_embedding_interface_satisfaction() {
    let source = r#"
package main
import "fmt"

type Namer interface {
    GetName() string
}

type Person struct {
    Name string
}

func (p Person) GetName() string {
    return p.Name
}

type Employee struct {
    Person
    Role string
}

func greet(n Namer) {
    fmt.Println("Hello " + n.GetName())
}

func main() {
    e := Employee{Person: Person{Name: "Alice"}, Role: "eng"}
    fmt.Println(e.GetName())
    fmt.Println(e.Name)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Alice\nAlice\n");
}

#[test]
fn compiles_and_runs_struct_embedding_interface_dispatch() {
    let source = r#"
package main
import "fmt"

type Namer interface {
    GetName() string
}

type Person struct {
    Name string
}

func (p Person) GetName() string {
    return p.Name
}

type Employee struct {
    Person
    Role string
}

func greet(n Namer) {
    fmt.Println("Hello " + n.GetName())
}

func main() {
    e := Employee{Person: Person{Name: "Bob"}, Role: "dev"}
    greet(e)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Hello Bob\n");
}

#[test]
fn compiles_and_runs_struct_embedding_multiple_interface_methods() {
    let source = r#"
package main
import "fmt"

type Shape interface {
    Area() int
    Name() string
}

type BaseShape struct {
    ShapeName string
}

func (b BaseShape) Name() string {
    return b.ShapeName
}

type Rect struct {
    BaseShape
    W int
    H int
}

func (r Rect) Area() int {
    return r.W * r.H
}

func describe(s Shape) {
    fmt.Println(s.Name(), s.Area())
}

func main() {
    r := Rect{BaseShape: BaseShape{ShapeName: "rect"}, W: 3, H: 4}
    describe(r)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "rect 12\n");
}

#[test]
fn compiles_and_runs_struct_embedding_field_write() {
    let source = r#"
package main
import "fmt"

type Animal struct {
    Name string
}

type Dog struct {
    Animal
    Breed string
}

func main() {
    d := Dog{Animal: Animal{Name: "Rex"}, Breed: "Lab"}
    fmt.Println(d.Name)
    d.Name = "Max"
    fmt.Println(d.Name)
    fmt.Println(d.Breed)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Rex\nMax\nLab\n");
}

#[test]
fn compiles_and_runs_positional_struct_literals() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    p := Point{3, 7}
    fmt.Println(p.x, p.y, p)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 7 {3 7}\n");
}

#[test]
fn compiles_and_runs_positional_struct_literal_with_strings() {
    let source = r#"
package main
import "fmt"

type Person struct {
    Name string
    Age int
}

func main() {
    p := Person{"Alice", 30}
    fmt.Println(p.Name, p.Age)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "Alice 30\n");
}

#[test]
fn rejects_positional_struct_literals_with_wrong_field_count() {
    let source = r#"
package main

type Point struct {
    x int
    y int
}

func main() {
    _ = Point{1}
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(error
        .to_string()
        .contains("struct `Point` has 2 fields, but literal provides 1"));
}

#[test]
fn compiles_and_runs_struct_equality() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    a := Point{x: 1, y: 2}
    b := Point{x: 1, y: 2}
    c := Point{x: 3, y: 4}
    fmt.Println(a == b)
    fmt.Println(a == c)
    fmt.Println(a != c)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\ntrue\n");
}

#[test]
fn compiles_and_runs_struct_equality_in_condition() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    a := Point{x: 1, y: 2}
    b := Point{x: 1, y: 2}
    if a == b {
        fmt.Println("equal")
    } else {
        fmt.Println("not equal")
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "equal\n");
}

#[test]
fn compiles_and_runs_struct_equality_with_strings() {
    let source = r#"
package main
import "fmt"

type Person struct {
    Name string
    Age int
}

func main() {
    a := Person{Name: "Alice", Age: 30}
    b := Person{Name: "Alice", Age: 30}
    c := Person{Name: "Bob", Age: 30}
    fmt.Println(a == b)
    fmt.Println(a == c)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\nfalse\n");
}

#[test]
fn compiles_and_runs_address_of_struct_literal() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func main() {
    p := &Point{x: 3, y: 7}
    fmt.Println(p.x, p.y)
    p.x = 10
    fmt.Println(p.x, p.y)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 7\n10 7\n");
}

#[test]
fn compiles_and_runs_method_value() {
    let source = r#"
package main
import "fmt"

type Point struct {
    x int
    y int
}

func (p Point) Sum() int {
    return p.x + p.y
}

func main() {
    p := Point{x: 3, y: 7}
    f := p.Sum
    fmt.Println(f())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "10\n");
}

#[test]
fn compiles_and_runs_method_value_with_args() {
    let source = r#"
package main
import "fmt"

type Calculator struct {
    base int
}

func (c Calculator) Add(x int) int {
    return c.base + x
}

func main() {
    calc := Calculator{base: 10}
    add := calc.Add
    fmt.Println(add(5))
    fmt.Println(add(20))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "15\n30\n");
}

#[test]
fn compiles_and_runs_method_value_passed_as_function() {
    let source = r#"
package main
import "fmt"

type Greeter struct {
    Name string
}

func (g Greeter) Greet() string {
    return "hello " + g.Name
}

func apply(f func() string) string {
    return f()
}

func main() {
    g := Greeter{Name: "world"}
    fmt.Println(apply(g.Greet))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello world\n");
}

#[test]
fn compiles_and_runs_struct_with_function_field() {
    let source = r#"
package main
import "fmt"

type Handler struct {
    Name string
    Run  func(int) int
}

func main() {
    h := Handler{
        Name: "double",
        Run: func(x int) int { return x * 2 },
    }
    fmt.Println(h.Name)
    fmt.Println(h.Run(21))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "double\n42\n");
}

#[test]
fn compiles_and_runs_struct_function_field_reassignment() {
    let source = r#"
package main
import "fmt"

type Op struct {
    Apply func(int, int) int
}

func main() {
    op := Op{Apply: func(a int, b int) int { return a + b }}
    fmt.Println(op.Apply(3, 4))
    op.Apply = func(a int, b int) int { return a * b }
    fmt.Println(op.Apply(3, 4))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n12\n");
}

#[test]
fn compiles_and_runs_nil_function_field_zero_value() {
    let source = r#"
package main
import "fmt"

type Config struct {
    Name string
    Hook func()
}

func main() {
    c := Config{Name: "test"}
    fmt.Println(c.Name)
    fmt.Println(c.Hook == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "test\ntrue\n");
}
