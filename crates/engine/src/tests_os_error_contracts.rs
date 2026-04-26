use gowasm_host_types::{EngineRequest, EngineResponse};

use super::handle_request;

fn main_file(contents: &str) -> gowasm_host_types::WorkspaceFile {
    gowasm_host_types::WorkspaceFile {
        path: "main.go".into(),
        contents: contents.into(),
    }
}

#[test]
fn run_preserves_os_invalid_path_and_missing_path_error_contracts() {
    let response = handle_request(EngineRequest::Run {
        files: vec![main_file(
            r#"
package main
import "errors"
import "fmt"
import "io/fs"
import "os"

func main() {
    _, missingErr := os.ReadFile("missing")
    _, readInvalid := os.ReadFile("")
    _, dirInvalid := os.ReadDir("")
    _, statInvalid := os.Stat("")
    _, lstatInvalid := os.Lstat("")
    mkdirInvalid := os.MkdirAll("", fs.FileMode(493))
    removeInvalid := os.RemoveAll("")

    fmt.Println(errors.Is(missingErr, os.ErrNotExist), errors.Unwrap(missingErr) == os.ErrNotExist)
    fmt.Println(errors.Is(readInvalid, os.ErrInvalid), errors.Unwrap(readInvalid) == os.ErrInvalid)
    fmt.Println(errors.Is(dirInvalid, os.ErrInvalid), errors.Is(statInvalid, os.ErrInvalid), errors.Is(lstatInvalid, os.ErrInvalid))
    fmt.Println(errors.Is(mkdirInvalid, os.ErrInvalid), errors.Is(removeInvalid, os.ErrInvalid))
    fmt.Println(errors.Unwrap(mkdirInvalid) == os.ErrInvalid, errors.Unwrap(removeInvalid) == os.ErrInvalid)
    fmt.Println(readInvalid.Error())
    fmt.Println(mkdirInvalid.Error())
    fmt.Println(removeInvalid.Error())
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
                "true false\ntrue true\ntrue true true\ntrue true\ntrue true\nopen : invalid path\nmkdir : invalid path\nremoveall : invalid path\n"
            );
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
