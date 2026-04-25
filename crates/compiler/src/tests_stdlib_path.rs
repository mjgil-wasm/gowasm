use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_path_helpers() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    fmt.Println(
        path.Base("/a/b/"),
        path.Clean("a/../../b"),
        path.Dir("/a/b/"),
        path.Ext("archive.tar.gz"),
        path.IsAbs("/a/b/"),
        path.IsAbs("a/../../b")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b ../b /a/b .gz true false\n");
}

#[test]
fn compiles_and_runs_path_edge_cases() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    fmt.Println(
        path.Base(""),
        path.Base("//"),
        path.Clean(""),
        path.Dir("a"),
        path.Ext(".bashrc")
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), ". / . . .bashrc\n");
}

#[test]
fn compiles_and_runs_path_split_pairs() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    dir, file := path.Split("a/b")
    parentDir, parentFile := path.Split("../a")
    fmt.Println(dir, file, parentDir, parentFile)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "a/ b ../ a\n");
}

#[test]
fn compiles_and_runs_path_join() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    fmt.Println(
        "[" + path.Join() + "]",
        path.Join("", "a", "b"),
        path.Join("/a", "../b", "c"),
        path.Join("/a/b", "..", "c"),
        "[" + path.Join("", "", "") + "]"
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[] a/b /b/c /a/c []\n");
}

#[test]
fn compiles_and_runs_path_match() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    matched, err := path.Match("a?c", "abc")
    slashMiss, slashErr := path.Match("*", "a/b")
    bad, badErr := path.Match("[", "go")
    fmt.Println(matched, err, slashMiss, slashErr, bad, badErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true <nil> false <nil> false syntax error in pattern\n"
    );
}

#[test]
fn compiles_and_runs_path_match_edge_cases() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    escaped, escapedErr := path.Match("a\\*b", "a*b")
    negated, negatedErr := path.Match("ab[^e-g]", "abc")
    unicode, unicodeErr := path.Match("a?b", "a☺b")
    rangeMatch, rangeErr := path.Match("[a-ζ]*", "α")
    badRange, badRangeErr := path.Match("[-]", "-")
    fmt.Println(
        escaped, escapedErr,
        negated, negatedErr,
        unicode, unicodeErr,
        rangeMatch, rangeErr,
        badRange, badRangeErr,
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true <nil> true <nil> true <nil> true <nil> false syntax error in pattern\n"
    );
}

#[test]
fn compiles_and_runs_path_match_star_scans() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    prefix, prefixErr := path.Match("a*/b", "abc/b")
    prefixSlashMiss, prefixSlashMissErr := path.Match("a*/b", "a/c/b")
    long, longErr := path.Match("a*b*c*d*e*/f", "axbxcxdxe/f")
    longSlashMiss, longSlashMissErr := path.Match("a*b*c*d*e*/f", "axbxcxdxe/xxx/f")
    chunk, chunkErr := path.Match("a*b?c*x", "abxbbxdbxebxczzx")
    chunkMiss, chunkMissErr := path.Match("a*b?c*x", "abxbbxdbxebxczzy")
    fmt.Println(
        prefix, prefixErr,
        prefixSlashMiss, prefixSlashMissErr,
        long, longErr,
        longSlashMiss, longSlashMissErr,
        chunk, chunkErr,
        chunkMiss, chunkMissErr,
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true <nil> false <nil> true <nil> false <nil> true <nil> false <nil>\n"
    );
}

#[test]
fn compiles_and_runs_path_match_bad_pattern_cases() {
    let source = r#"
package main
import "fmt"
import "path"

func main() {
    escapedClose, escapedCloseErr := path.Match("[\\]a]", "]")
    escapedDash, escapedDashErr := path.Match("[x\\-]", "-")
    escapedLeadingDash, escapedLeadingDashErr := path.Match("[\\-x]", "x")
    badEscape, badEscapeErr := path.Match("\\", "a")
    badRange, badRangeErr := path.Match("[a-b-c]", "a")
    badNegated, badNegatedErr := path.Match("[^bc", "a")
    fmt.Println(
        escapedClose, escapedCloseErr,
        escapedDash, escapedDashErr,
        escapedLeadingDash, escapedLeadingDashErr,
        badEscape, badEscapeErr,
        badRange, badRangeErr,
        badNegated, badNegatedErr,
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true <nil> true <nil> true <nil> false syntax error in pattern false syntax error in pattern false syntax error in pattern\n"
    );
}
