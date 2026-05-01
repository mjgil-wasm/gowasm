use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn os_directory_mutation_updates_os_and_io_fs_views() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "os"
import "strings"

func labels(entries []fs.DirEntry) string {
    var names []string
    for _, entry := range entries {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        names = append(names, name)
    }
    return strings.Join(names, ",")
}

func main() {
    dirfs := os.DirFS(".")

    fmt.Println(os.MkdirAll("empty/deep", fs.FileMode(493)) == nil)
    top, topErr := os.ReadDir(".")
    empty, emptyErr := os.ReadDir("empty")
    deepInfo, deepErr := os.Stat("empty/deep")
    fsTop, fsTopErr := fs.ReadDir(dirfs, ".")

    fmt.Println(labels(top), topErr == nil)
    fmt.Println(labels(empty), emptyErr == nil, deepInfo.IsDir(), deepErr == nil)
    fmt.Println(labels(fsTop), fsTopErr == nil)

    fmt.Println(os.RemoveAll("existing") == nil)
    after, afterErr := os.ReadDir(".")
    removed, removedErr := os.Stat("existing")
    fsAfter, fsAfterErr := fs.ReadDir(dirfs, ".")

    fmt.Println(labels(after), afterErr == nil)
    fmt.Println(removed == nil, removedErr != nil)
    fmt.Println(labels(fsAfter), fsAfterErr == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("keep.txt".into(), "keep".into());
    vm.workspace_files
        .insert("existing/file.txt".into(), "file".into());
    vm.workspace_files
        .insert("existing/nested/child.txt".into(), "child".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\nempty/,existing/,keep.txt true\ndeep/ true true true\nempty/,existing/,keep.txt true\ntrue\nempty/,keep.txt true\ntrue true\nempty/,keep.txt true\n"
    );
}

#[test]
fn os_directory_mutation_handles_missing_and_file_blocked_paths() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "os"
import "strings"

func labels(entries []fs.DirEntry) string {
    var names []string
    for _, entry := range entries {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        names = append(names, name)
    }
    return strings.Join(names, ",")
}

func main() {
    fmt.Println(os.RemoveAll("missing") == nil)
    err := os.MkdirAll("keep.txt/child", fs.FileMode(493))
    root, rootErr := os.ReadDir(".")
    fmt.Println(err != nil)
    fmt.Println(labels(root), rootErr == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("keep.txt".into(), "keep".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\ntrue\nkeep.txt true\n");
}

#[test]
fn os_write_file_updates_os_and_io_fs_views() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "os"

func main() {
    dirfs := os.DirFS(".")

    fmt.Println(os.WriteFile("assets/new.txt", []byte("alpha"), fs.FileMode(420)) == nil)
    fmt.Println(os.WriteFile("/assets/new.txt", []byte("beta"), fs.FileMode(384)) == nil)

    data, readErr := os.ReadFile("assets/new.txt")
    fsData, fsErr := fs.ReadFile(dirfs, "assets/new.txt")
    entries, entriesErr := os.ReadDir("assets")
    missingErr := os.WriteFile("missing/new.txt", []byte("x"), fs.FileMode(420))

    fmt.Println(string(data), readErr == nil, string(fsData), fsErr == nil)
    fmt.Println(entries[0].Name(), entries[1].Name(), entriesErr == nil)
    fmt.Println(missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/existing.txt".into(), "existing".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue\nbeta true beta true\nexisting.txt new.txt true\ntrue\n"
    );
}
