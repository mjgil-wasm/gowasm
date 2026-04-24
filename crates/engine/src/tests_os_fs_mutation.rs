use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_mutates_workspace_directories_through_os_helpers() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
    fsTop, fsTopErr := fs.ReadDir(dirfs, ".")
    fmt.Println(labels(top), topErr == nil)
    fmt.Println(labels(fsTop), fsTopErr == nil)
    fmt.Println(os.RemoveAll("existing") == nil)
    after, afterErr := os.ReadDir(".")
    fmt.Println(labels(after), afterErr == nil)
}
"#,
            ),
            WorkspaceFile {
                path: "keep.txt".into(),
                contents: "keep".into(),
            },
            WorkspaceFile {
                path: "existing/file.txt".into(),
                contents: "file".into(),
            },
            WorkspaceFile {
                path: "existing/nested/child.txt".into(),
                contents: "child".into(),
            },
        ],
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
                "true\nempty/,existing/,keep.txt,main.go true\nempty/,existing/,keep.txt,main.go true\ntrue\nempty/,keep.txt,main.go true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_mutates_workspace_files_through_os_write_file() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
"#,
            ),
            WorkspaceFile {
                path: "assets/existing.txt".into(),
                contents: "existing".into(),
            },
        ],
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
                "true\ntrue\nbeta true beta true\nexisting.txt new.txt true\ntrue\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
