use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArtifactSourceOrigin {
    Workspace,
    ModuleCache {
        module_path: String,
        version: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ArtifactDependencyEdges {
    pub(crate) source_paths: Vec<String>,
    pub(crate) direct_dependencies: Vec<String>,
    pub(crate) reverse_dependencies: Vec<String>,
    pub(crate) source_origin: Option<ArtifactSourceOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactGenericTemplateSource {
    pub(crate) source_path: String,
    pub(crate) source_text: String,
    pub(crate) spans: FunctionSourceSpans,
    pub(crate) imported_packages: HashMap<String, String>,
}

impl CompiledPackageArtifact {
    pub(crate) fn attach_dependency_edges(&mut self, package: &WorkspacePackageNode) {
        self.dependency_edges = ArtifactDependencyEdges {
            source_paths: package.source_paths.clone(),
            direct_dependencies: package.direct_dependencies.clone(),
            reverse_dependencies: package.reverse_dependencies.clone(),
            source_origin: Some(match &package.source_origin {
                PackageSourceOrigin::Workspace => ArtifactSourceOrigin::Workspace,
                PackageSourceOrigin::ModuleCache { module } => ArtifactSourceOrigin::ModuleCache {
                    module_path: module.module_path.clone(),
                    version: module.version.clone(),
                },
            }),
        };
    }

    pub(crate) fn capture_generic_template_sources(
        &mut self,
        source_lookup: &HashMap<&str, &str>,
    ) -> Result<(), CompileError> {
        let Some(context) = self.generic_package_context.as_ref() else {
            return Ok(());
        };
        self.generic_function_template_sources = context
            .generic_function_templates
            .iter()
            .map(|(name, template)| {
                Ok((
                    name.clone(),
                    template_source_from_parts(
                        &template.source_path,
                        &template.spans,
                        &template.imported_packages,
                        source_lookup,
                    )?,
                ))
            })
            .collect::<Result<HashMap<_, _>, CompileError>>()?;
        self.generic_method_template_sources = context
            .generic_method_templates
            .iter()
            .map(|(name, templates)| {
                Ok((
                    name.clone(),
                    templates
                        .iter()
                        .map(|template| {
                            template_source_from_parts(
                                &template.source_path,
                                &template.spans,
                                &template.imported_packages,
                                source_lookup,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            })
            .collect::<Result<HashMap<_, _>, CompileError>>()?;
        Ok(())
    }
}

fn template_source_from_parts(
    source_path: &str,
    spans: &FunctionSourceSpans,
    imported_packages: &HashMap<String, String>,
    source_lookup: &HashMap<&str, &str>,
) -> Result<ArtifactGenericTemplateSource, CompileError> {
    let source =
        source_lookup
            .get(source_path)
            .copied()
            .ok_or_else(|| CompileError::Unsupported {
                detail: format!("missing source text for generic template `{source_path}`"),
            })?;
    let source_text = source
        .get(spans.span.start..spans.span.end)
        .ok_or_else(|| CompileError::Unsupported {
            detail: format!("invalid generic template span in `{source_path}`"),
        })?
        .to_string();
    Ok(ArtifactGenericTemplateSource {
        source_path: source_path.to_string(),
        source_text,
        spans: spans.clone(),
        imported_packages: imported_packages.clone(),
    })
}
