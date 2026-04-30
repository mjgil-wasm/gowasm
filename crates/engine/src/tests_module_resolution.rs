use gowasm_compiler::module_cache_source_path;
use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::Engine;

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[test]
fn compile_request_resolves_remote_module_sources_from_virtual_cache_files() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");

    let response = Engine::new().handle_request(EngineRequest::Compile {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import "example.com/remote/greeter"

func main() {}
"#,
            ),
            workspace_file(
                module_go_mod_path.as_str(),
                "module example.com/remote\n\ngo 1.21\n",
            ),
            workspace_file(
                module_go_file_path.as_str(),
                r#"
package greeter

func Message() string {
    return "hi"
}
"#,
            ),
        ],
        entry_path: "main.go".into(),
    });

    match response {
        EngineResponse::Diagnostics { diagnostics } => {
            assert!(
                diagnostics.is_empty(),
                "expected empty diagnostics, got {diagnostics:?}"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
