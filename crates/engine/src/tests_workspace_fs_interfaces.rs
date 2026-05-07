use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_uses_imported_io_fs_interface_types_with_method_aware_helpers() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
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
"#,
        )],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });

    match response {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(
                stdout,
                "data:config.txt true\nconfig.txt info-sys true\n1 child.txt true\n1 glob:*.txt true\ntrue true data:nested/child.txt true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
