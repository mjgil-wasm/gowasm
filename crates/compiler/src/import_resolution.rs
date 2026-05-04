use std::collections::{HashMap, HashSet, VecDeque};

use gowasm_lexer::Span;
use gowasm_vm::stdlib_packages;

use crate::{
    module_graph_validation::validate_module_graph,
    record_compile_error_context,
    types::is_imported_type_only_package,
    workspace::{parse_workspace_files, ParsedFile},
    CompileError, SourceInput,
};

pub const MODULE_CACHE_SOURCE_PREFIX: &str = "__module_cache__";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleSourceKey {
    pub module_path: String,
    pub version: String,
}

impl ModuleSourceKey {
    pub fn new(module_path: &str, version: &str) -> Self {
        Self {
            module_path: module_path.into(),
            version: version.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageSourceOrigin {
    Workspace,
    ModuleCache { module: ModuleSourceKey },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportSiteContext {
    pub(crate) source_path: String,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub struct WorkspacePackageNode {
    pub import_path: String,
    pub package_name: String,
    pub source_paths: Vec<String>,
    pub direct_dependencies: Vec<String>,
    pub reverse_dependencies: Vec<String>,
    pub source_origin: PackageSourceOrigin,
}

#[derive(Debug, Clone)]
pub struct WorkspaceRebuildGraph {
    entry_import_path: String,
    dependency_order: Vec<String>,
    packages: HashMap<String, WorkspacePackageNode>,
    source_to_package: HashMap<String, String>,
    source_dir_to_packages: HashMap<String, Vec<String>>,
    module_to_packages: HashMap<ModuleSourceKey, Vec<String>>,
}

impl WorkspaceRebuildGraph {
    pub fn entry_import_path(&self) -> &str {
        &self.entry_import_path
    }

    pub fn dependency_order(&self) -> &[String] {
        &self.dependency_order
    }

    pub fn package(&self, import_path: &str) -> Option<&WorkspacePackageNode> {
        self.packages.get(import_path)
    }

    pub fn package_for_source_path(&self, path: &str) -> Option<&WorkspacePackageNode> {
        let import_path = self.source_to_package.get(path)?;
        self.packages.get(import_path)
    }

    pub fn changed_package_import_paths_for_source_paths<I, S>(&self, paths: I) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.ordered_import_paths(
            paths
                .into_iter()
                .flat_map(|path| {
                    self.changed_package_import_paths_for_source_path(path.as_ref())
                        .into_iter()
                })
                .collect(),
        )
    }

    pub fn changed_package_import_paths_for_modules<I>(&self, modules: I) -> Vec<String>
    where
        I: IntoIterator<Item = ModuleSourceKey>,
    {
        let mut roots = HashSet::new();
        for module in modules {
            if let Some(packages) = self.module_to_packages.get(&module) {
                roots.extend(packages.iter().cloned());
            }
        }
        self.ordered_import_paths(roots)
    }

    pub fn affected_package_import_paths_for_source_paths<I, S>(&self, paths: I) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let roots = paths
            .into_iter()
            .filter_map(|path| self.source_to_package.get(path.as_ref()).cloned())
            .collect::<HashSet<_>>();
        self.affected_package_import_paths_from_roots(roots)
    }

    pub fn affected_package_import_paths_for_modules<I>(&self, modules: I) -> Vec<String>
    where
        I: IntoIterator<Item = ModuleSourceKey>,
    {
        let mut roots = HashSet::new();
        for module in modules {
            if let Some(packages) = self.module_to_packages.get(&module) {
                roots.extend(packages.iter().cloned());
            }
        }
        self.affected_package_import_paths_from_roots(roots)
    }

    fn affected_package_import_paths_from_roots<I>(&self, roots: I) -> Vec<String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut pending = VecDeque::new();
        let mut affected = HashSet::new();

        for import_path in roots {
            if self.packages.contains_key(&import_path) {
                pending.push_back(import_path);
            }
        }

        while let Some(import_path) = pending.pop_front() {
            if !affected.insert(import_path.clone()) {
                continue;
            }
            let Some(package) = self.packages.get(&import_path) else {
                continue;
            };
            for reverse_dependency in &package.reverse_dependencies {
                pending.push_back(reverse_dependency.clone());
            }
        }

        self.ordered_import_paths(affected)
    }

    fn ordered_import_paths(&self, import_paths: HashSet<String>) -> Vec<String> {
        let dependency_positions = self
            .dependency_order
            .iter()
            .enumerate()
            .map(|(index, import_path)| (import_path.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut import_paths = import_paths.into_iter().collect::<Vec<_>>();
        import_paths.sort_by(|left, right| {
            let left_index = dependency_positions.get(left.as_str()).copied();
            let right_index = dependency_positions.get(right.as_str()).copied();
            left_index.cmp(&right_index).then_with(|| left.cmp(right))
        });
        import_paths
    }

    fn changed_package_import_paths_for_source_path(&self, path: &str) -> Vec<String> {
        if let Some(import_path) = self.source_to_package.get(path) {
            return vec![import_path.clone()];
        }
        if !path.ends_with(".go") {
            return Vec::new();
        }
        self.source_dir_to_packages
            .get(parent_dir(path))
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedWorkspace {
    pub(crate) entry_import_path: String,
    pub(crate) entry_package_name: String,
    pub(crate) package_files: HashMap<String, Vec<ParsedFile>>,
    pub(crate) rebuild_graph: WorkspaceRebuildGraph,
}

pub fn module_cache_source_path(module_path: &str, version: &str, relative_path: &str) -> String {
    format!(
        "{MODULE_CACHE_SOURCE_PREFIX}/{module_path}/@{version}/{}",
        relative_path.trim_start_matches('/')
    )
}

pub fn module_source_key_for_path(path: &str) -> Option<ModuleSourceKey> {
    let module_cache_path = parse_module_cache_source_path(path)?;
    Some(ModuleSourceKey::new(
        &module_cache_path.module_path,
        &module_cache_path.version,
    ))
}

pub(crate) fn resolve_workspace_imports(
    sources: &[SourceInput<'_>],
    entry_path: &str,
) -> Result<ResolvedWorkspace, CompileError> {
    validate_workspace_go_mod_features(sources)?;
    let parsed_files = parse_workspace_files(sources)?;
    let entry_file = parsed_files
        .iter()
        .find(|parsed| parsed.path == entry_path)
        .ok_or_else(|| CompileError::MissingEntryFile {
            path: entry_path.into(),
        })?;
    let workspace_module_path = parse_workspace_module_path(sources);
    let all_package_files = collect_package_files(&parsed_files, workspace_module_path.as_deref());
    let entry_import_path = source_import_path(&entry_file.path, workspace_module_path.as_deref());
    let reachable_packages = reachable_package_files(&entry_import_path, &all_package_files)?;
    validate_module_graph(
        sources,
        &reachable_packages.package_files,
        &reachable_packages.import_contexts,
    )?;
    let package_files = reachable_packages.package_files;
    let rebuild_graph = build_rebuild_graph(&entry_import_path, &package_files);

    Ok(ResolvedWorkspace {
        entry_import_path,
        entry_package_name: entry_file.file.package_name.clone(),
        package_files,
        rebuild_graph,
    })
}

pub fn resolve_workspace_rebuild_graph(
    sources: &[SourceInput<'_>],
    entry_path: &str,
) -> Result<WorkspaceRebuildGraph, CompileError> {
    validate_workspace_go_mod_features(sources)?;
    let parsed_files = parse_workspace_files(sources)?;
    let entry_file = parsed_files
        .iter()
        .find(|parsed| parsed.path == entry_path)
        .ok_or_else(|| CompileError::MissingEntryFile {
            path: entry_path.into(),
        })?;
    let workspace_module_path = parse_workspace_module_path(sources);
    let all_package_files = collect_package_files(&parsed_files, workspace_module_path.as_deref());
    let entry_import_path = source_import_path(&entry_file.path, workspace_module_path.as_deref());
    let reachable_packages = reachable_package_files(&entry_import_path, &all_package_files)?;
    validate_module_graph(
        sources,
        &reachable_packages.package_files,
        &reachable_packages.import_contexts,
    )?;
    Ok(build_rebuild_graph(&entry_import_path, &all_package_files))
}

#[derive(Debug, Default)]
struct ReachablePackageFiles {
    package_files: HashMap<String, Vec<ParsedFile>>,
    import_contexts: HashMap<String, ImportSiteContext>,
}

fn reachable_package_files(
    entry_import_path: &str,
    all_package_files: &HashMap<String, Vec<ParsedFile>>,
) -> Result<ReachablePackageFiles, CompileError> {
    let mut reachable = ReachablePackageFiles::default();
    let mut visited = HashSet::new();
    let mut visiting = Vec::new();
    let mut visiting_positions = HashMap::new();

    collect_reachable_package_files(
        entry_import_path,
        entry_import_path,
        all_package_files,
        &mut reachable,
        &mut visited,
        &mut visiting,
        &mut visiting_positions,
    )?;

    Ok(reachable)
}

fn collect_reachable_package_files(
    import_path: &str,
    entry_import_path: &str,
    all_package_files: &HashMap<String, Vec<ParsedFile>>,
    reachable: &mut ReachablePackageFiles,
    visited: &mut HashSet<String>,
    visiting: &mut Vec<String>,
    visiting_positions: &mut HashMap<String, usize>,
) -> Result<(), CompileError> {
    if visited.contains(import_path) {
        return Ok(());
    }

    let files =
        all_package_files
            .get(import_path)
            .ok_or_else(|| CompileError::UnresolvedImportPath {
                importer: entry_import_path.into(),
                path: import_path.into(),
            })?;

    let position = visiting.len();
    visiting.push(import_path.to_string());
    visiting_positions.insert(import_path.to_string(), position);
    reachable
        .package_files
        .insert(import_path.to_string(), files.clone());

    for parsed in files {
        for (index, import) in parsed.file.imports.iter().enumerate() {
            let import_span = parsed.spans.imports.get(index).copied();
            if let Some(detail) = unsupported_import_detail(&import.path) {
                if let Some(span) = import_span {
                    record_compile_error_context(&parsed.path, span);
                }
                return Err(CompileError::Unsupported { detail });
            }
            if is_stdlib_import(&import.path) {
                continue;
            }
            if let Some(cycle_start) = visiting_positions.get(&import.path).copied() {
                return Err(import_cycle_error(
                    parsed,
                    import_span,
                    cycle_path(visiting, cycle_start),
                ));
            }
            if !all_package_files.contains_key(&import.path) {
                if let Some(span) = import_span {
                    record_compile_error_context(&parsed.path, span);
                }
                return Err(CompileError::UnresolvedImportPath {
                    importer: import_path.to_string(),
                    path: import.path.clone(),
                });
            }
            if let Some(span) = import_span {
                reachable
                    .import_contexts
                    .entry(import.path.clone())
                    .or_insert_with(|| ImportSiteContext {
                        source_path: parsed.path.clone(),
                        span,
                    });
            }
            collect_reachable_package_files(
                &import.path,
                import_path,
                all_package_files,
                reachable,
                visited,
                visiting,
                visiting_positions,
            )?;
        }
    }

    visiting.pop();
    visiting_positions.remove(import_path);
    visited.insert(import_path.to_string());
    Ok(())
}

fn import_cycle_error(parsed: &ParsedFile, span: Option<Span>, cycle: Vec<String>) -> CompileError {
    let span = span.unwrap_or(Span { start: 0, end: 0 });
    record_compile_error_context(&parsed.path, span);
    let cycle_path = cycle.join(" -> ");
    CompileError::ImportCycle {
        cycle,
        cycle_path,
        source_path: parsed.path.clone(),
        span,
    }
}

fn cycle_path(visiting: &[String], cycle_start: usize) -> Vec<String> {
    let mut cycle = visiting[cycle_start..].to_vec();
    cycle.push(visiting[cycle_start].clone());
    cycle
}

fn build_rebuild_graph(
    entry_import_path: &str,
    package_files: &HashMap<String, Vec<ParsedFile>>,
) -> WorkspaceRebuildGraph {
    let mut packages = HashMap::new();
    let mut reverse_dependencies = HashMap::<String, HashSet<String>>::new();
    let mut source_to_package = HashMap::new();
    let mut source_dir_to_packages = HashMap::<String, HashSet<String>>::new();
    let mut module_to_packages = HashMap::<ModuleSourceKey, HashSet<String>>::new();

    let mut import_paths = package_files.keys().cloned().collect::<Vec<_>>();
    import_paths.sort();

    for import_path in import_paths {
        let files = package_files
            .get(&import_path)
            .expect("package files should exist for every tracked import path");
        let source_paths = sorted_source_paths(files);
        for path in &source_paths {
            source_to_package.insert(path.clone(), import_path.clone());
            source_dir_to_packages
                .entry(parent_dir(path).to_string())
                .or_default()
                .insert(import_path.clone());
        }

        let direct_dependencies = collect_direct_dependencies(files, package_files);
        for dependency in &direct_dependencies {
            reverse_dependencies
                .entry(dependency.clone())
                .or_default()
                .insert(import_path.clone());
        }

        let source_origin = package_source_origin(files);
        if let PackageSourceOrigin::ModuleCache { module } = &source_origin {
            module_to_packages
                .entry(module.clone())
                .or_default()
                .insert(import_path.clone());
        }

        packages.insert(
            import_path.clone(),
            WorkspacePackageNode {
                import_path: import_path.clone(),
                package_name: files
                    .first()
                    .map(|parsed| parsed.file.package_name.clone())
                    .unwrap_or_default(),
                source_paths,
                direct_dependencies,
                reverse_dependencies: Vec::new(),
                source_origin,
            },
        );
    }

    for (import_path, package) in &mut packages {
        let mut package_reverse_dependencies = reverse_dependencies
            .remove(import_path)
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        package_reverse_dependencies.sort();
        package.reverse_dependencies = package_reverse_dependencies;
    }

    let dependency_order = dependency_order(entry_import_path, &packages);
    let dependency_positions = dependency_order
        .iter()
        .enumerate()
        .map(|(index, import_path)| (import_path.as_str(), index))
        .collect::<HashMap<_, _>>();
    let module_to_packages = module_to_packages
        .into_iter()
        .map(|(module, packages)| {
            let mut packages = packages.into_iter().collect::<Vec<_>>();
            packages.sort_by(|left, right| {
                let left_index = dependency_positions.get(left.as_str()).copied();
                let right_index = dependency_positions.get(right.as_str()).copied();
                left_index.cmp(&right_index).then_with(|| left.cmp(right))
            });
            (module, packages)
        })
        .collect();
    let source_dir_to_packages = source_dir_to_packages
        .into_iter()
        .map(|(dir, packages)| {
            (
                dir,
                dependency_order
                    .iter()
                    .filter(|import_path| packages.contains(import_path.as_str()))
                    .cloned()
                    .collect(),
            )
        })
        .collect();

    WorkspaceRebuildGraph {
        entry_import_path: entry_import_path.into(),
        dependency_order,
        packages,
        source_to_package,
        source_dir_to_packages,
        module_to_packages,
    }
}

fn collect_direct_dependencies(
    files: &[ParsedFile],
    package_files: &HashMap<String, Vec<ParsedFile>>,
) -> Vec<String> {
    let mut direct_dependencies = HashSet::new();
    for parsed in files {
        for import in &parsed.file.imports {
            if is_stdlib_import(&import.path) || !package_files.contains_key(&import.path) {
                continue;
            }
            direct_dependencies.insert(import.path.clone());
        }
    }
    let mut direct_dependencies = direct_dependencies.into_iter().collect::<Vec<_>>();
    direct_dependencies.sort();
    direct_dependencies
}

fn sorted_source_paths(files: &[ParsedFile]) -> Vec<String> {
    let mut source_paths = files
        .iter()
        .map(|parsed| parsed.path.clone())
        .collect::<Vec<_>>();
    source_paths.sort();
    source_paths
}

fn package_source_origin(files: &[ParsedFile]) -> PackageSourceOrigin {
    let Some(module_cache_path) = files
        .iter()
        .find_map(|parsed| parse_module_cache_source_path(&parsed.path))
    else {
        return PackageSourceOrigin::Workspace;
    };

    PackageSourceOrigin::ModuleCache {
        module: ModuleSourceKey::new(&module_cache_path.module_path, &module_cache_path.version),
    }
}

fn dependency_order(
    entry_import_path: &str,
    packages: &HashMap<String, WorkspacePackageNode>,
) -> Vec<String> {
    let mut order = Vec::new();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();

    visit_dependency_order(
        entry_import_path,
        packages,
        &mut visiting,
        &mut visited,
        &mut order,
    );

    let mut remaining = packages.keys().cloned().collect::<Vec<_>>();
    remaining.sort();
    for import_path in remaining {
        visit_dependency_order(
            &import_path,
            packages,
            &mut visiting,
            &mut visited,
            &mut order,
        );
    }

    order
}

fn visit_dependency_order(
    import_path: &str,
    packages: &HashMap<String, WorkspacePackageNode>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    order: &mut Vec<String>,
) {
    if visited.contains(import_path) {
        return;
    }
    if !visiting.insert(import_path.to_string()) {
        return;
    }

    if let Some(package) = packages.get(import_path) {
        for dependency in &package.direct_dependencies {
            visit_dependency_order(dependency, packages, visiting, visited, order);
        }
    }

    visiting.remove(import_path);
    visited.insert(import_path.to_string());
    order.push(import_path.to_string());
}

fn collect_package_files(
    parsed_files: &[ParsedFile],
    workspace_module_path: Option<&str>,
) -> HashMap<String, Vec<ParsedFile>> {
    let mut packages = HashMap::new();
    for parsed in parsed_files {
        let import_path = source_import_path(&parsed.path, workspace_module_path);
        packages
            .entry(import_path)
            .or_insert_with(Vec::new)
            .push(parsed.clone());
    }
    for files in packages.values_mut() {
        files.sort_by(|left, right| left.path.cmp(&right.path));
    }
    packages
}

fn source_import_path(path: &str, workspace_module_path: Option<&str>) -> String {
    if let Some(module_cache) = parse_module_cache_source_path(path) {
        let dir = parent_dir(&module_cache.relative_path);
        return join_import_path(&module_cache.module_path, dir);
    }

    let dir = parent_dir(path);
    match workspace_module_path {
        Some(module_path) => join_import_path(module_path, dir),
        None if dir.is_empty() => ".".into(),
        None => dir.into(),
    }
}

fn join_import_path(prefix: &str, dir: &str) -> String {
    if dir.is_empty() {
        prefix.into()
    } else {
        format!("{prefix}/{dir}")
    }
}

fn parent_dir(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

fn parse_workspace_module_path(sources: &[SourceInput<'_>]) -> Option<String> {
    sources
        .iter()
        .find(|source| source.path == "go.mod")
        .and_then(|source| parse_module_path(source.source))
}

fn validate_workspace_go_mod_features(sources: &[SourceInput<'_>]) -> Result<(), CompileError> {
    let Some(go_mod) = sources.iter().find(|source| source.path == "go.mod") else {
        return Ok(());
    };

    if let Some((feature, span)) = unsupported_module_feature(go_mod.source) {
        record_compile_error_context(go_mod.path, span);
        return Err(CompileError::UnsupportedModuleFeature {
            source_path: go_mod.path.into(),
            feature,
        });
    }

    Ok(())
}

pub(crate) fn parse_module_path(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(module_path) = trimmed.strip_prefix("module ") {
            let module_path = module_path.trim();
            if !module_path.is_empty() {
                return Some(module_path.into());
            }
        }
    }
    None
}

fn is_stdlib_import(path: &str) -> bool {
    stdlib_packages().iter().any(|package| package.name == path)
        || is_imported_type_only_package(path)
}

fn unsupported_import_detail(path: &str) -> Option<String> {
    match path {
        "C" => Some("cgo via `import \"C\"` is outside the supported subset".into()),
        "plugin" | "os/exec" | "unsafe" => {
            Some(format!("package `{path}` is outside the supported subset"))
        }
        _ => None,
    }
}

pub(crate) fn unsupported_module_feature(source: &str) -> Option<(String, Span)> {
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let content = line.trim_end_matches('\n');
        let trimmed = content.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            offset += line.len();
            continue;
        }

        for feature in ["require", "replace", "exclude", "retract", "toolchain"] {
            if let Some(rest) = trimmed.strip_prefix(feature) {
                if rest.is_empty() || rest.starts_with(char::is_whitespace) || rest == "(" {
                    let leading_ws = content.len().saturating_sub(trimmed.len());
                    return Some((
                        feature.to_string(),
                        Span {
                            start: offset + leading_ws,
                            end: offset + content.len(),
                        },
                    ));
                }
            }
        }

        offset += line.len();
    }

    None
}

#[derive(Debug)]
pub(crate) struct ModuleCacheSourcePath {
    pub(crate) module_path: String,
    pub(crate) version: String,
    pub(crate) relative_path: String,
}

pub(crate) fn parse_module_cache_source_path(path: &str) -> Option<ModuleCacheSourcePath> {
    if !path.starts_with(MODULE_CACHE_SOURCE_PREFIX) {
        return None;
    }

    let mut segments = path.split('/');
    if segments.next()? != MODULE_CACHE_SOURCE_PREFIX {
        return None;
    }

    let mut module_segments = Vec::new();
    let mut version = None;
    let mut relative_segments = Vec::new();

    for segment in segments {
        if version.is_none() {
            if segment.starts_with('@') {
                version = Some(segment.trim_start_matches('@').to_string());
                continue;
            }
            module_segments.push(segment);
        } else {
            relative_segments.push(segment);
        }
    }

    let version = version?;

    if module_segments.is_empty() || relative_segments.is_empty() {
        return None;
    }

    Some(ModuleCacheSourcePath {
        module_path: module_segments.join("/"),
        version,
        relative_path: relative_segments.join("/"),
    })
}
