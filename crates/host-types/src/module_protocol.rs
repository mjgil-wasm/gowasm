use serde::{Deserialize, Serialize};

use crate::WorkspaceFile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleGraphRoot {
    pub module_path: String,
    pub version: String,
    pub fetch_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleCacheKey {
    pub module_path: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleSourceBundle {
    pub module: ModuleCacheKey,
    pub origin_url: String,
    pub files: Vec<WorkspaceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleCacheLookupRequest {
    pub module: ModuleCacheKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModuleCacheLookupResult {
    Hit { module: ModuleSourceBundle },
    Miss,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleFetchRequest {
    pub module: ModuleCacheKey,
    pub fetch_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModuleFetchResult {
    Module { module: ModuleSourceBundle },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleCacheFillRequest {
    pub module: ModuleSourceBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModuleRequest {
    CacheLookup { request: ModuleCacheLookupRequest },
    Fetch { request: ModuleFetchRequest },
    CacheFill { request: ModuleCacheFillRequest },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModuleResult {
    CacheLookup { result: ModuleCacheLookupResult },
    Fetch { result: ModuleFetchResult },
    CacheFill,
}
