use std::collections::{BTreeSet, HashMap, HashSet};

use gowasm_lexer::Span;

use crate::{
    import_resolution::{
        module_cache_source_path, module_source_key_for_path, parse_module_cache_source_path,
        parse_module_path, unsupported_module_feature, ImportSiteContext, ModuleSourceKey,
    },
    record_compile_error_context,
    workspace::ParsedFile,
    CompileError, SourceInput,
};

pub(crate) fn validate_module_graph(
    sources: &[SourceInput<'_>],
    package_files: &HashMap<String, Vec<ParsedFile>>,
    import_contexts: &HashMap<String, ImportSiteContext>,
) -> Result<(), CompileError> {
    let reachable_modules = reachable_module_keys(package_files);
    if reachable_modules.is_empty() {
        return Ok(());
    }

    let module_import_contexts = reachable_module_import_contexts(package_files, import_contexts);
    let module_manifests =
        collect_reachable_module_manifests(sources, &reachable_modules, &module_import_contexts)?;
    let mut versions_by_module_path = HashMap::<String, BTreeSet<String>>::new();
    let mut contexts_by_module_path = HashMap::<String, Vec<(String, ImportSiteContext)>>::new();

    for module in reachable_modules {
        if !module_manifests.contains_key(&module) {
            if let Some(context) = module_import_contexts.get(&module) {
                record_compile_error_context(&context.source_path, context.span);
            }
            return Err(CompileError::InvalidModuleRoot {
                source_path: module_cache_source_path(
                    &module.module_path,
                    &module.version,
                    "go.mod",
                ),
                detail: format!(
                    "cached module `{}`@`{}` is missing `go.mod`",
                    module.module_path, module.version
                ),
            });
        }
        versions_by_module_path
            .entry(module.module_path.clone())
            .or_default()
            .insert(module.version.clone());
        if let Some(context) = module_import_contexts.get(&module).cloned() {
            contexts_by_module_path
                .entry(module.module_path.clone())
                .or_default()
                .push((module.version.clone(), context));
        }
    }

    for (module_path, versions) in versions_by_module_path {
        if versions.len() > 1 {
            if let Some(contexts) = contexts_by_module_path.get(&module_path) {
                if let Some((_, context)) =
                    contexts.iter().max_by(|left, right| left.0.cmp(&right.0))
                {
                    record_compile_error_context(&context.source_path, context.span);
                }
            }
            return Err(CompileError::ConflictingModuleVersions {
                module_path,
                versions: versions.into_iter().collect(),
            });
        }
    }

    Ok(())
}

fn reachable_module_keys(
    package_files: &HashMap<String, Vec<ParsedFile>>,
) -> HashSet<ModuleSourceKey> {
    let mut modules = HashSet::new();
    for files in package_files.values() {
        for parsed in files {
            if let Some(module) = module_source_key_for_path(&parsed.path) {
                modules.insert(module);
            }
        }
    }
    modules
}

fn collect_reachable_module_manifests(
    sources: &[SourceInput<'_>],
    reachable_modules: &HashSet<ModuleSourceKey>,
    module_import_contexts: &HashMap<ModuleSourceKey, ImportSiteContext>,
) -> Result<HashMap<ModuleSourceKey, String>, CompileError> {
    let mut manifests = HashMap::new();

    for source in sources {
        let Some(module_cache_path) = parse_module_cache_source_path(source.path) else {
            continue;
        };
        if module_cache_path.relative_path != "go.mod" {
            continue;
        }

        let module_key =
            ModuleSourceKey::new(&module_cache_path.module_path, &module_cache_path.version);
        if !reachable_modules.contains(&module_key) {
            continue;
        }

        if let Some((feature, span)) = unsupported_module_feature(source.source) {
            record_compile_error_context(source.path, span);
            return Err(CompileError::UnsupportedModuleFeature {
                source_path: source.path.into(),
                feature,
            });
        }

        let Some(declared_module_path) = parse_module_path(source.source) else {
            record_module_root_error_context(source.path, source.source);
            return Err(CompileError::InvalidModuleRoot {
                source_path: source.path.into(),
                detail: format!(
                    "cached module `{}`@`{}` is missing a `module` declaration",
                    module_key.module_path, module_key.version
                ),
            });
        };

        if declared_module_path != module_cache_path.module_path {
            record_module_root_error_context(source.path, source.source);
            return Err(CompileError::InvalidModuleRoot {
                source_path: source.path.into(),
                detail: format!(
                    "declares module `{declared_module_path}` but is loaded under `{}`",
                    module_cache_path.module_path
                ),
            });
        }

        manifests.insert(module_key, declared_module_path);
    }

    for module in reachable_modules {
        if manifests.contains_key(module) {
            continue;
        }
        if let Some(context) = module_import_contexts.get(module) {
            record_compile_error_context(&context.source_path, context.span);
        }
    }

    Ok(manifests)
}

fn reachable_module_import_contexts(
    package_files: &HashMap<String, Vec<ParsedFile>>,
    import_contexts: &HashMap<String, ImportSiteContext>,
) -> HashMap<ModuleSourceKey, ImportSiteContext> {
    let mut contexts = HashMap::new();

    for (import_path, files) in package_files {
        let Some(module) = files
            .iter()
            .find_map(|parsed| module_source_key_for_path(&parsed.path))
        else {
            continue;
        };
        let Some(context) = import_contexts.get(import_path) else {
            continue;
        };
        contexts.entry(module).or_insert_with(|| context.clone());
    }

    contexts
}

fn record_module_root_error_context(path: &str, source: &str) {
    if let Some(span) = module_declaration_span(source) {
        record_compile_error_context(path, span);
        return;
    }

    let end = source.find('\n').unwrap_or(source.len());
    record_compile_error_context(path, Span { start: 0, end });
}

fn module_declaration_span(source: &str) -> Option<Span> {
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let content = line.trim_end_matches('\n');
        if content.trim_start().starts_with("module ") {
            return Some(Span {
                start: offset,
                end: offset + content.len(),
            });
        }
        offset += line.len();
    }

    None
}
