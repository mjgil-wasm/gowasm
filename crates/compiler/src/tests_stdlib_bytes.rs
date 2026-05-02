use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_bytes_contains_and_equal() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{104, 101, 108, 108, 111}
    sub := []byte{101, 108}
    other := []byte{104, 101, 108, 108, 111}
    diff := []byte{119, 111}
    fmt.Println(
        bytes.Contains(data, sub),
        bytes.Contains(data, diff),
        bytes.Equal(data, other),
        bytes.Equal(data, diff),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true false\n");
}

#[test]
fn compiles_and_runs_bytes_prefix_suffix() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{1, 2, 3, 4, 5}
    pre := []byte{1, 2}
    suf := []byte{4, 5}
    bad := []byte{2, 3}
    fmt.Println(
        bytes.HasPrefix(data, pre),
        bytes.HasPrefix(data, bad),
        bytes.HasSuffix(data, suf),
        bytes.HasSuffix(data, bad),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true false\n");
}

#[test]
fn compiles_and_runs_bytes_index_and_count() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{1, 2, 3, 2, 1}
    needle := []byte{2, 3}
    missing := []byte{9, 9}
    fmt.Println(
        bytes.Index(data, needle),
        bytes.Index(data, missing),
        bytes.LastIndex(data, []byte{1}),
        bytes.IndexByte(data, 3),
        bytes.LastIndexByte(data, 2),
        bytes.Count(data, []byte{2}),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 -1 4 2 3 2\n");
}

#[test]
fn compiles_and_runs_bytes_repeat_and_join() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{65, 66}
    repeated := bytes.Repeat(data, 3)
    fmt.Println(repeated)
    sep := []byte{44}
    parts := [][]byte{[]byte{72, 73}, []byte{66, 89}}
    joined := bytes.Join(parts, sep)
    fmt.Println(joined)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[65 66 65 66 65 66]\n[72 73 44 66 89]\n");
}

#[test]
fn compiles_and_runs_bytes_replace_and_trim() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{65, 66, 67, 66, 68}
    old := []byte{66}
    new := []byte{88, 88}
    replaced := bytes.ReplaceAll(data, old, new)
    fmt.Println(replaced)

    spaced := []byte{32, 72, 73, 32}
    trimmed := bytes.TrimSpace(spaced)
    fmt.Println(trimmed)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[65 88 88 67 88 88 68]\n[72 73]\n");
}

#[test]
fn compiles_and_runs_bytes_case_and_compare() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{104, 101, 108, 108, 111}
    upper := bytes.ToUpper(data)
    fmt.Println(upper)
    lower := bytes.ToLower(upper)
    fmt.Println(lower)
    fmt.Println(
        bytes.Compare([]byte{1}, []byte{2}),
        bytes.Compare([]byte{2}, []byte{2}),
        bytes.Compare([]byte{3}, []byte{2}),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[72 69 76 76 79]\n[104 101 108 108 111]\n-1 0 1\n"
    );
}

#[test]
fn compiles_and_runs_bytes_contains_any_and_rune() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    data := []byte{104, 101, 108, 108, 111}
    fmt.Println(
        bytes.ContainsAny(data, "aeiou"),
        bytes.ContainsAny(data, "xyz"),
        bytes.ContainsRune(data, 104),
        bytes.ContainsRune(data, 122),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true false\n");
}

#[test]
fn compiles_and_runs_bytes_split_variants_and_cut_helpers() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    fmt.Println(bytes.SplitN([]byte("go,wasm,zig"), []byte(","), 2))
    fmt.Println(bytes.SplitAfterN([]byte("go,wasm,zig"), []byte(","), 2))
    fmt.Println(bytes.SplitAfter([]byte("ab"), []byte("")))

    trimmedPrefix, okPrefix := bytes.CutPrefix([]byte("prefix-body"), []byte("prefix-"))
    trimmedSuffix, okSuffix := bytes.CutSuffix([]byte("body-suffix"), []byte("-suffix"))
    before, after, okCut := bytes.Cut([]byte("left=right"), []byte("="))
    fmt.Println(trimmedPrefix, okPrefix)
    fmt.Println(trimmedSuffix, okSuffix)
    fmt.Println(before, after, okCut)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[[103 111] [119 97 115 109 44 122 105 103]]\n[[103 111 44] [119 97 115 109 44 122 105 103]]\n[[97] [98]]\n[98 111 100 121] true\n[98 111 100 121] true\n[108 101 102 116] [114 105 103 104 116] true\n"
    );
}

#[test]
fn compiles_and_runs_bytes_search_trim_clone_title_and_fold() {
    let source = r#"
package main
import "bytes"
import "fmt"

func main() {
    fmt.Println(bytes.Fields([]byte("  go wasm\tzig  ")))
    fmt.Println(bytes.Trim([]byte("¡¡go!!"), "!¡"))
    fmt.Println(bytes.TrimLeft([]byte("¡¡go!!"), "!¡"))
    fmt.Println(bytes.TrimRight([]byte("¡¡go!!"), "!¡"))
    fmt.Println(bytes.IndexAny([]byte("héllo"), "xyzé"))
    fmt.Println(bytes.LastIndexAny([]byte("héllö"), "öé"))
    fmt.Println(bytes.IndexRune([]byte("héllö"), 'ö'))
    fmt.Println(bytes.Clone([]byte("copy")))
    fmt.Println(bytes.ToTitle([]byte("héllö")))
    fmt.Println(bytes.EqualFold([]byte("Straße"), []byte("straße")))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[[103 111] [119 97 115 109] [122 105 103]]\n[103 111]\n[103 111 33 33]\n[194 161 194 161 103 111]\n1\n5\n5\n[99 111 112 121]\n[72 195 137 76 76 195 150]\ntrue\n"
    );
}
