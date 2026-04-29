use std::collections::{HashMap, VecDeque};

use gowasm_host_types::{
    EngineResponse, ErrorCategory, ModuleCacheFillRequest, ModuleCacheKey,
    ModuleCacheLookupRequest, ModuleCacheLookupResult, ModuleFetchRequest, ModuleFetchResult,
    ModuleGraphRoot, ModuleRequest, ModuleResult, ModuleSourceBundle,
};

pub(super) struct PausedModuleLoad {
    active_root: ModuleGraphRoot,
    pending_request: ModuleRequest,
    remaining: VecDeque<ModuleGraphRoot>,
    loaded: Vec<ModuleSourceBundle>,
    fetched_module: Option<ModuleSourceBundle>,
}

impl PausedModuleLoad {
    fn new(first: ModuleGraphRoot, remaining: VecDeque<ModuleGraphRoot>) -> Self {
        let pending_request = ModuleRequest::CacheLookup {
            request: ModuleCacheLookupRequest {
                module: module_cache_key(&first),
            },
        };
        Self {
            active_root: first,
            pending_request,
            remaining,
            loaded: Vec::new(),
            fetched_module: None,
        }
    }
}

pub(super) fn start_module_graph_load(
    next_request_id: &mut u64,
    paused_module_loads: &mut HashMap<u64, PausedModuleLoad>,
    modules: Vec<ModuleGraphRoot>,
) -> EngineResponse {
    let mut modules = VecDeque::from(modules);
    let Some(first) = modules.pop_front() else {
        return EngineResponse::ModuleGraphResult {
            modules: Vec::new(),
        };
    };

    let request_id = *next_request_id;
    *next_request_id = next_request_id.saturating_add(1);
    let paused = PausedModuleLoad::new(first, modules);
    let pending_request = paused.pending_request.clone();
    paused_module_loads.insert(request_id, paused);
    EngineResponse::ModuleRequest {
        request_id,
        module: pending_request,
    }
}

pub(super) fn resume_module_graph_load(
    paused_module_loads: &mut HashMap<u64, PausedModuleLoad>,
    request_id: u64,
    module: ModuleResult,
) -> EngineResponse {
    let Some(paused) = paused_module_loads.get_mut(&request_id) else {
        return EngineResponse::Fatal {
            message: format!("module load resumed for unknown request `{request_id}`"),
            category: ErrorCategory::ProtocolError,
        };
    };

    match (&paused.pending_request, module) {
        (ModuleRequest::CacheLookup { request }, ModuleResult::CacheLookup { result }) => {
            match result {
                ModuleCacheLookupResult::Hit { module } => {
                    if let Err(message) = validate_module_bundle(&request.module, &module) {
                        paused_module_loads.remove(&request_id);
                        return EngineResponse::Fatal {
                            message,
                            category: ErrorCategory::HostError,
                        };
                    }
                    paused.loaded.push(module);
                    advance_module_graph_load(paused_module_loads, request_id)
                }
                ModuleCacheLookupResult::Miss => {
                    paused.pending_request = ModuleRequest::Fetch {
                        request: ModuleFetchRequest {
                            module: module_cache_key(&paused.active_root),
                            fetch_url: paused.active_root.fetch_url.clone(),
                        },
                    };
                    EngineResponse::ModuleRequest {
                        request_id,
                        module: paused.pending_request.clone(),
                    }
                }
            }
        }
        (ModuleRequest::Fetch { request }, ModuleResult::Fetch { result }) => match result {
            ModuleFetchResult::Module { module } => {
                if let Err(message) = validate_module_bundle(&request.module, &module) {
                    paused_module_loads.remove(&request_id);
                    return EngineResponse::Fatal {
                        message,
                        category: ErrorCategory::HostError,
                    };
                }
                paused.fetched_module = Some(module.clone());
                paused.pending_request = ModuleRequest::CacheFill {
                    request: ModuleCacheFillRequest { module },
                };
                EngineResponse::ModuleRequest {
                    request_id,
                    module: paused.pending_request.clone(),
                }
            }
            ModuleFetchResult::Error { message } => {
                let module_path = request.module.module_path.clone();
                let version = request.module.version.clone();
                paused_module_loads.remove(&request_id);
                EngineResponse::Fatal {
                    message: format!("module fetch failed for {module_path}@{version}: {message}"),
                    category: ErrorCategory::HostError,
                }
            }
        },
        (ModuleRequest::CacheFill { .. }, ModuleResult::CacheFill) => {
            let Some(fetched_module) = paused.fetched_module.take() else {
                paused_module_loads.remove(&request_id);
                return EngineResponse::Fatal {
                    message: format!(
                        "module request `{request_id}` reached cache fill without a fetched bundle"
                    ),
                    category: ErrorCategory::ProtocolError,
                };
            };
            paused.loaded.push(fetched_module);
            advance_module_graph_load(paused_module_loads, request_id)
        }
        (expected, actual) => {
            let expected_name = module_request_name(expected);
            let actual_name = module_result_name(&actual);
            paused_module_loads.remove(&request_id);
            EngineResponse::Fatal {
                message: format!(
                    "module request `{request_id}` resumed with result `{actual_name}` while waiting for `{expected_name}`"
                ),
                category: ErrorCategory::ProtocolError,
            }
        }
    }
}

fn advance_module_graph_load(
    paused_module_loads: &mut HashMap<u64, PausedModuleLoad>,
    request_id: u64,
) -> EngineResponse {
    let Some(paused) = paused_module_loads.get_mut(&request_id) else {
        return EngineResponse::Fatal {
            message: format!("module request `{request_id}` disappeared during advancement"),
            category: ErrorCategory::ProtocolError,
        };
    };
    let Some(next_root) = paused.remaining.pop_front() else {
        let paused = paused_module_loads
            .remove(&request_id)
            .expect("paused module load should exist");
        return EngineResponse::ModuleGraphResult {
            modules: paused.loaded,
        };
    };

    paused.active_root = next_root;
    paused.pending_request = ModuleRequest::CacheLookup {
        request: ModuleCacheLookupRequest {
            module: module_cache_key(&paused.active_root),
        },
    };
    EngineResponse::ModuleRequest {
        request_id,
        module: paused.pending_request.clone(),
    }
}

fn module_cache_key(root: &ModuleGraphRoot) -> ModuleCacheKey {
    ModuleCacheKey {
        module_path: root.module_path.clone(),
        version: root.version.clone(),
    }
}

fn validate_module_bundle(
    expected: &ModuleCacheKey,
    module: &ModuleSourceBundle,
) -> Result<(), String> {
    if module.module != *expected {
        return Err(format!(
            "module bundle identity mismatch: expected {}@{} but got {}@{}",
            expected.module_path,
            expected.version,
            module.module.module_path,
            module.module.version
        ));
    }
    Ok(())
}

fn module_request_name(request: &ModuleRequest) -> &'static str {
    match request {
        ModuleRequest::CacheLookup { .. } => "cache_lookup",
        ModuleRequest::Fetch { .. } => "fetch",
        ModuleRequest::CacheFill { .. } => "cache_fill",
    }
}

fn module_result_name(result: &ModuleResult) -> &'static str {
    match result {
        ModuleResult::CacheLookup { .. } => "cache_lookup",
        ModuleResult::Fetch { .. } => "fetch",
        ModuleResult::CacheFill => "cache_fill",
    }
}
