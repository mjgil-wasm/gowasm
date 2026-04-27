use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn os_read_file_reads_workspace_files() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    data, err := os.ReadFile("config.txt")
    fmt.Println(string(data))
    fmt.Println(err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("config.txt".into(), "hello".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\ntrue\n");
}

#[test]
fn io_fs_read_file_reads_dirfs_values_and_validates_paths() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    var filesystem fs.FS = os.DirFS("assets")
    data, err := fs.ReadFile(filesystem, "config.txt")
    fmt.Println(fs.ValidPath("config.txt"), fs.ValidPath("../bad"))
    fmt.Println(string(data))
    fmt.Println(err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "nested".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false\nnested\ntrue\n");
}

#[test]
fn io_fs_stat_prefers_stat_methods_before_open_fallback() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "time"

type info struct {}

func (info) Name() string { return "from-stat" }
func (info) Size() int { return 0 }
func (info) Mode() fs.FileMode { return fs.ModeDir }
func (info) ModTime() time.Time { return time.Time{} }
func (info) IsDir() bool { return true }
func (info) Sys() interface{} { return nil }

type customFS struct {}

func (customFS) Stat(name string) (fs.FileInfo, error) {
    return info{}, nil
}

func (customFS) Open(name string) (fs.File, error) {
    return nil, fmt.Errorf("open called")
}

func main() {
    info, err := fs.Stat(customFS{}, "ignored.txt")
    fmt.Println(info.Name(), info.IsDir(), err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "from-stat true true\n");
}

#[test]
fn io_fs_stat_falls_back_to_open_and_file_stat_for_dirfs() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    info, err := fs.Stat(os.DirFS("assets"), "config.txt")
    missing, missingErr := fs.Stat(os.DirFS("assets"), "missing.txt")
    fmt.Println(info.Name(), info.IsDir(), err == nil)
    fmt.Println(missing == nil, missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "nested".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "config.txt false true\ntrue true\n");
}

#[test]
fn io_fs_read_dir_prefers_custom_readdir_methods() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "strings"
import "time"

type info struct {
    name string
    dir bool
}

func (i info) Name() string { return i.name }
func (i info) Size() int { return len(i.name) }
func (i info) Mode() fs.FileMode {
    if i.dir {
        return fs.ModeDir
    }
    return 0
}
func (i info) ModTime() time.Time { return time.Time{} }
func (i info) IsDir() bool { return i.dir }
func (i info) Sys() interface{} { return nil }

type entry struct {
    name string
    dir bool
}

func (e entry) Name() string { return e.name }
func (e entry) IsDir() bool { return e.dir }
func (e entry) Type() fs.FileMode {
    if e.dir {
        return fs.ModeDir
    }
    return 0
}
func (e entry) Info() (fs.FileInfo, error) { return info{name: e.name, dir: e.dir}, nil }

type customFS struct {}

func (customFS) ReadDir(name string) ([]fs.DirEntry, error) {
    var first fs.DirEntry = entry{name: "b", dir: false}
    var second fs.DirEntry = entry{name: "a", dir: true}
    return []fs.DirEntry{first, second}, nil
}

func (customFS) Open(name string) (fs.File, error) {
    return nil, fmt.Errorf("open called")
}

func main() {
    entries, err := fs.ReadDir(customFS{}, ".")
    var names []string
    for _, entry := range entries {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        names = append(names, name)
    }
    fmt.Println(strings.Join(names, ","), err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b,a/ true\n");
}

#[test]
fn io_fs_read_dir_reads_workspace_directories_and_sorts_entries() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
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
    root, err := fs.ReadDir(os.DirFS("assets"), ".")
    nested, nestedErr := fs.ReadDir(os.DirFS("assets"), "nested")
    missing, missingErr := fs.ReadDir(os.DirFS("assets"), "missing")
    fmt.Println(labels(root), err == nil)
    fmt.Println(labels(nested), nestedErr == nil)
    fmt.Println(missing == nil, missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.workspace_files
        .insert("assets/other/c.txt".into(), "c".into());
    vm.workspace_files.insert("assets/z.txt".into(), "z".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "a.txt,nested/,other/,z.txt true\nb.txt true\ntrue true\n"
    );
}

#[test]
fn io_fs_sub_re_roots_workspace_dirfs_values() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    sub, err := fs.Sub(os.DirFS("assets"), "nested")
    _, bad := fs.Sub(os.DirFS("assets"), "../bad")
    data, readErr := fs.ReadFile(sub, "config.txt")
    fmt.Println(err == nil, bad != nil, readErr == nil)
    fmt.Println(string(data))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/nested/config.txt".into(), "subdir".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true\nsubdir\n");
}

#[test]
fn io_fs_imported_interface_types_match_method_aware_helpers() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "time"

type info struct {
    name string
    dir bool
}

func (i info) Name() string { return i.name }
func (i info) Size() int { return len(i.name) }
func (i info) Mode() fs.FileMode {
    if i.dir {
        return fs.ModeDir
    }
    return 0
}
func (i info) ModTime() time.Time { return time.Unix(1, 2) }
func (i info) IsDir() bool { return i.dir }
func (i info) Sys() interface{} { return "info-sys" }

type entry struct {
    name string
    dir bool
}

func (e entry) Name() string { return e.name }
func (e entry) IsDir() bool { return e.dir }
func (e entry) Type() fs.FileMode {
    if e.dir {
        return fs.ModeDir
    }
    return 0
}
func (e entry) Info() (fs.FileInfo, error) { return info{name: e.name, dir: e.dir}, nil }

type customFS struct {
    prefix string
}

func (f customFS) full(name string) string {
    if f.prefix == "" {
        return name
    }
    if name == "." {
        return f.prefix
    }
    return f.prefix + "/" + name
}

func (customFS) Open(name string) (fs.File, error) { return nil, fmt.Errorf("open called") }
func (f customFS) ReadFile(name string) ([]byte, error) { return []byte("data:" + f.full(name)), nil }
func (f customFS) Stat(name string) (fs.FileInfo, error) { return info{name: f.full(name), dir: false}, nil }
func (f customFS) ReadDir(name string) ([]fs.DirEntry, error) { return []fs.DirEntry{entry{name: f.full("child.txt"), dir: false}}, nil }
func (f customFS) Glob(pattern string) ([]string, error) { return []string{f.full("glob:" + pattern)}, nil }
func (f customFS) Sub(dir string) (fs.FS, error) { return customFS{prefix: f.full(dir)}, nil }

func main() {
    root := customFS{}
    var readfs fs.ReadFileFS = root
    var statfs fs.StatFS = root
    var dirfs fs.ReadDirFS = root
    var globfs fs.GlobFS = root
    var subfs fs.SubFS = root

    data, readErr := fs.ReadFile(readfs, "config.txt")
    info, statErr := fs.Stat(statfs, "config.txt")
    entries, dirErr := fs.ReadDir(dirfs, ".")
    matches, globErr := fs.Glob(globfs, "*.txt")
    sub, subErr := fs.Sub(subfs, "nested")
    subData, subReadErr := fs.ReadFile(sub, "child.txt")

    fmt.Println(string(data), readErr == nil)
    fmt.Println(info.Name(), info.Sys().(string), statErr == nil)
    fmt.Println(len(entries), entries[0].Name(), dirErr == nil)
    fmt.Println(len(matches), matches[0], globErr == nil)
    fmt.Println(sub != nil, subErr == nil, string(subData), subReadErr == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "data:config.txt true\nconfig.txt info-sys true\n1 child.txt true\n1 glob:*.txt true\ntrue true data:nested/child.txt true\n"
    );
}

#[test]
fn io_fs_glob_matches_workspace_files_from_dirfs_and_subfs() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"
import "strings"

func main() {
    matches, err := fs.Glob(os.DirFS("assets"), "*.txt")
    sub, subErr := fs.Sub(os.DirFS("assets"), "nested")
    nested, nestedErr := fs.Glob(sub, "*.txt")
    none, noneErr := fs.Glob(os.DirFS("assets"), "*.md")
    _, bad := fs.Glob(os.DirFS("assets"), "[")

    fmt.Println(strings.Join(matches, ","), err == nil)
    fmt.Println(strings.Join(nested, ","), subErr == nil, nestedErr == nil)
    fmt.Println(none == nil, noneErr == nil, bad != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files.insert("assets/b.txt".into(), "b".into());
    vm.workspace_files
        .insert("assets/nested/c.txt".into(), "c".into());
    vm.workspace_files
        .insert("assets/keep.go".into(), "package keep".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "a.txt,b.txt true\nc.txt true true\ntrue true true\n"
    );
}

#[test]
fn io_fs_open_returns_workspace_file_objects_with_close() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    var filesystem fs.FS = os.DirFS("assets")
    file, err := filesystem.Open("config.txt")
    direct, directErr := os.DirFS("assets").Open("config.txt")
    missing, missingErr := filesystem.Open("missing.txt")

    fmt.Println(err == nil, directErr == nil, missing == nil, missingErr != nil)
    fmt.Println(file != nil, direct != nil)
    fmt.Println(file.Close() == nil, file.Close() != nil, direct.Close() == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "nested".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true true true\ntrue true\ntrue true true\n"
    );
}

#[test]
fn io_fs_open_and_stat_support_workspace_directories() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    file, err := os.DirFS("assets").Open("nested")
    info, statErr := file.Stat()
    fmt.Println(err == nil, statErr == nil)
    fmt.Println(info.Name(), info.IsDir())
    fmt.Println(file.Close() == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/nested/config.txt".into(), "nested".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\nnested true\ntrue\n");
}

#[test]
fn io_fs_dir_entry_info_returns_file_info_views() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    entries, err := os.ReadDir("assets")
    for _, entry := range entries {
        info, infoErr := entry.Info()
        fmt.Println(entry.Name(), err == nil, infoErr == nil, info.Name(), info.IsDir())
    }
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "a.txt true true a.txt false\nnested true true nested true\n"
    );
}

#[test]
fn io_fs_file_stat_returns_partial_file_info_views() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    file, err := os.DirFS("assets").Open("config.txt")
    info, statErr := file.Stat()
    fmt.Println(err == nil, statErr == nil)
    fmt.Println(info.Name(), info.IsDir())
    fmt.Println(file.Close() == nil)
    closedInfo, closedErr := file.Stat()
    fmt.Println(closedInfo == nil, closedErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "nested".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\nconfig.txt false\ntrue\ntrue true\n"
    );
}

#[test]
fn io_fs_file_read_mutates_byte_buffers_for_workspace_files() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    file, err := os.DirFS("assets").Open("config.txt")
    buf := []byte(".....")
    n, readErr := file.Read(buf)
    fmt.Println(err == nil, n, readErr == nil, string(buf))

    tail := []byte("??")
    tailN, tailErr := file.Read(tail)
    fmt.Println(tailN, tailErr == nil, string(tail))

    eofBuf := []byte("!")
    eofN, eofErr := file.Read(eofBuf)
    fmt.Println(eofN, eofErr != nil, string(eofBuf))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/config.txt".into(), "example".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true 5 true examp\n2 true le\n0 true !\n");
}

#[test]
fn io_fs_plain_fs_fallbacks_support_read_file_and_sub() {
    let source = r#"
package main
import "fmt"
import fs "io/fs"
import "time"

type info struct {
    name string
    dir bool
    size int
}

func (i info) Name() string { return i.name }
func (i info) Size() int { return i.size }
func (i info) Mode() fs.FileMode {
    if i.dir {
        return fs.ModeDir
    }
    return 0
}
func (i info) ModTime() time.Time { return time.Time{} }
func (i info) IsDir() bool { return i.dir }
func (i info) Sys() interface{} { return nil }

type textFile struct {
    name string
    data string
}

func (f textFile) Stat() (fs.FileInfo, error) {
    return info{name: f.name, size: len(f.data)}, nil
}

func (f textFile) Read(p []byte) (int, error) {
    n := len(p)
    if n > len(f.data) {
        n = len(f.data)
    }
    for i := 0; i < n; i++ {
        p[i] = f.data[i]
    }
    return n, fmt.Errorf("EOF")
}

func (f textFile) Close() error { return nil }

type customFS struct{}

func (customFS) Open(name string) (fs.File, error) {
    switch name {
    case "nested/config.txt":
        return textFile{name: "config.txt", data: "subdir"}, nil
    default:
        return nil, fmt.Errorf("open %s: file does not exist", name)
    }
}

func main() {
    baseData, baseErr := fs.ReadFile(customFS{}, "nested/config.txt")
    sub, subErr := fs.Sub(customFS{}, "nested")
    subData, subReadErr := fs.ReadFile(sub, "config.txt")

    fmt.Println(baseErr == nil, subErr == nil, subReadErr == nil)
    fmt.Println(sub != nil)
    fmt.Println(string(baseData), string(subData))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true true\ntrue\nsubdir subdir\n"
    );
}
