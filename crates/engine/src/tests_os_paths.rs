use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_accepts_absolute_and_cleaned_os_and_filepath_paths() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "io/fs"
import "os"
import "path/filepath"
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
    os.Setenv("HOME", "/users/alice")
    configRoot, _ := os.UserConfigDir()
    appDir := filepath.Join(configRoot, "app")
    fmt.Println(os.MkdirAll("/users/alice/.config/../.config/app", fs.FileMode(493)) == nil)

    cfgInfo, cfgErr := os.Stat("/users/alice/.config")
    appInfo, appErr := os.Stat(appDir)
    entries, entriesErr := os.ReadDir("/users/alice")
    dirfs := os.DirFS("/users/alice/./")
    fsEntries, fsErr := fs.ReadDir(dirfs, ".config")

    abs, absErr := filepath.Abs("assets/../assets")
    matches, matchesErr := filepath.Glob("/assets/../assets/*.txt")
    var walked []string
    walkErr := filepath.WalkDir("/assets/../assets", func(path string, d fs.DirEntry, err error) error {
        if err != nil {
            return err
        }
        walked = append(walked, path)
        return nil
    })
    data, readErr := os.ReadFile(filepath.Join(abs, "a.txt"))

    fmt.Println(cfgInfo.Name(), cfgInfo.IsDir(), cfgErr == nil)
    fmt.Println(appInfo.Name(), appInfo.IsDir(), appErr == nil)
    fmt.Println(labels(entries), entriesErr == nil)
    fmt.Println(labels(fsEntries), fsErr == nil)
    fmt.Println(abs, absErr == nil)
    fmt.Println(strings.Join(matches, ","), matchesErr == nil)
    fmt.Println(strings.Join(walked, ","), walkErr == nil)
    fmt.Println(string(data), readErr == nil)
}
"#,
            ),
            WorkspaceFile {
                path: "assets/a.txt".into(),
                contents: "a".into(),
            },
            WorkspaceFile {
                path: "assets/b.txt".into(),
                contents: "b".into(),
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
                "true\n.config true true\napp true true\n.config/ true\napp/ true\n/assets true\n/assets/a.txt,/assets/b.txt true\n/assets,/assets/a.txt,/assets/b.txt true\na true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
