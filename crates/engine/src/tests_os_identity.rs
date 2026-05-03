use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::handle_request;

fn main_file(contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_executes_os_hostname_and_executable() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    host, hostErr := os.Hostname()
    exe, exeErr := os.Executable()
    fmt.Println(host, hostErr == nil)
    fmt.Println(exe == "", exeErr != nil)
    fmt.Println(exeErr.Error())
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
                "js true\ntrue true\nExecutable not implemented for js\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_identity_helpers() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(
        os.Getuid(),
        os.Geteuid(),
        os.Getgid(),
        os.Getegid(),
        os.Getpid(),
        os.Getppid(),
    )
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
            assert_eq!(stdout, "-1 -1 -1 -1 -1 -1\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_getpagesize() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.Getpagesize())
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
            assert_eq!(stdout, "65536\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn run_executes_os_getgroups() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "fmt"
import "os"

func main() {
    groups, err := os.Getgroups()
    fmt.Printf("%d %t\n", len(groups), err != nil)
    fmt.Println(err.Error())
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
            assert_eq!(stdout, "0 true\nGetgroups not implemented for js\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
