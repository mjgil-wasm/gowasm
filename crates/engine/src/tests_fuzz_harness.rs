use std::env;
use std::fs;
use std::path::PathBuf;

use gowasm_compiler::compile_source;
use gowasm_host_types::{EngineRequest, EngineResponse, WorkspaceFile};
use gowasm_lexer::lex;
use gowasm_parser::{parse_source_file, parse_type_repr, TypeRepr};
use gowasm_vm::{Instruction, Vm};
use serde::{Deserialize, Serialize};

use super::Engine;

#[derive(Debug, Deserialize)]
struct FuzzCorpus {
    schema_version: u32,
    targets: Vec<FuzzTarget>,
}

#[derive(Debug, Deserialize)]
struct FuzzTarget {
    name: String,
    iterations_per_seed: usize,
    seeds: Vec<FuzzSeed>,
}

#[derive(Debug, Deserialize)]
struct FuzzSeed {
    name: String,
    data: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PropertySummary {
    schema_version: u32,
    targets: Vec<PropertySummaryTarget>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PropertySummaryTarget {
    name: String,
    seed_count: usize,
    iterations_per_seed: usize,
    mutated_case_count: usize,
    mutation_digest_hex: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus_path() -> PathBuf {
    repo_root().join("testdata/fuzz-harness/index.json")
}

fn summary_path() -> PathBuf {
    repo_root().join("testdata/fuzz-harness/property-summary.json")
}

fn load_corpus() -> FuzzCorpus {
    let path = corpus_path();
    let text = fs::read_to_string(&path).expect("fuzz corpus should be readable");
    let corpus: FuzzCorpus = serde_json::from_str(&text).expect("fuzz corpus should parse");
    assert_eq!(
        corpus.schema_version, 1,
        "fuzz corpus schema should stay frozen"
    );
    corpus
}

fn load_summary() -> PropertySummary {
    let path = summary_path();
    let text = fs::read_to_string(&path).expect("summary should be readable");
    serde_json::from_str(&text).expect("summary should parse")
}

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

fn compile_and_run(source: &str) {
    let program = match compile_source(source) {
        Ok(program) => program,
        Err(error) => panic!("expected source to compile, got: {error}"),
    };
    let mut vm = Vm::default();
    vm.run_program(&program)
        .unwrap_or_else(|error| panic!("expected source to run, got: {error}"));
}

fn compile_if_possible(source: &str) {
    let _ = compile_source(source);
}

fn json_runtime_source(payload: &str) -> String {
    let payload_literal = serde_json::to_string(payload).expect("json payload should quote");
    format!(
        "package main\n\nimport \"encoding/json\"\n\nfunc main() {{\n\tinput := []byte({payload_literal})\n\tvar value any\n\terr := json.Unmarshal(input, &value)\n\tif err == nil {{\n\t\t_, _ = json.Marshal(value)\n\t}}\n}}\n"
    )
}

fn url_runtime_source(payload: &str) -> String {
    let payload_literal = serde_json::to_string(payload).expect("url payload should quote");
    format!(
        "package main\n\nimport \"net/url\"\n\nfunc main() {{\n\tinput := {payload_literal}\n\tparsed, _ := url.Parse(input)\n\t_, _ = url.ParseRequestURI(input)\n\t_ = url.QueryEscape(input)\n\t_, _ = url.QueryUnescape(input)\n\t_ = url.PathEscape(input)\n\tif parsed != nil {{\n\t\t_ = parsed.String()\n\t\t_ = parsed.Hostname()\n\t\t_ = parsed.Port()\n\t}}\n}}\n"
    )
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

struct StableRng(u64);

impl StableRng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn choose_index(&mut self, len: usize) -> usize {
        if len == 0 {
            0
        } else {
            (self.next_u64() % len as u64) as usize
        }
    }
}

fn target_alphabet(name: &str) -> &'static [u8] {
    match name {
        "json_runtime" | "worker_protocol" | "vm_instruction_decoding" => {
            br#"{}[],:\"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_ -"# 
        }
        "url_runtime" => {
            br#"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:/?&=#%._-+"#
        }
        "type_strings_type_ast" => {
            br#"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789[]*(),<>- chanfuncmap "# 
        }
        _ => br#"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_(){}[],:;.+-*/%<>=!&|^" \n\t`"#,
    }
}

fn mutate_text(target_name: &str, seed_name: &str, input: &str, iteration: usize) -> String {
    let base_seed = fnv1a64(target_name.as_bytes())
        ^ fnv1a64(seed_name.as_bytes()).rotate_left(7)
        ^ ((iteration as u64) << 17);
    let mut rng = StableRng::new(base_seed);
    let alphabet = target_alphabet(target_name);
    let mut bytes = input.as_bytes().to_vec();
    let edit_count = 1 + (rng.next_u64() % 4) as usize;

    for _ in 0..edit_count {
        match rng.next_u64() % 3 {
            0 => {
                let index = rng.choose_index(bytes.len().saturating_add(1));
                let value = alphabet[rng.choose_index(alphabet.len())];
                bytes.insert(index.min(bytes.len()), value);
            }
            1 if !bytes.is_empty() => {
                let index = rng.choose_index(bytes.len());
                let value = alphabet[rng.choose_index(alphabet.len())];
                bytes[index] = value;
            }
            2 if !bytes.is_empty() => {
                let index = rng.choose_index(bytes.len());
                bytes.remove(index);
            }
            _ => {}
        }
    }

    if bytes.is_empty() {
        bytes.push(alphabet[0]);
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

fn replay_seed_target(target: &FuzzTarget) {
    match target.name.as_str() {
        "lexer_parser" => {
            for seed in &target.seeds {
                lex(&seed.data).expect("seed lexer source should lex");
                parse_source_file(&seed.data).expect("seed parser source should parse");
            }
        }
        "type_strings_type_ast" => {
            for seed in &target.seeds {
                let ast = parse_type_repr(&seed.data).expect("seed type should parse");
                let json = serde_json::to_string(&ast).expect("type ast should serialize");
                let decoded: TypeRepr =
                    serde_json::from_str(&json).expect("type ast should deserialize");
                assert_eq!(decoded, ast, "type ast should round-trip through json");
            }
        }
        "assignment_lowering" => {
            for seed in &target.seeds {
                compile_source(&seed.data).expect("assignment lowering seed should compile");
            }
        }
        "json_runtime" => {
            for seed in &target.seeds {
                compile_and_run(&json_runtime_source(&seed.data));
            }
        }
        "url_runtime" => {
            for seed in &target.seeds {
                compile_and_run(&url_runtime_source(&seed.data));
            }
        }
        "formatter" => {
            let mut engine = Engine::new();
            for seed in &target.seeds {
                match engine.handle_request(EngineRequest::Format {
                    files: vec![workspace_file("main.go", &seed.data)],
                }) {
                    EngineResponse::FormatResult { files, diagnostics } => {
                        assert_eq!(files.len(), 1, "formatter should keep one file");
                        assert!(
                            diagnostics.is_empty(),
                            "checked formatter seed `{}` should not emit diagnostics",
                            seed.name
                        );
                    }
                    other => panic!("unexpected formatter response: {other:?}"),
                }
            }
        }
        "worker_protocol" => {
            let mut engine = Engine::new();
            for seed in &target.seeds {
                let response_json = engine.handle_request_json(&seed.data);
                let _: EngineResponse =
                    serde_json::from_str(&response_json).expect("engine response should parse");
            }
        }
        "vm_instruction_decoding" => {
            for seed in &target.seeds {
                let instruction: Instruction =
                    serde_json::from_str(&seed.data).expect("instruction seed should decode");
                let json =
                    serde_json::to_string(&instruction).expect("instruction should serialize");
                let decoded: Instruction =
                    serde_json::from_str(&json).expect("instruction should deserialize");
                assert_eq!(decoded, instruction, "instruction should round-trip");
            }
        }
        other => panic!("unexpected fuzz target `{other}`"),
    }
}

fn exercise_mutated_target(target: &FuzzTarget) {
    match target.name.as_str() {
        "lexer_parser" => {
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    let _ = lex(&candidate);
                    let _ = parse_source_file(&candidate);
                }
            }
        }
        "type_strings_type_ast" => {
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    if let Ok(ast) = parse_type_repr(&candidate) {
                        let json = serde_json::to_string(&ast).expect("type ast should serialize");
                        let _: TypeRepr =
                            serde_json::from_str(&json).expect("type ast should deserialize");
                    }
                }
            }
        }
        "assignment_lowering" => {
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    compile_if_possible(&candidate);
                }
            }
        }
        "json_runtime" => {
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    compile_and_run(&json_runtime_source(&candidate));
                }
            }
        }
        "url_runtime" => {
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    compile_and_run(&url_runtime_source(&candidate));
                }
            }
        }
        "formatter" => {
            let mut engine = Engine::new();
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    match engine.handle_request(EngineRequest::Format {
                        files: vec![workspace_file("main.go", &candidate)],
                    }) {
                        EngineResponse::FormatResult { .. } => {}
                        other => panic!("unexpected formatter response: {other:?}"),
                    }
                }
            }
        }
        "worker_protocol" => {
            let mut engine = Engine::new();
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    let response_json = engine.handle_request_json(&candidate);
                    let _: EngineResponse =
                        serde_json::from_str(&response_json).expect("engine response should parse");
                }
            }
        }
        "vm_instruction_decoding" => {
            for seed in &target.seeds {
                for iteration in 0..target.iterations_per_seed {
                    let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                    if let Ok(instruction) = serde_json::from_str::<Instruction>(&candidate) {
                        let json = serde_json::to_string(&instruction)
                            .expect("instruction should serialize");
                        let _: Instruction =
                            serde_json::from_str(&json).expect("instruction should deserialize");
                    }
                }
            }
        }
        other => panic!("unexpected fuzz target `{other}`"),
    }
}

fn compute_property_summary(corpus: &FuzzCorpus) -> PropertySummary {
    let mut targets = Vec::with_capacity(corpus.targets.len());
    for target in &corpus.targets {
        let mut digest = 0xcbf29ce484222325_u64;
        let mut case_count = 0usize;
        for seed in &target.seeds {
            digest ^= fnv1a64(seed.name.as_bytes());
            digest = digest.wrapping_mul(0x100000001b3);
            for iteration in 0..target.iterations_per_seed {
                let candidate = mutate_text(&target.name, &seed.name, &seed.data, iteration);
                digest ^= fnv1a64(candidate.as_bytes());
                digest = digest.wrapping_mul(0x100000001b3);
                case_count += 1;
            }
        }
        targets.push(PropertySummaryTarget {
            name: target.name.clone(),
            seed_count: target.seeds.len(),
            iterations_per_seed: target.iterations_per_seed,
            mutated_case_count: case_count,
            mutation_digest_hex: format!("{digest:016x}"),
        });
    }
    PropertySummary {
        schema_version: 1,
        targets,
    }
}

#[test]
fn fuzz_seed_corpus_replays_without_panics() {
    let corpus = load_corpus();
    for target in &corpus.targets {
        replay_seed_target(target);
    }
}

#[test]
fn deterministic_property_mutations_match_checked_summary() {
    let corpus = load_corpus();
    let actual = compute_property_summary(&corpus);
    let expected = load_summary();
    assert_eq!(
        actual,
        expected,
        "deterministic mutation summary drifted; update testdata/fuzz-harness/property-summary.json if this change is intentional"
    );
}

#[test]
fn fuzz_property_mutations_exercise_all_targets_without_panics() {
    let corpus = load_corpus();
    for target in &corpus.targets {
        exercise_mutated_target(target);
    }
}
