use gowasm_lexer::Span;

use crate::{
    compile_workspace, module_cache_source_path, take_last_compile_error_context, CompileError,
    SourceInput,
};

#[test]
fn resolves_local_workspace_imports_through_go_mod() {
    let program = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/lib"

func main() {}
"#,
            },
            SourceInput {
                path: "lib/lib.go",
                source: r#"
package lib

func Message() string {
    return "hello"
}
"#,
            },
        ],
        "main.go",
    );

    assert!(
        program.is_ok(),
        "expected local import resolution to succeed"
    );
}

#[test]
fn resolves_remote_module_imports_through_virtual_module_cache_files() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");
    let program = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/remote/greeter"

func main() {}
"#,
            },
            SourceInput {
                path: module_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: module_go_file_path.as_str(),
                source: r#"
package greeter

func Message() string {
    return "hi"
}
"#,
            },
        ],
        "main.go",
    );

    assert!(
        program.is_ok(),
        "expected remote module import resolution to succeed"
    );
}

#[test]
fn missing_import_reports_importer_package_path() {
    let main_source = r#"
package main

import "example.com/app/lib"

func main() {}
"#;
    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: main_source,
            },
        ],
        "main.go",
    )
    .expect_err("expected unresolved import to fail");

    match error {
        CompileError::UnresolvedImportPath { importer, path } => {
            assert_eq!(importer, "example.com/app");
            assert_eq!(path, "example.com/app/lib");
            let context = take_last_compile_error_context()
                .expect("unresolved import should record compile error context");
            assert_eq!(context.file_path, "main.go");
            assert_eq!(
                &main_source[context.span_start..context.span_end],
                "\"example.com/app/lib\""
            );
        }
        other => panic!("expected unresolved import error, got {other:?}"),
    }
}

#[test]
fn detects_simple_local_import_cycles_with_source_spans() {
    let main_source = r#"
package main

import "example.com/app/lib"

func main() {}
"#;
    let lib_source = r#"
package lib

import "example.com/app"
"#;

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: main_source,
            },
            SourceInput {
                path: "lib/lib.go",
                source: lib_source,
            },
        ],
        "main.go",
    )
    .expect_err("expected import cycle to fail");

    match error {
        CompileError::ImportCycle {
            cycle,
            source_path,
            span,
            ..
        } => {
            assert_eq!(
                cycle,
                vec![
                    "example.com/app".to_string(),
                    "example.com/app/lib".to_string(),
                    "example.com/app".to_string(),
                ]
            );
            assert_eq!(source_path, "lib/lib.go");
            assert_eq!(span_text(lib_source, span), "\"example.com/app\"");
            let context = take_last_compile_error_context()
                .expect("import cycle should record compile error context");
            assert_eq!(context.file_path, "lib/lib.go");
            assert_eq!(
                &lib_source[context.span_start..context.span_end],
                "\"example.com/app\""
            );
        }
        other => panic!("expected import cycle error, got {other:?}"),
    }
}

#[test]
fn detects_multi_package_local_import_cycles() {
    let third_source = r#"
package third

import "example.com/app/first"
"#;

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/app/first"

func main() {}
"#,
            },
            SourceInput {
                path: "first/first.go",
                source: r#"
package first

import "example.com/app/second"
"#,
            },
            SourceInput {
                path: "second/second.go",
                source: r#"
package second

import "example.com/app/third"
"#,
            },
            SourceInput {
                path: "third/third.go",
                source: third_source,
            },
        ],
        "main.go",
    )
    .expect_err("expected import cycle to fail");

    match error {
        CompileError::ImportCycle {
            cycle,
            source_path,
            span,
            ..
        } => {
            assert_eq!(
                cycle,
                vec![
                    "example.com/app/first".to_string(),
                    "example.com/app/second".to_string(),
                    "example.com/app/third".to_string(),
                    "example.com/app/first".to_string(),
                ]
            );
            assert_eq!(source_path, "third/third.go");
            assert_eq!(span_text(third_source, span), "\"example.com/app/first\"");
        }
        other => panic!("expected import cycle error, got {other:?}"),
    }
}

#[test]
fn detects_module_cache_import_cycles() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let first_path = module_cache_source_path("example.com/remote", "v1.2.3", "first/first.go");
    let second_path = module_cache_source_path("example.com/remote", "v1.2.3", "second/second.go");
    let second_source = r#"
package second

import "example.com/remote/first"
"#;

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/remote/first"

func main() {}
"#,
            },
            SourceInput {
                path: module_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: first_path.as_str(),
                source: r#"
package first

import "example.com/remote/second"
"#,
            },
            SourceInput {
                path: second_path.as_str(),
                source: second_source,
            },
        ],
        "main.go",
    )
    .expect_err("expected import cycle to fail");

    match error {
        CompileError::ImportCycle {
            cycle,
            source_path,
            span,
            ..
        } => {
            assert_eq!(
                cycle,
                vec![
                    "example.com/remote/first".to_string(),
                    "example.com/remote/second".to_string(),
                    "example.com/remote/first".to_string(),
                ]
            );
            assert_eq!(source_path, second_path);
            assert_eq!(
                span_text(second_source, span),
                "\"example.com/remote/first\""
            );
        }
        other => panic!("expected import cycle error, got {other:?}"),
    }
}

#[test]
fn rejects_reachable_remote_module_without_go_mod() {
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/remote/greeter"

func main() {}
"#,
            },
            SourceInput {
                path: module_go_file_path.as_str(),
                source: r#"
package greeter

func Message() string {
    return "hi"
}
"#,
            },
        ],
        "main.go",
    )
    .expect_err("expected missing remote go.mod to fail");

    match error {
        CompileError::InvalidModuleRoot {
            source_path,
            detail,
        } => {
            assert_eq!(
                source_path,
                module_cache_source_path("example.com/remote", "v1.2.3", "go.mod")
            );
            assert!(detail.contains("missing `go.mod`"));
            let context = take_last_compile_error_context()
                .expect("missing reachable module go.mod should record import-site context");
            assert_eq!(context.file_path, "main.go");
        }
        other => panic!("expected invalid module root error, got {other:?}"),
    }
}

#[test]
fn rejects_remote_module_root_declaration_mismatch() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/remote/greeter"

func main() {}
"#,
            },
            SourceInput {
                path: module_go_mod_path.as_str(),
                source: "module example.com/not-remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: module_go_file_path.as_str(),
                source: r#"
package greeter

func Message() string {
    return "hi"
}
"#,
            },
        ],
        "main.go",
    )
    .expect_err("expected mismatched remote module root to fail");

    match error {
        CompileError::InvalidModuleRoot {
            source_path,
            detail,
        } => {
            assert_eq!(source_path, module_go_mod_path);
            assert!(detail.contains("example.com/not-remote"));
            assert!(detail.contains("example.com/remote"));
        }
        other => panic!("expected invalid module root error, got {other:?}"),
    }
}

#[test]
fn rejects_reachable_conflicting_remote_module_versions() {
    let first_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let first_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "first/first.go");
    let second_go_mod_path = module_cache_source_path("example.com/remote", "v1.3.0", "go.mod");
    let second_go_file_path =
        module_cache_source_path("example.com/remote", "v1.3.0", "second/second.go");

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

import "example.com/remote/first"

func main() {}
"#,
            },
            SourceInput {
                path: first_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: first_go_file_path.as_str(),
                source: r#"
package first

import "example.com/remote/second"
"#,
            },
            SourceInput {
                path: second_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: second_go_file_path.as_str(),
                source: r#"
package second
"#,
            },
        ],
        "main.go",
    )
    .expect_err("expected conflicting remote module versions to fail");

    match error {
        CompileError::ConflictingModuleVersions {
            module_path,
            versions,
        } => {
            assert_eq!(module_path, "example.com/remote");
            assert_eq!(versions, vec!["v1.2.3".to_string(), "v1.3.0".to_string()]);
            let context = take_last_compile_error_context()
                .expect("conflicting module versions should record compile error context");
            assert_eq!(context.file_path, first_go_file_path);
        }
        other => panic!("expected conflicting module versions error, got {other:?}"),
    }
}

#[test]
fn rejects_unsupported_workspace_go_mod_features_with_source_context() {
    let go_mod_source = r#"module example.com/app

go 1.21

require example.com/remote v1.2.3
"#;

    let error = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: go_mod_source,
            },
            SourceInput {
                path: "main.go",
                source: "package main\n\nfunc main() {}\n",
            },
        ],
        "main.go",
    )
    .expect_err("unsupported go.mod directives should fail");

    match error {
        CompileError::UnsupportedModuleFeature {
            source_path,
            feature,
        } => {
            assert_eq!(source_path, "go.mod");
            assert_eq!(feature, "require");
            let context = take_last_compile_error_context()
                .expect("unsupported module feature should record compile error context");
            assert_eq!(context.file_path, "go.mod");
            assert_eq!(
                &go_mod_source[context.span_start..context.span_end],
                "require example.com/remote v1.2.3"
            );
        }
        other => panic!("expected unsupported module feature error, got {other:?}"),
    }
}

#[test]
fn ignores_unreachable_conflicting_remote_module_versions() {
    let first_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let first_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "first/first.go");
    let second_go_mod_path = module_cache_source_path("example.com/remote", "v1.3.0", "go.mod");
    let second_go_file_path =
        module_cache_source_path("example.com/remote", "v1.3.0", "second/second.go");

    let program = compile_workspace(
        &[
            SourceInput {
                path: "go.mod",
                source: "module example.com/app\n\ngo 1.21\n",
            },
            SourceInput {
                path: "main.go",
                source: r#"
package main

func main() {}
"#,
            },
            SourceInput {
                path: first_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: first_go_file_path.as_str(),
                source: r#"
package first
"#,
            },
            SourceInput {
                path: second_go_mod_path.as_str(),
                source: "module example.com/remote\n\ngo 1.21\n",
            },
            SourceInput {
                path: second_go_file_path.as_str(),
                source: r#"
package second
"#,
            },
        ],
        "main.go",
    );

    assert!(
        program.is_ok(),
        "expected unreachable remote-module versions to be ignored"
    );
}

fn span_text(source: &str, span: Span) -> &str {
    &source[span.start..span.end]
}
