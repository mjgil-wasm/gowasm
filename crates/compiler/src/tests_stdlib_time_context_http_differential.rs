use super::tests_stdlib_differential_support::{load_corpus_index, run_stdout_differential_case};

const CORPUS_ROOT: &str = "testdata/time-context-http-differential";

#[test]
fn time_context_http_differential_corpus_matches_checked_in_native_go_outputs() {
    let index = load_corpus_index(CORPUS_ROOT);
    assert_eq!(
        index.schema_version, 1,
        "unexpected time/context/http differential schema"
    );
    assert!(
        !index.cases.is_empty(),
        "time/context/http differential corpus should contain representative cases"
    );

    for case in index.cases {
        run_stdout_differential_case(CORPUS_ROOT, case);
    }
}
