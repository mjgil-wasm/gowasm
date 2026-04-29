use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn io_fs_file_info_size_reports_workspace_lengths() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Stat("assets/config.txt")
    dirInfo, dirErr := os.Stat("assets/nested")
    entries, readErr := os.ReadDir("assets")
    fmt.Println(fileErr == nil, dirErr == nil, readErr == nil)
    for _, entry := range entries {
        info, infoErr := entry.Info()
        fmt.Println(entry.Name(), infoErr == nil, info.Size())
    }
    fmt.Println(fileInfo.Size(), dirInfo.Size())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true true\nconfig.txt true 7\nnested true 0\n7 0\n"
    );
}

#[test]
fn io_fs_file_mode_reports_directory_bits() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    fileInfo, fileErr := os.Stat("assets/config.txt")
    dirInfo, dirErr := os.Stat("assets/nested")
    entries, readErr := os.ReadDir("assets")
    fmt.Println(fileErr == nil, dirErr == nil, readErr == nil, fs.ModeDir == dirInfo.Mode())
    fmt.Println(fileInfo.Mode() == fs.FileMode(0), dirInfo.Mode() == fs.ModeDir)
    for _, entry := range entries {
        info, infoErr := entry.Info()
        fmt.Println(entry.Name(), infoErr == nil, entry.Type() == info.Mode(), entry.Type() == fs.ModeDir)
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true true true\ntrue true\nconfig.txt true true false\nnested true true true\n"
    );
}

#[test]
fn io_fs_file_mode_is_dir_reports_directory_shape() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    fileInfo, _ := os.Stat("assets/config.txt")
    dirInfo, _ := os.Stat("assets/nested")
    entries, _ := os.ReadDir("assets")
    fmt.Println(fs.ModeDir.IsDir(), fs.FileMode(0).IsDir())
    fmt.Println(fileInfo.Mode().IsDir(), dirInfo.Mode().IsDir())
    for _, entry := range entries {
        fmt.Println(entry.Name(), entry.Type().IsDir())
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true false\nfalse true\nconfig.txt false\nnested true\n"
    );
}

#[test]
fn io_fs_file_mode_is_regular_reports_regular_shape() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    fileInfo, _ := os.Stat("assets/config.txt")
    dirInfo, _ := os.Stat("assets/nested")
    entries, _ := os.ReadDir("assets")
    fmt.Println(fs.FileMode(0).IsRegular(), fs.ModeDir.IsRegular())
    fmt.Println(fileInfo.Mode().IsRegular(), dirInfo.Mode().IsRegular())
    for _, entry := range entries {
        fmt.Println(entry.Name(), entry.Type().IsRegular())
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true false\ntrue false\nconfig.txt true\nnested false\n"
    );
}

#[test]
fn io_fs_file_mode_type_projects_type_bits() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    fileInfo, _ := os.Stat("assets/config.txt")
    dirInfo, _ := os.Stat("assets/nested")
    entries, _ := os.ReadDir("assets")
    fmt.Println(fs.ModeDir.Type() == fs.ModeDir, fs.FileMode(0).Type() == fs.FileMode(0))
    fmt.Println(fileInfo.Mode().Type() == fs.FileMode(0), dirInfo.Mode().Type() == fs.ModeDir)
    for _, entry := range entries {
        fmt.Println(entry.Name(), entry.Type().Type() == entry.Type())
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\ntrue true\nconfig.txt true\nnested true\n"
    );
}

#[test]
fn io_fs_walk_dir_walks_workspace_trees_and_root_errors() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"
import "time"

type rootInfo struct {}

func (rootInfo) Name() string { return "root.bin" }
func (rootInfo) Size() int { return 7 }
func (rootInfo) Mode() fs.FileMode { return 0 }
func (rootInfo) ModTime() time.Time { return time.Unix(5, 6) }
func (rootInfo) IsDir() bool { return false }
func (rootInfo) Sys() interface{} { return "root-sys" }

type customFS struct {}

func (customFS) Stat(name string) (fs.FileInfo, error) { return rootInfo{}, nil }
func (customFS) Open(name string) (fs.File, error) { return nil, fmt.Errorf("open called") }

func main() {
    err := fs.WalkDir(os.DirFS("assets"), ".", func(path string, d fs.DirEntry, err error) error {
        info, infoErr := d.Info()
        fmt.Println(path, d.Name(), d.IsDir(), infoErr == nil, info.Name())
        if path == "nested" { return fs.SkipDir }
        return err
    })
    fmt.Println(err == nil)
    fileErr := fs.WalkDir(customFS{}, "root.bin", func(path string, d fs.DirEntry, err error) error {
        info, infoErr := d.Info()
        fmt.Println("file", path, d.Name(), d.IsDir(), infoErr == nil, info.Name())
        return err
    })
    fmt.Println(fileErr == nil)
    missingErr := fs.WalkDir(os.DirFS("assets"), "missing.txt", func(path string, d fs.DirEntry, err error) error {
        fmt.Println("missing", path, d == nil, err != nil)
        return fs.SkipAll
    })
    fmt.Println(missingErr == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.workspace_files.insert("assets/z.txt".into(), "z".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        ". assets true true assets\na.txt a.txt false true a.txt\nnested nested true true nested\nz.txt z.txt false true z.txt\ntrue\nfile root.bin root.bin false true root.bin\ntrue\nmissing missing.txt true true\ntrue\n"
    );
}

#[test]
fn io_fs_file_info_to_dir_entry_projects_file_info_values() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "time"

type customInfo struct {}

func (customInfo) Name() string { return "custom" }
func (customInfo) Size() int { return 9 }
func (customInfo) Mode() fs.FileMode { return fs.ModeDir }
func (customInfo) ModTime() time.Time { return time.Unix(12, 34) }
func (customInfo) IsDir() bool { return true }
func (customInfo) Sys() interface{} { return "custom-sys" }

func main() {
    var nilInfo fs.FileInfo
    nilEntry := fs.FileInfoToDirEntry(nilInfo)
    entry := fs.FileInfoToDirEntry(customInfo{})
    info, err := entry.Info()
    fmt.Println(nilEntry == nil)
    fmt.Println(entry.Name(), entry.IsDir(), entry.Type() == fs.ModeDir, err == nil)
    fmt.Println(info.Name(), info.IsDir(), info.Mode() == fs.ModeDir, info.Size(), info.ModTime().UnixNano())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ncustom true true true\ncustom true true 9 12000000034\n"
    );
}

#[test]
fn io_fs_file_info_mod_time_uses_zero_workspace_times() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Stat("assets/config.txt")
    dirInfo, dirErr := os.Stat("assets/nested")
    fmt.Println(fileErr == nil, dirErr == nil)
    fmt.Println(fileInfo.ModTime().IsZero(), dirInfo.ModTime().IsZero())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\ntrue true\n");
}

#[test]
fn io_fs_file_info_sys_returns_workspace_nil_and_preserves_custom_values() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"
import "time"

type customInfo struct {}

func (customInfo) Name() string { return "custom" }
func (customInfo) Size() int { return 9 }
func (customInfo) Mode() fs.FileMode { return fs.ModeDir }
func (customInfo) ModTime() time.Time { return time.Unix(12, 34) }
func (customInfo) IsDir() bool { return true }
func (customInfo) Sys() interface{} { return "custom-sys" }

func main() {
    fileInfo, _ := os.Stat("assets/config.txt")
    dirInfo, _ := os.Stat("assets/nested")
    entry := fs.FileInfoToDirEntry(customInfo{})
    info, err := entry.Info()
    fmt.Println(fileInfo.Sys() == nil, dirInfo.Sys() == nil)
    fmt.Println(err == nil, info.Sys().(string))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\ntrue custom-sys\n");
}

#[test]
fn io_fs_format_dir_entry_uses_file_mode_string_rendering() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "time"

type customInfo struct {}

func (customInfo) Name() string { return "custom" }
func (customInfo) Size() int { return 9 }
func (customInfo) Mode() fs.FileMode { return fs.ModeDir }
func (customInfo) ModTime() time.Time { return time.Unix(12, 34) }
func (customInfo) IsDir() bool { return true }
func (customInfo) Sys() interface{} { return "custom-sys" }

func main() {
    entry := fs.FileInfoToDirEntry(customInfo{})
    fmt.Println(fs.FileMode(420).String())
    fmt.Println(fs.ModeDir.String())
    fmt.Println(fs.FormatDirEntry(entry))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-rw-r--r--\nd---------\nd custom/\n");
}

#[test]
fn io_fs_format_file_info_uses_mode_size_datetime_and_name() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"
import "time"

type customInfo struct {}

func (customInfo) Name() string { return "custom" }
func (customInfo) Size() int { return 9 }
func (customInfo) Mode() fs.FileMode { return fs.ModeDir | fs.FileMode(420) }
func (customInfo) ModTime() time.Time { return time.Unix(12, 34) }
func (customInfo) IsDir() bool { return true }
func (customInfo) Sys() interface{} { return "custom-sys" }

func main() {
    fileInfo, _ := os.Stat("assets/config.txt")
    dirInfo, _ := os.Stat("assets/nested")
    fmt.Println(fs.FormatFileInfo(fileInfo))
    fmt.Println(fs.FormatFileInfo(dirInfo))
    fmt.Println(fs.FormatFileInfo(customInfo{}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.workspace_files
        .insert("assets/nested/child.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "---------- 7 1970-01-01 00:00:00 config.txt\nd--------- 0 1970-01-01 00:00:00 nested/\ndrw-r--r-- 9 1970-01-01 00:00:12 custom/\n"
    );
}

#[test]
fn io_fs_file_mode_perm_and_masks_match_go_bits() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"

func main() {
    mode := fs.ModeDir | fs.ModeAppend | fs.ModeSticky | fs.FileMode(420)
    fmt.Println(fs.ModeType == fs.ModeDir|fs.ModeSymlink|fs.ModeNamedPipe|fs.ModeSocket|fs.ModeDevice|fs.ModeCharDevice|fs.ModeIrregular)
    fmt.Println(fs.ModePerm == fs.FileMode(511))
    fmt.Println(mode.Perm() == fs.FileMode(420), fs.FileMode(420).Perm() == fs.FileMode(420))
    fmt.Println(mode.Type() == fs.ModeDir, fs.ModeSymlink.Type() == fs.ModeSymlink)
    fmt.Println(mode.IsRegular(), fs.ModeAppend.IsRegular(), fs.ModeSymlink.IsRegular())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue\ntrue true\ntrue true\nfalse true false\n"
    );
}
