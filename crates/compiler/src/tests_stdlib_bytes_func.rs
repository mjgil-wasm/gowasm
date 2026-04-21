use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn bytes_index_func_found() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("Hello, World!")
    idx := bytes.IndexFunc(data, unicode.IsSpace)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n");
}

#[test]
fn bytes_index_func_not_found() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("hello")
    idx := bytes.IndexFunc(data, unicode.IsUpper)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n");
}

#[test]
fn bytes_index_func_closure() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte("abc123")
    idx := bytes.IndexFunc(data, func(r int) bool {
        return r >= 48 && r <= 57
    })
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\n");
}

#[test]
fn bytes_last_index_func_found() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("Hello World")
    idx := bytes.LastIndexFunc(data, unicode.IsUpper)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "6\n");
}

#[test]
fn bytes_trim_func_spaces() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("  hello  ")
    result := bytes.TrimFunc(data, unicode.IsSpace)
    fmt.Println(string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn bytes_trim_left_func() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("  hello  ")
    result := bytes.TrimLeftFunc(data, unicode.IsSpace)
    fmt.Printf("[%s]\n", string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello  ]\n");
}

#[test]
fn bytes_trim_right_func() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("  hello  ")
    result := bytes.TrimRightFunc(data, unicode.IsSpace)
    fmt.Printf("[%s]\n", string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[  hello]\n");
}

#[test]
fn bytes_fields_func_spaces() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("  foo  bar  baz  ")
    result := bytes.FieldsFunc(data, unicode.IsSpace)
    fmt.Println(len(result))
    fmt.Println(string(result[0]))
    fmt.Println(string(result[1]))
    fmt.Println(string(result[2]))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\nfoo\nbar\nbaz\n");
}

#[test]
fn bytes_fields_func_custom_separator() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte("a,b,,c")
    result := bytes.FieldsFunc(data, func(r int) bool {
        return r == 44
    })
    fmt.Println(len(result))
    fmt.Println(string(result[0]))
    fmt.Println(string(result[1]))
    fmt.Println(string(result[2]))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3\na\nb\nc\n");
}

#[test]
fn bytes_index_func_empty() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("")
    idx := bytes.IndexFunc(data, unicode.IsSpace)
    fmt.Println(idx)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1\n");
}

#[test]
fn bytes_trim_func_digits() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("123abc456")
    result := bytes.TrimFunc(data, unicode.IsDigit)
    fmt.Println(string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "abc\n");
}

#[test]
fn bytes_map_to_uppercase() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("hello")
    result := bytes.Map(unicode.ToUpper, data)
    fmt.Println(string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "HELLO\n");
}

#[test]
fn bytes_map_with_closure() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte("hello")
    result := bytes.Map(func(r int) int {
        if r == 108 {
            return 76
        }
        return r
    }, data)
    fmt.Println(string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "heLLo\n");
}

#[test]
fn bytes_map_drop_characters() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte("h e l l o")
    result := bytes.Map(func(r int) int {
        if r == 32 {
            return -1
        }
        return r
    }, data)
    fmt.Println(string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn bytes_map_empty() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte("")
    result := bytes.Map(unicode.ToUpper, data)
    fmt.Println(string(result))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "\n");
}

#[test]
fn bytes_callbacks_preserve_invalid_utf8_byte_offsets() {
    let source = r#"
package main
import "bytes"
import "fmt"
import "unicode"

func main() {
    data := []byte{0xff, 'A', 0xc0, '(', 'b'}
    fmt.Println(bytes.IndexFunc(data, unicode.IsGraphic))
    fmt.Println(bytes.LastIndexFunc(data, unicode.IsGraphic))
    fmt.Println(bytes.TrimFunc(data, unicode.IsGraphic))
    fmt.Println(bytes.FieldsFunc([]byte{0xff, ' ', 'a', ' ', 0xc0}, unicode.IsSpace))
    mapped := bytes.Map(func(r int) int {
        if r == 'b' {
            return 'B'
        }
        return unicode.ToUpper(r)
    }, data)
    fmt.Println(mapped)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0\n4\n[]\n[[255] [97] [192]]\n[239 191 189 65 239 191 189 40 66]\n"
    );
}
