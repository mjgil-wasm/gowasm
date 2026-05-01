use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_executes_os_environ_and_expand_env() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "errors"
import "fmt"
import "os"
import "strings"

func main() {
    os.Setenv("B", "2")
    os.Setenv("A", "1")
    fmt.Println(strings.Join(os.Environ(), ","))
    fmt.Printf("[%s]\n", os.ExpandEnv("value=$A ${MISSING} $"))
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
            assert_eq!(stdout, "A=1,B=2\n[value=1  $]\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_expand_callback() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    marker := "["
    fmt.Printf("%s\n", os.Expand("$A ${B} $", func(name string) string {
        return marker + name + "]"
    }))
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
            assert_eq!(stdout, "[A] [B] $\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_read_dir() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "os"
import "strings"

func main() {
    root, rootErr := os.ReadDir(".")
    entries, err := os.ReadDir("assets")
    nested, nestedErr := os.ReadDir("assets/nested")
    fileEntries, fileErr := os.ReadDir("assets/a.txt")
    missing, missingErr := os.ReadDir("missing")

    var topNames []string
    for _, entry := range root {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        topNames = append(topNames, name)
    }

    var rootNames []string
    for _, entry := range entries {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        rootNames = append(rootNames, name)
    }

    var nestedNames []string
    for _, entry := range nested {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        nestedNames = append(nestedNames, name)
    }

    fmt.Println(strings.Join(topNames, ","), rootErr == nil)
    fmt.Println(strings.Join(rootNames, ","), err == nil)
    fmt.Println(strings.Join(nestedNames, ","), nestedErr == nil)
    fmt.Println(fileEntries == nil, fileErr != nil)
    fmt.Println(missing == nil, missingErr != nil)
}
"#,
            ),
            WorkspaceFile {
                path: "root.txt".into(),
                contents: "root".into(),
            },
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
                "assets/,main.go,root.txt true\na.txt,nested/,other/,z.txt true\nb.txt true\ntrue true\ntrue true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_stat() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Stat("assets/a.txt")
    dirInfo, dirErr := os.Stat("assets/nested")
    missing, missingErr := os.Stat("missing")

    fmt.Println(fileErr == nil, fileInfo.Name(), fileInfo.IsDir())
    fmt.Println(dirErr == nil, dirInfo.Name(), dirInfo.IsDir())
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
            assert_eq!(stdout, "true a.txt false\ntrue nested true\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_lstat() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Lstat("assets/a.txt")
    dirInfo, dirErr := os.Lstat("assets/nested")
    missing, missingErr := os.Lstat("missing")

    fmt.Println(fileErr == nil, fileInfo.Name(), fileInfo.IsDir())
    fmt.Println(dirErr == nil, dirInfo.Name(), dirInfo.IsDir())
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
            assert_eq!(stdout, "true a.txt false\ntrue nested true\ntrue true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_getwd() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    dir, err := os.Getwd()
    fmt.Println(dir, err == nil)
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
            assert_eq!(stdout, "/ true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_directory_helpers() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.TempDir())
    os.Setenv("TMPDIR", "/sandbox/tmp")
    fmt.Println(os.TempDir())

    os.Clearenv()
    home, homeErr := os.UserHomeDir()
    fmt.Println(home == "", homeErr.Error())

    os.Setenv("HOME", "/users/alice")
    home, homeErr = os.UserHomeDir()
    cache, cacheErr := os.UserCacheDir()
    config, configErr := os.UserConfigDir()
    fmt.Println(home, homeErr == nil)
    fmt.Println(cache, cacheErr == nil)
    fmt.Println(config, configErr == nil)

    os.Unsetenv("HOME")
    os.Setenv("XDG_CACHE_HOME", "/cache")
    os.Setenv("XDG_CONFIG_HOME", "/config")
    cache, cacheErr = os.UserCacheDir()
    config, configErr = os.UserConfigDir()
    fmt.Println(cache, cacheErr == nil)
    fmt.Println(config, configErr == nil)

    os.Clearenv()
    _, cacheErr = os.UserCacheDir()
    _, configErr = os.UserConfigDir()
    fmt.Println(cacheErr.Error())
    fmt.Println(configErr.Error())
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
                "/tmp\n/sandbox/tmp\ntrue $HOME is not defined\n/users/alice true\n/users/alice/.cache true\n/users/alice/.config true\n/cache true\n/config true\nneither $XDG_CACHE_HOME nor $HOME are defined\nneither $XDG_CONFIG_HOME nor $HOME are defined\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_is_path_separator() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.IsPathSeparator('/'), os.IsPathSeparator('a'), os.IsPathSeparator('\\'))
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
            assert_eq!(stdout, "true false false\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_exposes_os_path_separator_constant() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.PathSeparator == '/', os.IsPathSeparator(os.PathSeparator))
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
            assert_eq!(stdout, "true true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_exposes_os_path_list_separator_and_dev_null_constants() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.PathListSeparator == ':', os.DevNull == "/dev/null")
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
            assert_eq!(stdout, "true true\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_is_not_exist() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "errors"
import "fmt"
import "os"

func main() {
    _, readErr := os.ReadFile("missing")
    _, dirErr := os.ReadDir("missing")
    _, statErr := os.Stat("missing")
    _, invalidErr := os.ReadFile("")
    other := errors.New("other")

    fmt.Println(os.IsNotExist(readErr), os.IsNotExist(dirErr), os.IsNotExist(statErr))
    fmt.Println(os.IsNotExist(invalidErr), os.IsNotExist(other), os.IsNotExist(nil))
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
            assert_eq!(stdout, "true true true\nfalse false false\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_same_file() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, _ := os.Stat("assets/a.txt")
    samePath, _ := os.Lstat("assets/a.txt")
    sameShape, _ := os.Stat("other/a.txt")
    dirInfo, _ := os.Stat("assets")

    fsys := os.DirFS(".")
    file, _ := fsys.Open("assets/a.txt")
    openedInfo, _ := file.Stat()

    fmt.Println(os.SameFile(fileInfo, samePath))
    fmt.Println(os.SameFile(fileInfo, sameShape))
    fmt.Println(os.SameFile(fileInfo, dirInfo))
    fmt.Println(os.SameFile(fileInfo, openedInfo))
    fmt.Println(os.SameFile(nil, fileInfo))
}
"#,
            ),
            WorkspaceFile {
                path: "assets/a.txt".into(),
                contents: "x".into(),
            },
            WorkspaceFile {
                path: "assets/nested/b.txt".into(),
                contents: "b".into(),
            },
            WorkspaceFile {
                path: "other/a.txt".into(),
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
            assert_eq!(stdout, "true\nfalse\nfalse\ntrue\nfalse\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_error_values_and_helpers() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "errors"
import "fmt"
import "os"

var globalErr = os.ErrClosed
var invalid = os.ErrInvalid
var timeout = os.ErrDeadlineExceeded

func main() {
    _, missingErr := os.ReadFile("missing")
    wrappedMissing := fmt.Errorf("wrap: %w", missingErr)
    wrappedPerm := fmt.Errorf("wrap: %w", os.ErrPermission)
    wrappedExist := fmt.Errorf("wrap: %w", os.ErrExist)
    wrappedTimeout := fmt.Errorf("wrap: %w", os.ErrDeadlineExceeded)
    syscallErr := os.NewSyscallError("open", os.ErrNotExist)
    nilSyscallErr := os.NewSyscallError("open", nil)
    timeoutSyscallErr := os.NewSyscallError("read", os.ErrDeadlineExceeded)

    fmt.Println(os.ErrInvalid.Error())
    fmt.Println(os.ErrPermission.Error())
    fmt.Println(os.ErrExist.Error())
    fmt.Println(os.ErrNotExist.Error())
    fmt.Println(os.ErrClosed.Error())
    fmt.Println(os.ErrDeadlineExceeded.Error())
    fmt.Println(os.ErrNoDeadline.Error())
    fmt.Println(os.IsExist(os.ErrExist), os.IsExist(wrappedExist), os.IsExist(missingErr))
    fmt.Println(os.IsNotExist(os.ErrNotExist), os.IsNotExist(wrappedMissing))
    fmt.Println(os.IsPermission(os.ErrPermission), os.IsPermission(wrappedPerm), os.IsPermission(missingErr))
    fmt.Println(os.IsTimeout(os.ErrDeadlineExceeded), os.IsTimeout(wrappedTimeout), os.IsTimeout(os.ErrInvalid), os.IsTimeout(missingErr), os.IsTimeout(nil))
    fmt.Println(syscallErr.Error())
    fmt.Println(nilSyscallErr == nil)
    fmt.Println(errors.Is(syscallErr, os.ErrNotExist), errors.Is(timeoutSyscallErr, os.ErrDeadlineExceeded))
    fmt.Println(os.IsTimeout(timeoutSyscallErr), os.IsTimeout(os.ErrNoDeadline))
    fmt.Println(globalErr == os.ErrClosed)
    fmt.Println(invalid == os.ErrInvalid, timeout == os.ErrDeadlineExceeded)
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
                "invalid argument\npermission denied\nfile already exists\nfile does not exist\nfile already closed\ni/o timeout\nfile type does not support deadline\ntrue true false\ntrue true\ntrue true false\ntrue true false false false\nopen: file does not exist\ntrue\ntrue true\ntrue false\ntrue\ntrue true\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_errors_is_with_os_path_errors() {
    let response = handle_request(EngineRequest::Run {
        files: vec![
            main_file(
                r#"
package main
import "errors"
import "fmt"
import "os"

func main() {
    _, missingErr := os.ReadFile("missing")
    wrappedMissing := fmt.Errorf("wrapped: %w", missingErr)

    fsys := os.DirFS(".")
    file, _ := fsys.Open("assets/a.txt")
    _ = file.Close()
    closedErr := file.Close()

    buf := make([]byte, 1)
    _, readClosedErr := file.Read(buf)

    fmt.Println(errors.Is(missingErr, os.ErrNotExist))
    fmt.Println(errors.Is(wrappedMissing, os.ErrNotExist))
    fmt.Println(errors.Is(closedErr, os.ErrClosed))
    fmt.Println(errors.Is(readClosedErr, os.ErrClosed))
    fmt.Println(errors.Is(missingErr, os.ErrClosed))
}
"#,
            ),
            WorkspaceFile {
                path: "assets/a.txt".into(),
                contents: "a".into(),
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
            assert_eq!(stdout, "true\ntrue\ntrue\ntrue\nfalse\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
