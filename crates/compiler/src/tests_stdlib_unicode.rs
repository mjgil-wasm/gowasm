use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_unicode_predicates() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsDigit(55),
        unicode.IsDigit(65),
        unicode.IsLetter(955),
        unicode.IsSpace(10),
        unicode.IsUpper(71),
        unicode.IsLower(103),
        unicode.IsLetter(-1),
        unicode.IsSpace(1114112),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true true true true false false\n");
}

#[test]
fn compiles_and_runs_unicode_case_mappings() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.ToUpper(97),
        unicode.ToUpper(955),
        unicode.ToLower(65),
        unicode.ToLower(923),
        unicode.ToUpper(-1),
        unicode.ToLower(1114112),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65 923 97 955 -1 1114112\n");
}

#[test]
fn compiles_and_runs_unicode_simple_case_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.ToUpper(223),
        unicode.ToLower(7838),
        unicode.ToUpper(8561),
        unicode.ToLower(8546),
        unicode.ToUpper(837),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7838 223 8545 8562 921\n");
}

#[test]
fn compiles_and_runs_unicode_title_mapping() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.ToTitle(454),
        unicode.ToTitle(452),
        unicode.ToTitle(453),
        unicode.ToTitle(97),
        unicode.ToTitle(65),
        unicode.ToTitle(-1),
        unicode.ToTitle(1114112),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "453 453 453 65 65 -1 1114112\n");
}

#[test]
fn compiles_and_runs_unicode_to_mapping() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.To(0, 97),
        unicode.To(1, 65),
        unicode.To(2, 454),
        unicode.To(-1, 65),
        unicode.To(0, -1),
        unicode.To(1, 1114112),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65 97 453 65533 -1 1114112\n");
}

#[test]
fn compiles_and_runs_unicode_to_simple_case_boundaries() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.To(0, 223),
        unicode.To(1, 7838),
        unicode.To(0, 8561),
        unicode.To(1, 8546),
        unicode.To(0, 837),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7838 223 8545 8562 921\n");
}

#[test]
fn compiles_and_runs_unicode_case_constants() {
    let source = r#"
package main
import "fmt"
import "unicode"

const title = unicode.TitleCase

func main() {
    lower := unicode.LowerCase
    fmt.Println(
        unicode.UpperCase,
        lower,
        title,
        unicode.To(unicode.UpperCase, 97),
        unicode.To(lower, 65),
        unicode.To(title, 454),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 1 2 65 97 453\n");
}

#[test]
fn compiles_and_runs_unicode_scalar_constants() {
    let source = r#"
package main
import "fmt"
import "unicode"

const replacement = unicode.ReplacementChar

func main() {
    fmt.Println(
        unicode.MaxASCII,
        unicode.MaxLatin1,
        unicode.MaxRune,
        replacement,
        unicode.MaxRune > unicode.MaxLatin1,
        replacement == 65533,
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "127 255 1114111 65533 true true\n");
}

#[test]
fn compiles_and_runs_unicode_version_constant() {
    let source = r#"
package main
import "fmt"
import "unicode"

const version = unicode.Version

func main() {
    fmt.Println(version, unicode.Version == "15.0.0")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "15.0.0 true\n");
}

#[test]
fn compiles_and_runs_unicode_simple_fold_cycle() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.SimpleFold(65),
        unicode.SimpleFold(97),
        unicode.SimpleFold(75),
        unicode.SimpleFold(107),
        unicode.SimpleFold(8490),
        unicode.SimpleFold(49),
        unicode.SimpleFold(-2),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "97 65 107 8490 75 49 -2\n");
}

#[test]
fn compiles_and_runs_unicode_non_ascii_simple_fold_cycles() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.SimpleFold(931),
        unicode.SimpleFold(962),
        unicode.SimpleFold(963),
        unicode.SimpleFold(452),
        unicode.SimpleFold(453),
        unicode.SimpleFold(454),
        unicode.SimpleFold(223),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "962 963 931 453 454 452 7838\n");
}

#[test]
fn compiles_and_runs_unicode_number_and_print_predicates() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsNumber(189),
        unicode.IsNumber(65),
        unicode.IsPrint(955),
        unicode.IsPrint(10),
        unicode.IsPrint(-1),
        unicode.IsPrint(1114112),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false true false false false\n");
}

#[test]
fn compiles_and_runs_unicode_control_predicate() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsControl(10),
        unicode.IsControl(127),
        unicode.IsControl(32),
        unicode.IsControl(955),
        unicode.IsControl(-1),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true false false false\n");
}

#[test]
fn compiles_and_runs_unicode_graphic_and_print_predicates() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsGraphic(32),
        unicode.IsPrint(32),
        unicode.IsGraphic(12288),
        unicode.IsPrint(12288),
        unicode.IsGraphic(769),
        unicode.IsPrint(769),
        unicode.IsGraphic(8205),
        unicode.IsPrint(-1),
        unicode.IsControl(8205),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true true false true true false false false\n"
    );
}

#[test]
fn compiles_and_runs_unicode_punctuation_predicate() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsPunct(33),
        unicode.IsPunct(8212),
        unicode.IsPunct(95),
        unicode.IsPunct(36),
        unicode.IsPunct(65),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false false\n");
}

#[test]
fn compiles_and_runs_unicode_symbol_predicate() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsSymbol(36),
        unicode.IsSymbol(43),
        unicode.IsSymbol(169),
        unicode.IsSymbol(65),
        unicode.IsSymbol(95),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false false\n");
}

#[test]
fn compiles_and_runs_unicode_mark_predicate() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsMark(769),
        unicode.IsMark(2307),
        unicode.IsMark(8413),
        unicode.IsMark(65),
        unicode.IsMark(36),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true false false\n");
}

#[test]
fn compiles_and_runs_unicode_title_predicate() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsTitle(453),
        unicode.IsTitle(452),
        unicode.IsTitle(454),
        unicode.IsTitle(65),
        unicode.IsTitle(97),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false false false false\n");
}

#[test]
fn compiles_and_runs_unicode_non_ascii_predicates() {
    let source = r#"
package main
import "fmt"
import "unicode"

func main() {
    fmt.Println(
        unicode.IsDigit(1637),
        unicode.IsDigit(8544),
        unicode.IsNumber(8544),
        unicode.IsSpace(12288),
        unicode.IsUpper(1046),
        unicode.IsLower(1078),
        unicode.IsControl(133),
        unicode.IsGraphic(133),
        unicode.IsLetter(1046),
        unicode.IsPrint(12288),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true false true true true true true false true false\n"
    );
}
