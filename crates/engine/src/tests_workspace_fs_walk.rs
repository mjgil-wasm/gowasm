use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_walks_workspace_files_through_io_fs_walk_dir() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    err := fs.WalkDir(os.DirFS("assets"), ".", func(path string, d fs.DirEntry, err error) error {
        info, infoErr := d.Info()
        fmt.Println(path, d.Name(), d.IsDir(), infoErr == nil, info.Name())
        if path == "nested" { return fs.SkipDir }
        return err
    })
    fmt.Println(err == nil)
    missingErr := fs.WalkDir(os.DirFS("assets"), "missing.txt", func(path string, d fs.DirEntry, err error) error {
        fmt.Println("missing", path, d == nil, err != nil)
        return fs.SkipAll
    })
    fmt.Println(missingErr == nil)
}
"#,
            ),
            WorkspaceFile {
                path: "assets/a.txt".into(),
                contents: "a".into(),
            },
            WorkspaceFile {
                path: "assets/nested/b.txt".into(),
                contents: "b".into(),
            },
            WorkspaceFile {
                path: "assets/z.txt".into(),
                contents: "z".into(),
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
                ". assets true true assets\na.txt a.txt false true a.txt\nnested nested true true nested\nz.txt z.txt false true z.txt\ntrue\nmissing missing.txt true true\ntrue\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_projects_file_info_to_dir_entry_values() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
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
                "true\ncustom true true true\ncustom true true 9 12000000034\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_workspace_file_info_zero_mod_times() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Stat("assets/config.txt")
    dirInfo, dirErr := os.Stat("assets/nested")
    fmt.Println(fileErr == nil, dirErr == nil)
    fmt.Println(fileInfo.ModTime().IsZero(), dirInfo.ModTime().IsZero())
}
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "example".into(),
            },
            WorkspaceFile {
                path: "assets/nested/child.txt".into(),
                contents: "x".into(),
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
            assert_eq!(stdout, "true true\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_workspace_file_info_sys_values() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "example".into(),
            },
            WorkspaceFile {
                path: "assets/nested/child.txt".into(),
                contents: "x".into(),
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
            assert_eq!(stdout, "true true\ntrue custom-sys\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_formats_dir_entries_with_file_mode_strings() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
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
            assert_eq!(stdout, "-rw-r--r--\nd---------\nd custom/\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_formats_file_info_with_datetime_layouts() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "example".into(),
            },
            WorkspaceFile {
                path: "assets/nested/child.txt".into(),
                contents: "x".into(),
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
                "---------- 7 1970-01-01 00:00:00 config.txt\nd--------- 0 1970-01-01 00:00:00 nested/\ndrw-r--r-- 9 1970-01-01 00:00:12 custom/\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_file_mode_perm_and_masks() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
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
                "true\ntrue\ntrue true\ntrue true\nfalse true false\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
