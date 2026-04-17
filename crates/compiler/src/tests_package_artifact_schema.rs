use super::{compile_workspace_with_graph, SourceInput};
use crate::workspace_artifacts::package_artifact_schema::{
    ArtifactSourceOriginSchema, SerializedPackageArtifact, PACKAGE_ARTIFACT_SCHEMA_VERSION,
};

fn generic_workspace_sources() -> Vec<SourceInput<'static>> {
    vec![
        SourceInput {
            path: "main.go",
            source: r#"
package main

import "helpers"

func main() {
    value := helpers.Identity(7)
    box := helpers.Box[int]{Value: value}
    _, _ = value, box
}
"#,
        },
        SourceInput {
            path: "helpers/helpers.go",
            source: r#"
package helpers

import "fmt"

type Box[T any] struct {
    Value T
}

func Identity[T any](value T) T {
    _ = fmt.Sprint(value)
    return value
}
"#,
        },
    ]
}

#[test]
fn serialized_package_artifact_round_trips_for_generic_workspace_package() {
    let sources = generic_workspace_sources();
    let compiled =
        compile_workspace_with_graph(&sources, "main.go").expect("workspace should compile");
    let schema = compiled
        .package_artifacts
        .iter()
        .map(|artifact| artifact.serialized_schema())
        .find(|artifact| artifact.import_path == "helpers")
        .expect("helpers package artifact should exist");

    assert_eq!(schema.schema_version, PACKAGE_ARTIFACT_SCHEMA_VERSION);
    assert!(schema.dependencies.direct_dependencies.is_empty());
    assert_eq!(
        schema.dependencies.reverse_dependencies,
        vec![".".to_string()]
    );
    assert!(matches!(
        schema.dependencies.source_origin,
        Some(ArtifactSourceOriginSchema::Workspace)
    ));
    let generics = schema
        .generics
        .as_ref()
        .expect("generic helper package should serialize generic metadata");
    assert!(generics
        .generic_function_templates
        .get("Identity")
        .expect("generic function template should be present")
        .source_text
        .contains("func Identity[T any](value T) T"));

    let json = serde_json::to_string(&schema).expect("artifact schema should serialize");
    let decoded: SerializedPackageArtifact =
        serde_json::from_str(&json).expect("artifact schema should deserialize");
    assert_eq!(decoded, schema);
}

#[test]
fn serialized_package_artifact_fixture_v1_remains_compatible() {
    let fixture = include_str!("../testdata/package_artifact_schema_v1.json");
    let decoded: SerializedPackageArtifact =
        serde_json::from_str(fixture).expect("fixture should deserialize");
    assert_eq!(decoded.schema_version, PACKAGE_ARTIFACT_SCHEMA_VERSION);
    assert_eq!(decoded.import_path, "example/helpers");
    assert!(matches!(
        decoded.dependencies.source_origin,
        Some(ArtifactSourceOriginSchema::Workspace)
    ));
    assert_eq!(decoded.runtime_metadata.function_start, 0);
    assert!(decoded.generics.is_none());
}
