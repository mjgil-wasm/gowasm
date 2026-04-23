use super::{
    EngineRequest, EngineResponse, ModuleCacheFillRequest, ModuleCacheKey,
    ModuleCacheLookupRequest, ModuleCacheLookupResult, ModuleFetchRequest, ModuleFetchResult,
    ModuleGraphRoot, ModuleRequest, ModuleResult, ModuleSourceBundle, WorkspaceFile,
};

#[test]
fn module_request_round_trips_through_json() {
    let request = EngineRequest::LoadModuleGraph {
        modules: vec![ModuleGraphRoot {
            module_path: "example.com/hello".into(),
            version: "v1.2.3".into(),
            fetch_url: "https://modules.example.com/hello/v1.2.3.json".into(),
        }],
    };
    let json = serde_json::to_string(&request).expect("request should serialize");
    let decoded: EngineRequest = serde_json::from_str(&json).expect("request should parse");
    assert_eq!(decoded, request);

    let response = EngineResponse::ModuleRequest {
        request_id: 41,
        module: ModuleRequest::Fetch {
            request: ModuleFetchRequest {
                module: ModuleCacheKey {
                    module_path: "example.com/hello".into(),
                    version: "v1.2.3".into(),
                },
                fetch_url: "https://modules.example.com/hello/v1.2.3.json".into(),
            },
        },
    };
    let response_json = serde_json::to_string(&response).expect("response should serialize");
    let decoded_response: EngineResponse =
        serde_json::from_str(&response_json).expect("response should parse");
    assert_eq!(decoded_response, response);
}

#[test]
fn module_resume_and_graph_result_round_trip_through_json() {
    let bundle = ModuleSourceBundle {
        module: ModuleCacheKey {
            module_path: "example.com/hello".into(),
            version: "v1.2.3".into(),
        },
        origin_url: "https://modules.example.com/hello/v1.2.3.json".into(),
        files: vec![WorkspaceFile {
            path: "go.mod".into(),
            contents: "module example.com/hello\n\ngo 1.21\n".into(),
        }],
    };

    let resume = EngineRequest::ResumeModule {
        request_id: 41,
        module: ModuleResult::CacheLookup {
            result: ModuleCacheLookupResult::Hit {
                module: bundle.clone(),
            },
        },
    };
    let resume_json = serde_json::to_string(&resume).expect("request should serialize");
    let decoded_resume: EngineRequest =
        serde_json::from_str(&resume_json).expect("request should parse");
    assert_eq!(decoded_resume, resume);

    let graph_result = EngineResponse::ModuleGraphResult {
        modules: vec![bundle.clone()],
    };
    let graph_result_json =
        serde_json::to_string(&graph_result).expect("response should serialize");
    let decoded_graph_result: EngineResponse =
        serde_json::from_str(&graph_result_json).expect("response should parse");
    assert_eq!(decoded_graph_result, graph_result);
}

#[test]
fn module_protocol_supports_cache_hit_and_fill_shapes() {
    let bundle = ModuleSourceBundle {
        module: ModuleCacheKey {
            module_path: "example.com/hello".into(),
            version: "v1.2.3".into(),
        },
        origin_url: "https://modules.example.com/hello/v1.2.3.json".into(),
        files: vec![WorkspaceFile {
            path: "hello/hello.go".into(),
            contents: "package hello\n".into(),
        }],
    };

    assert_eq!(
        ModuleRequest::CacheLookup {
            request: ModuleCacheLookupRequest {
                module: bundle.module.clone(),
            },
        },
        ModuleRequest::CacheLookup {
            request: ModuleCacheLookupRequest {
                module: ModuleCacheKey {
                    module_path: "example.com/hello".into(),
                    version: "v1.2.3".into(),
                },
            },
        }
    );

    assert_eq!(
        ModuleRequest::CacheFill {
            request: ModuleCacheFillRequest {
                module: bundle.clone(),
            },
        },
        ModuleRequest::CacheFill {
            request: ModuleCacheFillRequest { module: bundle },
        }
    );

    assert_eq!(
        ModuleResult::Fetch {
            result: ModuleFetchResult::Error {
                message: "network down".into(),
            },
        },
        ModuleResult::Fetch {
            result: ModuleFetchResult::Error {
                message: "network down".into(),
            },
        }
    );
}
