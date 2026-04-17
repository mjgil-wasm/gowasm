use gowasm_host_types::{
    EngineRequest, EngineResponse, ErrorCategory, ModuleCacheKey, ModuleCacheLookupResult,
    ModuleFetchResult, ModuleGraphRoot, ModuleResult, ModuleSourceBundle, WorkspaceFile,
};

use super::Engine;

fn sample_root(module_path: &str, version: &str, fetch_url: &str) -> ModuleGraphRoot {
    ModuleGraphRoot {
        module_path: module_path.into(),
        version: version.into(),
        fetch_url: fetch_url.into(),
    }
}

fn sample_bundle(module_path: &str, version: &str, fetch_url: &str) -> ModuleSourceBundle {
    ModuleSourceBundle {
        module: ModuleCacheKey {
            module_path: module_path.into(),
            version: version.into(),
        },
        origin_url: fetch_url.into(),
        files: vec![
            WorkspaceFile {
                path: "go.mod".into(),
                contents: format!("module {module_path}\n\ngo 1.21\n"),
            },
            WorkspaceFile {
                path: "pkg/file.go".into(),
                contents: "package pkg\n".into(),
            },
        ],
    }
}

#[test]
fn module_load_negotiates_cache_miss_fetch_and_fill() {
    let mut engine = Engine::new();
    let root = sample_root("example.com/hello", "v1.2.3", "data:application/json,hello");

    let response = engine.handle_request(EngineRequest::LoadModuleGraph {
        modules: vec![root.clone()],
    });
    let request_id = match response {
        EngineResponse::ModuleRequest { request_id, module } => {
            assert_eq!(
                module,
                gowasm_host_types::ModuleRequest::CacheLookup {
                    request: gowasm_host_types::ModuleCacheLookupRequest {
                        module: ModuleCacheKey {
                            module_path: root.module_path.clone(),
                            version: root.version.clone(),
                        },
                    },
                }
            );
            request_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::CacheLookup {
            result: ModuleCacheLookupResult::Miss,
        },
    });
    match response {
        EngineResponse::ModuleRequest {
            request_id: resumed,
            module,
        } => {
            assert_eq!(resumed, request_id);
            assert_eq!(
                module,
                gowasm_host_types::ModuleRequest::Fetch {
                    request: gowasm_host_types::ModuleFetchRequest {
                        module: ModuleCacheKey {
                            module_path: root.module_path.clone(),
                            version: root.version.clone(),
                        },
                        fetch_url: root.fetch_url.clone(),
                    },
                }
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let bundle = sample_bundle(&root.module_path, &root.version, &root.fetch_url);
    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::Fetch {
            result: ModuleFetchResult::Module {
                module: bundle.clone(),
            },
        },
    });
    match response {
        EngineResponse::ModuleRequest {
            request_id: resumed,
            module,
        } => {
            assert_eq!(resumed, request_id);
            assert_eq!(
                module,
                gowasm_host_types::ModuleRequest::CacheFill {
                    request: gowasm_host_types::ModuleCacheFillRequest {
                        module: bundle.clone(),
                    },
                }
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::CacheFill,
    });
    assert_eq!(
        response,
        EngineResponse::ModuleGraphResult {
            modules: vec![bundle],
        }
    );
}

#[test]
fn module_load_advances_across_multiple_roots_after_cache_hits() {
    let mut engine = Engine::new();
    let first = sample_root("example.com/one", "v1.0.0", "data:application/json,one");
    let second = sample_root("example.com/two", "v1.0.0", "data:application/json,two");

    let response = engine.handle_request(EngineRequest::LoadModuleGraph {
        modules: vec![first.clone(), second.clone()],
    });
    let request_id = match response {
        EngineResponse::ModuleRequest { request_id, .. } => request_id,
        other => panic!("unexpected response: {other:?}"),
    };

    let first_bundle = sample_bundle(&first.module_path, &first.version, &first.fetch_url);
    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::CacheLookup {
            result: ModuleCacheLookupResult::Hit {
                module: first_bundle.clone(),
            },
        },
    });
    match response {
        EngineResponse::ModuleRequest {
            request_id: resumed,
            module,
        } => {
            assert_eq!(resumed, request_id);
            assert_eq!(
                module,
                gowasm_host_types::ModuleRequest::CacheLookup {
                    request: gowasm_host_types::ModuleCacheLookupRequest {
                        module: ModuleCacheKey {
                            module_path: second.module_path.clone(),
                            version: second.version.clone(),
                        },
                    },
                }
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let second_bundle = sample_bundle(&second.module_path, &second.version, &second.fetch_url);
    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::CacheLookup {
            result: ModuleCacheLookupResult::Hit {
                module: second_bundle.clone(),
            },
        },
    });
    assert_eq!(
        response,
        EngineResponse::ModuleGraphResult {
            modules: vec![first_bundle, second_bundle],
        }
    );
}

#[test]
fn module_load_reports_host_error_categories_for_fetch_failures() {
    let mut engine = Engine::new();
    let root = sample_root(
        "example.com/hello",
        "v1.2.3",
        "https://example.com/hello.zip",
    );

    let response = engine.handle_request(EngineRequest::LoadModuleGraph {
        modules: vec![root.clone()],
    });
    let request_id = match response {
        EngineResponse::ModuleRequest { request_id, .. } => request_id,
        other => panic!("unexpected response: {other:?}"),
    };

    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::CacheLookup {
            result: ModuleCacheLookupResult::Miss,
        },
    });
    match response {
        EngineResponse::ModuleRequest { .. } => {}
        other => panic!("unexpected response: {other:?}"),
    }

    let response = engine.handle_request(EngineRequest::ResumeModule {
        request_id,
        module: ModuleResult::Fetch {
            result: ModuleFetchResult::Error {
                message: "network down".into(),
            },
        },
    });
    match response {
        EngineResponse::Fatal { message, category } => {
            assert_eq!(category, ErrorCategory::HostError);
            assert_eq!(
                message,
                "module fetch failed for example.com/hello@v1.2.3: network down"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
