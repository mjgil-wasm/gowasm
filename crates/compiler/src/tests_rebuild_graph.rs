use crate::{
    module_cache_source_path, resolve_workspace_rebuild_graph, ModuleSourceKey,
    PackageSourceOrigin, SourceInput,
};

#[test]
fn rebuild_graph_tracks_local_dependencies_and_reverse_dependents() {
    let graph = resolve_workspace_rebuild_graph(
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

import "example.com/app/shared"

func Message() string {
    return shared.Message()
}
"#,
            },
            SourceInput {
                path: "shared/shared.go",
                source: r#"
package shared

func Message() string {
    return "hello"
}
"#,
            },
        ],
        "main.go",
    )
    .expect("expected rebuild graph to resolve local packages");

    assert_eq!(graph.entry_import_path(), "example.com/app");
    assert_eq!(
        graph.dependency_order(),
        &[
            "example.com/app/shared".to_string(),
            "example.com/app/lib".to_string(),
            "example.com/app".to_string(),
        ]
    );

    let lib_package = graph
        .package("example.com/app/lib")
        .expect("expected lib package");
    assert_eq!(lib_package.package_name, "lib");
    assert_eq!(lib_package.source_paths, vec!["lib/lib.go".to_string()]);
    assert_eq!(
        lib_package.direct_dependencies,
        vec!["example.com/app/shared".to_string()]
    );
    assert_eq!(
        lib_package.reverse_dependencies,
        vec!["example.com/app".to_string()]
    );
    assert_eq!(lib_package.source_origin, PackageSourceOrigin::Workspace);

    let shared_package = graph
        .package_for_source_path("shared/shared.go")
        .expect("expected source path to map to shared package");
    assert_eq!(shared_package.import_path, "example.com/app/shared");
    assert_eq!(
        shared_package.reverse_dependencies,
        vec!["example.com/app/lib".to_string()]
    );
    assert_eq!(
        graph.affected_package_import_paths_for_source_paths(["shared/shared.go"]),
        vec![
            "example.com/app/shared".to_string(),
            "example.com/app/lib".to_string(),
            "example.com/app".to_string(),
        ]
    );
    assert_eq!(
        graph.changed_package_import_paths_for_source_paths(["shared/shared.go"]),
        vec!["example.com/app/shared".to_string()]
    );
}

#[test]
fn rebuild_graph_maps_module_edits_through_workspace_dependents() {
    let module_go_mod_path = module_cache_source_path("example.com/remote", "v1.2.3", "go.mod");
    let module_go_file_path =
        module_cache_source_path("example.com/remote", "v1.2.3", "greeter/greeter.go");
    let graph = resolve_workspace_rebuild_graph(
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

import "example.com/remote/greeter"

func Message() string {
    return greeter.Message()
}
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
    )
    .expect("expected rebuild graph to resolve remote module package");

    let remote_package = graph
        .package("example.com/remote/greeter")
        .expect("expected remote greeter package");
    assert_eq!(
        remote_package.source_origin,
        PackageSourceOrigin::ModuleCache {
            module: ModuleSourceKey::new("example.com/remote", "v1.2.3"),
        }
    );
    assert_eq!(
        graph.affected_package_import_paths_for_modules(vec![ModuleSourceKey::new(
            "example.com/remote",
            "v1.2.3",
        )]),
        vec![
            "example.com/remote/greeter".to_string(),
            "example.com/app/lib".to_string(),
            "example.com/app".to_string(),
        ]
    );
    assert_eq!(
        graph.changed_package_import_paths_for_modules(vec![ModuleSourceKey::new(
            "example.com/remote",
            "v1.2.3",
        )]),
        vec!["example.com/remote/greeter".to_string()]
    );
}

#[test]
fn rebuild_graph_ignores_unknown_edit_keys() {
    let graph = resolve_workspace_rebuild_graph(
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
        ],
        "main.go",
    )
    .expect("expected rebuild graph to resolve single-package workspace");

    assert!(graph
        .affected_package_import_paths_for_source_paths(["missing.go"])
        .is_empty());
    assert!(graph
        .affected_package_import_paths_for_modules(vec![ModuleSourceKey::new(
            "example.com/missing",
            "v0.0.1",
        )])
        .is_empty());
}

#[test]
fn rebuild_graph_maps_new_go_files_in_existing_package_directories() {
    let graph = resolve_workspace_rebuild_graph(
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
                path: "lib/lib.go",
                source: r#"
package lib

func Message() string {
    return "hi"
}
"#,
            },
        ],
        "main.go",
    )
    .expect("expected rebuild graph to resolve multi-package workspace");

    assert_eq!(
        graph.changed_package_import_paths_for_source_paths(["extra.go"]),
        vec!["example.com/app".to_string()]
    );
    assert_eq!(
        graph.changed_package_import_paths_for_source_paths(["lib/extra.go"]),
        vec!["example.com/app/lib".to_string()]
    );
}
