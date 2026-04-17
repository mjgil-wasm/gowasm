use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};

use super::{compile_cache::CompileReuse, Engine};

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[test]
fn run_request_recompiles_downstream_packages_when_dependency_export_surface_changes() {
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
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message())
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
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import (
    "fmt"
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message())
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

func Message() int {
    return 1
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
            assert_eq!(stdout, "1\n");
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
        vec![
            "example.com/app/lib".to_string(),
            "example.com/app".to_string(),
        ],
    );
}

#[test]
fn compile_request_recompiles_only_changed_package_when_dependency_shape_changes_but_exports_do_not(
) {
    let mut engine = Engine::new();
    let first = engine.handle_request(EngineRequest::Compile {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import "example.com/app/lib"

func main() {}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

func Message() string {
    return "hi"
}

func helper() string {
    return "helper"
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

import "example.com/app/lib"

func main() {}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

func Message() string {
    return "hi"
}

func Extra() string {
    return "extra"
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
        vec!["example.com/app/lib".to_string()],
    );
}

#[test]
fn compile_request_recompiles_changed_and_new_packages_when_new_reachable_package_is_added() {
    let mut engine = Engine::new();
    let first = engine.handle_request(EngineRequest::Compile {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
            workspace_file(
                "main.go",
                r#"
package main

import "example.com/app/lib"

func main() {}
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

import "example.com/app/lib"

func main() {}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

import "example.com/app/helper"

func Message() string {
    return helper.Message()
}
"#,
            ),
            workspace_file(
                "helper/helper.go",
                r#"
package helper

func Message() string {
    return "helper"
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
        vec![
            "example.com/app/helper".to_string(),
            "example.com/app/lib".to_string(),
        ],
    );
}

#[test]
fn run_request_recompiles_entry_package_when_new_go_file_is_added_to_existing_package_directory() {
    let mut engine = Engine::new();
    let first = engine.handle_request(EngineRequest::Run {
        files: vec![
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
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
            workspace_file("go.mod", "module example.com/app\n\ngo 1.21\n"),
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
            workspace_file(
                "extra.go",
                r#"
package main

import "fmt"

func init() {
    fmt.Println("extra")
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
            assert_eq!(stdout, "extra\nhi\n");
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
        vec!["example.com/app".to_string()],
    );
}

#[test]
fn run_request_handles_deleted_reachable_packages_without_full_rebuild() {
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
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message())
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

import "example.com/app/helper"

func Message() string {
    return "hi " + helper.Tag()
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
    "example.com/app/lib"
)

func main() {
    fmt.Println(lib.Message())
}
"#,
            ),
            workspace_file(
                "lib/lib.go",
                r#"
package lib

func Message() string {
    return "hi direct"
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
            assert_eq!(stdout, "hi direct\n");
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
fn run_request_recompiles_downstream_packages_for_generic_export_shape_changes() {
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
    "example.com/app/lib"
)

func main() {
    box := lib.First()
    fmt.Println(box.Label, box.Speak())
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
    "example.com/app/lib"
)

func main() {
    box := lib.First()
    fmt.Println(box.Label, box.Speak())
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

func (box Box[T]) Speak() int {
    return 7
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
            assert_eq!(stdout, "Ada 7\n");
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
        vec![
            "example.com/app/lib".to_string(),
            "example.com/app".to_string(),
        ],
    );
}
