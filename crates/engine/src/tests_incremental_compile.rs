use gowasm_compiler::module_cache_source_path;
use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::{compile_cache::CompileReuse, Engine};

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[test]
fn compile_request_reuses_exact_workspace_snapshot() {
    let mut engine = Engine::new();
    let files = vec![
        workspace_file(
            "main.go",
            r#"
package main

func main() {}
"#,
        ),
        workspace_file("notes.txt", "hello"),
    ];

    let first = engine.handle_request(EngineRequest::Compile {
        files: files.clone(),
        entry_path: "main.go".into(),
    });
    match first {
        EngineResponse::Diagnostics { diagnostics } => assert!(diagnostics.is_empty()),
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_reuse,
        CompileReuse::Recompiled,
    );

    let second = engine.handle_request(EngineRequest::Compile {
        files,
        entry_path: "main.go".into(),
    });
    match second {
        EngineResponse::Diagnostics { diagnostics } => assert!(diagnostics.is_empty()),
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_reuse,
        CompileReuse::ReusedExact,
    );
}

#[test]
fn run_request_reuses_program_when_only_runtime_data_changes() {
    let mut engine = Engine::new();
    let first = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file(
                "main.go",
                r#"
package main

import "fmt"

func main() {
    fmt.Println("hi")
}
"#,
            ),
            workspace_file("state.json", "one"),
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    match first {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "hi\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let second = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file(
                "main.go",
                r#"
package main

import "fmt"

func main() {
    fmt.Println("hi")
}
"#,
            ),
            workspace_file("state.json", "two"),
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    match second {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "hi\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_reuse,
        CompileReuse::ReusedUnaffected,
    );
}

#[test]
fn compile_request_recompiles_when_reachable_module_sources_change() {
    let mut engine = Engine::new();
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");

    let first = engine.handle_request(EngineRequest::Compile {
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
    match first {
        EngineResponse::Diagnostics { diagnostics } => assert!(diagnostics.is_empty()),
        other => panic!("unexpected response: {other:?}"),
    }

    let second = engine.handle_request(EngineRequest::Compile {
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
    return "hello"
}
"#,
            ),
        ],
        entry_path: "main.go".into(),
    });
    match second {
        EngineResponse::Diagnostics { diagnostics } => assert!(diagnostics.is_empty()),
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_reuse,
        CompileReuse::RecompiledAffectedPackages,
    );
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_recompiled_import_paths,
        vec!["example.com/remote/greeter".to_string()],
    );
}

#[test]
fn run_request_recompiles_only_affected_packages_when_package_shapes_stay_stable() {
    let mut engine = Engine::new();
    let first = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import (
    "fmt"
    "example.com/app/helper"
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message(), helper.Tag())
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

func Message() string {
    return "hi"
}
"#,
            ),
            workspace_file(
                "helper/helper.go",
                r#"
package helper

func Tag() string {
    return "helper"
}
"#,
            ),
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    match first {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "hi helper\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let second = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import (
    "fmt"
    "example.com/app/helper"
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message(), helper.Tag())
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

func Message() string {
    return "hello"
}
"#,
            ),
            workspace_file(
                "helper/helper.go",
                r#"
package helper

func Tag() string {
    return "helper"
}
"#,
            ),
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    match second {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "hello helper\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_reuse,
        CompileReuse::RecompiledAffectedPackages,
    );
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_recompiled_import_paths,
        vec!["example.com/app/lib".to_string()],
    );
}

#[test]
fn run_request_recompiles_imported_generic_instance_packages_without_full_rebuild() {
    let mut engine = Engine::new();
    let first = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import (
    "fmt"
    "example.com/app/report"
)

func main() {
    box := report.First()
    fmt.Println(box.Label, box.Speak())
}
"#,
            ),
            workspace_file(
                "report/report.go",
                r#"
package report

import "example.com/app/lib"

func First() lib.Box[string] {
    return lib.First()
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

type Box[T any] struct {
    Label T
}

func First() Box[string] {
    return Box[string]{Label: "Ada"}
}

func (box Box[T]) Speak() string {
    return "box-a"
}
"#,
            ),
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    match first {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "Ada box-a\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let second = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import (
    "fmt"
    "example.com/app/report"
)

func main() {
    box := report.First()
    fmt.Println(box.Label, box.Speak())
}
"#,
            ),
            workspace_file(
                "report/report.go",
                r#"
package report

import "example.com/app/lib"

func First() lib.Box[string] {
    return lib.First()
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

type Box[T any] struct {
    Label T
}

func First() Box[string] {
    return Box[string]{Label: "Lin"}
}

func (box Box[T]) Speak() string {
    return "box-b"
}
"#,
            ),
        ],
        entry_path: "main.go".into(),
        host_time_unix_nanos: None,
        host_time_unix_millis: None,
    });
    match second {
        EngineResponse::RunResult {
            stdout,
            diagnostics,
        } => {
            assert_eq!(stdout, "Lin box-b\n");
            assert!(diagnostics.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_reuse,
        CompileReuse::RecompiledAffectedPackages,
    );
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected compile session")
            .last_recompiled_import_paths,
        vec!["example.com/app/lib".to_string()],
    );
}
