use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn os_invalid_paths_unwrap_to_err_invalid_and_missing_paths_unwrap_to_err_not_exist() {
    let source = r#"
package main
import "errors"
import "fmt"
import "io/fs"
import "os"

func main() {
    _, missingErr := os.ReadFile("missing")
    writeMissing := os.WriteFile("missing/file.txt", []byte("x"), fs.FileMode(420))
    _, readInvalid := os.ReadFile("")
    writeInvalid := os.WriteFile("", []byte("x"), fs.FileMode(420))
    _, dirInvalid := os.ReadDir("")
    _, statInvalid := os.Stat("")
    _, lstatInvalid := os.Lstat("")
    mkdirInvalid := os.MkdirAll("", fs.FileMode(493))
    removeInvalid := os.RemoveAll("")

    fmt.Println(errors.Is(missingErr, os.ErrNotExist), errors.Unwrap(missingErr) == os.ErrNotExist)
    fmt.Println(errors.Is(writeMissing, os.ErrNotExist), errors.Unwrap(writeMissing) == os.ErrNotExist)
    fmt.Println(errors.Is(readInvalid, os.ErrInvalid), errors.Unwrap(readInvalid) == os.ErrInvalid)
    fmt.Println(errors.Is(writeInvalid, os.ErrInvalid), errors.Unwrap(writeInvalid) == os.ErrInvalid)
    fmt.Println(errors.Is(dirInvalid, os.ErrInvalid), errors.Is(statInvalid, os.ErrInvalid), errors.Is(lstatInvalid, os.ErrInvalid))
    fmt.Println(errors.Is(mkdirInvalid, os.ErrInvalid), errors.Is(removeInvalid, os.ErrInvalid))
    fmt.Println(errors.Unwrap(mkdirInvalid) == os.ErrInvalid, errors.Unwrap(removeInvalid) == os.ErrInvalid)
    fmt.Println(writeMissing.Error())
    fmt.Println(readInvalid.Error())
    fmt.Println(writeInvalid.Error())
    fmt.Println(mkdirInvalid.Error())
    fmt.Println(removeInvalid.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true false\ntrue false\ntrue true\ntrue true\ntrue true true\ntrue true\ntrue true\nopen missing/file.txt: file does not exist\nopen : invalid path\nopen : invalid path\nmkdir : invalid path\nremoveall : invalid path\n"
    );
}
