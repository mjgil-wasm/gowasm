use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn runner_script() -> PathBuf {
    repo_root().join("scripts/run-browser-acceptance-corpus.py")
}

fn temp_index_file(name: &str, contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("gowasm-{name}-{}-{unique}.json", process::id()));
    fs::write(&path, contents).expect("index file should be written");
    path
}

fn run_runner(args: &[&str]) -> Output {
    Command::new("python3")
        .arg(runner_script())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("browser acceptance runner should run")
}

#[test]
fn browser_acceptance_runner_lists_required_cases() {
    let output = run_runner(&["--list-cases"]);
    assert!(
        output.status.success(),
        "runner should list cases: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for case_id in [
        "worker_end_to_end_capabilities",
        "worker_cancellation_recovery",
        "shell_module_cache_and_workspace_fs",
        "shell_snapshot_roundtrip",
        "shell_project_archive_import",
        "shell_cancellation_and_recovery",
        "shell_error_ui_diagnostics",
    ] {
        assert!(
            stdout.contains(case_id),
            "case list should contain `{case_id}`, got:\n{stdout}"
        );
    }
}

#[test]
fn browser_acceptance_runner_lists_required_tags() {
    let output = run_runner(&["--list-tags"]);
    assert!(
        output.status.success(),
        "runner should list tags: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for tag in [
        "workspace_fs",
        "fetch",
        "timers",
        "context_cancellation",
        "module_cache",
        "snapshots",
        "project_import",
        "worker_recovery",
        "dom",
        "worker_protocol",
        "output",
        "diagnostics",
    ] {
        assert!(
            stdout.contains(tag),
            "tag list should contain `{tag}`, got:\n{stdout}"
        );
    }
}

#[test]
fn browser_acceptance_runner_dry_run_filters_worker_cases() {
    let output = run_runner(&["--tag", "worker", "--dry-run"]);
    assert!(
        output.status.success(),
        "runner dry-run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("web/ci-worker-smoke.html"),
        "worker dry-run should include worker end-to-end page, got:\n{stdout}"
    );
    assert!(
        stdout.contains("web/test-worker-cancellation.html"),
        "worker dry-run should include worker cancellation page, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("web/test-browser-shell-archive.html"),
        "worker dry-run should exclude shell-only pages, got:\n{stdout}"
    );
}

#[test]
fn browser_acceptance_runner_rejects_index_missing_required_coverage_tags() {
    let index = temp_index_file(
        "browser-acceptance-missing-tags",
        r#"{
  "schema_version": 1,
  "cases": [
    {
      "id": "worker_only",
      "name": "worker only",
      "page": "web/test-worker.html",
      "element_id": "summary",
      "expect_substrings": ["assertions passed"],
      "reject_substrings": ["failed"],
      "timeout_seconds": 120,
      "tags": ["worker", "worker_protocol", "output"]
    }
  ]
}"#,
    );
    let output = run_runner(&["--index", index.to_str().expect("utf-8 path"), "--dry-run"]);
    let _ = fs::remove_file(&index);

    assert!(
        !output.status.success(),
        "runner should fail for missing required coverage tags"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("missing required coverage tag"),
        "failure should mention missing coverage tags, got:\n{stderr}"
    );
}
