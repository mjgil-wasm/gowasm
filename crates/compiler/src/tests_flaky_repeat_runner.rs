use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn runner_script() -> PathBuf {
    repo_root().join("scripts/flaky_repeat_runner.py")
}

fn unique_temp_path(name: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "gowasm-{name}-{}-{unique}.{extension}",
        process::id()
    ))
}

fn temp_json_file(name: &str, contents: &str) -> PathBuf {
    let path = unique_temp_path(name, "json");
    fs::write(&path, contents).expect("temporary json file should be written");
    path
}

fn run_runner(args: &[&str]) -> Output {
    Command::new("python3")
        .arg(runner_script())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("flaky repeat runner should run")
}

#[test]
fn flaky_repeat_runner_lists_checked_targets() {
    let output = run_runner(&["--list-targets"]);
    assert!(
        output.status.success(),
        "runner should list targets: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for target in [
        "vm_scheduler",
        "vm_select_runtime",
        "compiler_sync_concurrency",
        "browser_shell_soak",
    ] {
        assert!(
            stdout.contains(target),
            "target list should contain `{target}`, got:\n{stdout}"
        );
    }
}

#[test]
fn flaky_repeat_runner_lists_deterministic_seed_sequence() {
    let output = run_runner(&["--list-seeds", "--seed", "274668190", "--runs", "4"]);
    assert!(
        output.status.success(),
        "runner should list seeds: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let seeds: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        seeds,
        vec![
            "6557114237473886485",
            "18356994681197849856",
            "7830583811065096783",
            "7779428165923807794",
        ],
        "seed sequence should stay stable"
    );
}

#[test]
fn flaky_repeat_runner_dry_run_logs_seed_and_browser_query() {
    let catalog = temp_json_file(
        "flaky-repeat-browser-dry-run",
        r#"{
  "schema_version": 1,
  "targets": [
    {
      "id": "browser_case",
      "name": "Browser Case",
      "kind": "browser_page",
      "page": "web/test-browser-shell-soak.html",
      "page_query_template": "seed={seed}&cycles=1",
      "element_id": "summary",
      "expect_substrings": ["ok"],
      "reject_substrings": ["failure"],
      "timeout_seconds": 30,
      "artifact_element_id": "artifact",
      "tags": ["browser", "dry-run"]
    }
  ]
}"#,
    );
    let report = unique_temp_path("flaky-repeat-browser-dry-run-report", "json");
    let output = run_runner(&[
        "--catalog",
        catalog.to_str().expect("utf-8 path"),
        "--dry-run",
        "--runs",
        "1",
        "--seed",
        "9",
        "--report-output",
        report.to_str().expect("utf-8 path"),
    ]);
    let _ = fs::remove_file(&catalog);

    assert!(
        output.status.success(),
        "dry-run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("browser_case seed=3379688834381445604 iteration=0"),
        "dry-run should log the derived seed, got:\n{stdout}"
    );
    assert!(
        stdout.contains("--page-query 'seed=3379688834381445604&cycles=1'")
            || stdout.contains("--page-query seed=3379688834381445604&cycles=1"),
        "dry-run should include the formatted browser query, got:\n{stdout}"
    );

    let payload: Value =
        serde_json::from_slice(&fs::read(&report).expect("dry-run report should be written"))
            .expect("dry-run report should be valid json");
    assert_eq!(payload["status"], "dry_run");
    assert_eq!(
        payload["results"][0]["env_overrides"]["GOWASM_FLAKY_TARGET"],
        "browser_case"
    );
    assert_eq!(
        payload["results"][0]["env_overrides"]["GOWASM_FLAKY_ITERATION"],
        "0"
    );
    let _ = fs::remove_file(&report);
}

#[test]
fn flaky_repeat_runner_writes_failure_report_with_replay_details() {
    let catalog = temp_json_file(
        "flaky-repeat-failure",
        r#"{
  "schema_version": 1,
  "targets": [
    {
      "id": "forced_failure",
      "name": "Forced Failure",
      "kind": "command",
      "argv": ["/bin/sh", "-lc", "exit 9"],
      "tags": ["command", "failure"]
    }
  ]
}"#,
    );
    let report = unique_temp_path("flaky-repeat-failure-report", "json");
    let output = run_runner(&[
        "--catalog",
        catalog.to_str().expect("utf-8 path"),
        "--runs",
        "1",
        "--seed",
        "12",
        "--report-output",
        report.to_str().expect("utf-8 path"),
    ]);
    let _ = fs::remove_file(&catalog);

    assert!(
        !output.status.success(),
        "runner should fail for a failing target"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("forced_failure"),
        "failure output should mention the failing target, got:\n{stderr}"
    );

    let payload: Value =
        serde_json::from_slice(&fs::read(&report).expect("failure report should be written"))
            .expect("failure report should be valid json");
    assert_eq!(payload["status"], "failed");
    assert_eq!(payload["failure"]["target_id"], "forced_failure");
    assert_eq!(payload["failure"]["exit_code"], 9);
    assert_eq!(
        payload["failure"]["env_overrides"]["GOWASM_FLAKY_TARGET"],
        "forced_failure"
    );
    assert!(
        payload["replay"]["shell_command"]
            .as_str()
            .expect("replay shell command should be a string")
            .contains("--replay-report"),
        "replay shell command should point back at the saved report"
    );
    let _ = fs::remove_file(&report);
}

#[test]
fn flaky_repeat_runner_replays_saved_failure_report() {
    let report = temp_json_file(
        "flaky-repeat-replay",
        r#"{
  "schema_version": 1,
  "failure": {
    "argv": ["/bin/sh", "-lc", "exit 7"],
    "env_overrides": {
      "GOWASM_FLAKY_SEED": "42",
      "GOWASM_FLAKY_ITERATION": "0",
      "GOWASM_FLAKY_TARGET": "replay_case"
    }
  }
}"#,
    );
    let output = run_runner(&["--replay-report", report.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&report);

    assert_eq!(
        output.status.code(),
        Some(7),
        "replay should return the failing target exit code"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("==> replay: /bin/sh -lc 'exit 7'")
            || stdout.contains("==> replay: /bin/sh -lc exit 7"),
        "replay output should show the saved command, got:\n{stdout}"
    );
}
