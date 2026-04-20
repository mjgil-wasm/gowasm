use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_reads_workspace_files_through_os_and_io_fs() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    root, _ := os.ReadFile("root.txt")
    nested, _ := fs.ReadFile(os.DirFS("assets"), "config.txt")
    fmt.Println(string(root))
    fmt.Println(string(nested))
}
"#,
            ),
            WorkspaceFile {
                path: "root.txt".into(),
                contents: "from root".into(),
            },
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "from dirfs".into(),
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
            assert_eq!(stdout, "from root\nfrom dirfs\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reads_workspace_files_through_io_fs_sub() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    sub, err := fs.Sub(os.DirFS("assets"), "nested")
    data, readErr := fs.ReadFile(sub, "config.txt")
    fmt.Println(err == nil, readErr == nil)
    fmt.Println(string(data))
}
"#,
            ),
            WorkspaceFile {
                path: "assets/nested/config.txt".into(),
                contents: "from sub".into(),
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
            assert_eq!(stdout, "true true\nfrom sub\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_stats_workspace_files_through_io_fs_stat() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "from stat".into(),
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
            assert_eq!(stdout, "config.txt false true\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reads_workspace_directories_through_io_fs_read_dir() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
                path: "assets/other/c.txt".into(),
                contents: "c".into(),
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
                "a.txt,nested/,other/,z.txt true\nb.txt true\ntrue true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_projects_dir_entries_into_file_info() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    entries, err := fs.ReadDir(os.DirFS("assets"), ".")
    for _, entry := range entries {
        info, infoErr := entry.Info()
        fmt.Println(entry.Name(), err == nil, infoErr == nil, info.Name(), info.IsDir())
    }
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
                "a.txt true true a.txt false\nnested true true nested true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_matches_workspace_files_through_io_fs_glob() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"
import "strings"

func main() {
    matches, err := fs.Glob(os.DirFS("assets"), "*.txt")
    fmt.Println(strings.Join(matches, ","), err == nil)
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
            WorkspaceFile {
                path: "assets/keep.go".into(),
                contents: "package keep".into(),
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
            assert_eq!(stdout, "a.txt,b.txt true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_projects_workspace_file_modes() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    fileInfo, _ := os.Stat("assets/config.txt")
    dirInfo, _ := os.Stat("assets/nested")
    entries, _ := os.ReadDir("assets")
    fmt.Println(fileInfo.Mode() == fs.FileMode(0), dirInfo.Mode() == fs.ModeDir)
    for _, entry := range entries {
        info, infoErr := entry.Info()
        fmt.Println(entry.Name(), infoErr == nil, entry.Type() == info.Mode(), entry.Type() == fs.ModeDir)
    }
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
                "true true\nconfig.txt true true false\nnested true true true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_workspace_file_mode_is_dir() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
                "true false\nfalse true\nconfig.txt false\nnested true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_workspace_file_mode_is_regular() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
                "true false\ntrue false\nconfig.txt true\nnested false\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_workspace_file_mode_type() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
                "true true\ntrue true\nconfig.txt true\nnested true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_opens_workspace_files_through_fs_open_and_close() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import fs "io/fs"
import "os"

func main() {
    var filesystem fs.FS = os.DirFS("assets")
    file, err := filesystem.Open("config.txt")
    missing, missingErr := filesystem.Open("missing.txt")
    fmt.Println(err == nil, missing == nil, missingErr != nil)
    fmt.Println(file != nil)
    fmt.Println(file.Close() == nil, file.Close() != nil)
}
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "from open".into(),
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
            assert_eq!(stdout, "true true true\ntrue\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reads_workspace_files_through_fs_file_read() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "os"

func main() {
    file, err := os.DirFS("assets").Open("config.txt")
    buf := []byte(".....")
    n, readErr := file.Read(buf)
    tail := []byte("??")
    tailN, tailErr := file.Read(tail)
    eof := []byte("!")
    eofN, eofErr := file.Read(eof)
    fmt.Println(err == nil, n, readErr == nil, string(buf))
    fmt.Println(tailN, tailErr == nil, string(tail))
    fmt.Println(eofN, eofErr != nil, string(eof))
}
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "example".into(),
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
            assert_eq!(stdout, "true 5 true examp\n2 true le\n0 true !\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_reports_workspace_file_sizes_through_file_info() {
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
    entries, readErr := os.ReadDir("assets")
    fmt.Println(fileErr == nil, dirErr == nil, readErr == nil)
    for _, entry := range entries {
        info, infoErr := entry.Info()
        fmt.Println(entry.Name(), infoErr == nil, info.Size())
    }
    fmt.Println(fileInfo.Size(), dirInfo.Size())
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
                "true true true\nconfig.txt true 7\nnested true 0\n7 0\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_opens_workspace_directories_through_fs_open_and_stat() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
"#,
            ),
            WorkspaceFile {
                path: "assets/nested/config.txt".into(),
                contents: "from nested".into(),
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
            assert_eq!(stdout, "true true\nnested true\ntrue\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_stats_workspace_files_through_fs_file_stat() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
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
"#,
            ),
            WorkspaceFile {
                path: "assets/config.txt".into(),
                contents: "from stat".into(),
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
            assert_eq!(stdout, "true true\nconfig.txt false\ntrue\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
