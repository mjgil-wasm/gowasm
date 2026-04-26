use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn benchmark_script() -> PathBuf {
    repo_root().join("scripts/check-benchmark-suite.sh")
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

fn temp_file(name: &str, extension: &str, contents: &str) -> PathBuf {
    let path = unique_temp_path(name, extension);
    fs::write(&path, contents).expect("temporary file should be written");
    path
}

fn run_benchmark_script(args: &[&str]) -> Output {
    Command::new("bash")
        .arg(benchmark_script())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("benchmark suite script should run")
}

#[test]
fn benchmark_suite_lists_checked_benchmarks() {
    let output = run_benchmark_script(&["--list-benchmarks"]);
    assert!(
        output.status.success(),
        "benchmark suite should list benchmarks: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for benchmark in [
        "parser_smoke",
        "compiler_smoke",
        "vm_smoke",
        "gc_smoke",
        "maps_smoke",
        "channels_smoke",
        "select_smoke",
        "json_smoke",
        "fmt_smoke",
        "package_tests_smoke",
        "module_cache_smoke",
        "worker_startup_smoke",
        "browser_run_smoke",
    ] {
        assert!(
            stdout.contains(benchmark),
            "benchmark list should contain `{benchmark}`, got:\n{stdout}"
        );
    }
}

#[test]
fn benchmark_suite_dry_run_prints_commands_and_browser_metrics() {
    let output = run_benchmark_script(&[
        "--dry-run",
        "--benchmark",
        "parser_smoke",
        "--benchmark",
        "worker_startup_smoke",
    ]);
    assert!(
        output.status.success(),
        "benchmark suite dry-run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("parser_smoke: budget<=") && stdout.contains("cargo test"),
        "dry-run should print the parser command, got:\n{stdout}"
    );
    assert!(
        stdout.contains("worker_startup_smoke: budget<=")
            && stdout.contains("metrics[worker_boot_ms]"),
        "dry-run should print the browser metric mapping, got:\n{stdout}"
    );
}

#[test]
fn benchmark_suite_succeeds_for_matching_temp_suite_and_budget() {
    let suite = temp_file(
        "benchmark-suite-success-suite",
        "json",
        r#"{
  "schema_version": 1,
  "benchmarks": [
    {
      "id": "command_case",
      "name": "Command Case",
      "kind": "command",
      "metric_key": "command_case_ms",
      "budget_key": "command_case_ms",
      "argv": ["/bin/sh", "-lc", "sleep 0.01"],
      "runs": 2,
      "tags": ["command"]
    },
    {
      "id": "browser_case",
      "name": "Browser Case",
      "kind": "browser_metric",
      "metric_key": "browser_case_ms",
      "budget_key": "browser_case_ms",
      "source_metric": "worker_boot_ms",
      "tags": ["browser"]
    }
  ]
}"#,
    );
    let budget = temp_file(
        "benchmark-suite-success-budget",
        "env",
        "command_case_ms=500\nbrowser_case_ms=500\n",
    );
    let metrics = temp_file(
        "benchmark-suite-success-metrics",
        "env",
        "worker_boot_ms=123\n",
    );
    let report = unique_temp_path("benchmark-suite-success-report", "json");

    let output = run_benchmark_script(&[
        "--suite",
        suite.to_str().expect("utf-8 path"),
        "--budget",
        budget.to_str().expect("utf-8 path"),
        "--metrics",
        metrics.to_str().expect("utf-8 path"),
        "--report-output",
        report.to_str().expect("utf-8 path"),
    ]);
    let _ = fs::remove_file(&suite);
    let _ = fs::remove_file(&budget);
    let _ = fs::remove_file(&metrics);

    assert!(
        output.status.success(),
        "benchmark suite should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value =
        serde_json::from_slice(&fs::read(&report).expect("report should be written"))
            .expect("report should be valid json");
    assert_eq!(payload["status"], "passed");
    assert_eq!(payload["results"][0]["status"], "passed");
    assert_eq!(payload["results"][1]["status"], "passed");
    let _ = fs::remove_file(&report);
}

#[test]
fn benchmark_suite_fails_when_metric_exceeds_budget() {
    let suite = temp_file(
        "benchmark-suite-budget-failure-suite",
        "json",
        r#"{
  "schema_version": 1,
  "benchmarks": [
    {
      "id": "browser_case",
      "name": "Browser Case",
      "kind": "browser_metric",
      "metric_key": "browser_case_ms",
      "budget_key": "browser_case_ms",
      "source_metric": "worker_boot_ms",
      "tags": ["browser"]
    }
  ]
}"#,
    );
    let budget = temp_file(
        "benchmark-suite-budget-failure-budget",
        "env",
        "browser_case_ms=10\n",
    );
    let metrics = temp_file(
        "benchmark-suite-budget-failure-metrics",
        "env",
        "worker_boot_ms=25\n",
    );
    let report = unique_temp_path("benchmark-suite-budget-failure-report", "json");

    let output = run_benchmark_script(&[
        "--suite",
        suite.to_str().expect("utf-8 path"),
        "--budget",
        budget.to_str().expect("utf-8 path"),
        "--metrics",
        metrics.to_str().expect("utf-8 path"),
        "--report-output",
        report.to_str().expect("utf-8 path"),
    ]);
    let _ = fs::remove_file(&suite);
    let _ = fs::remove_file(&budget);
    let _ = fs::remove_file(&metrics);

    assert!(
        !output.status.success(),
        "benchmark suite should fail when a metric exceeds budget"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("exceeded budget"),
        "failure output should mention the budget overflow, got:\n{stderr}"
    );

    let payload: Value =
        serde_json::from_slice(&fs::read(&report).expect("report should be written"))
            .expect("report should be valid json");
    assert_eq!(payload["status"], "budget_failed");
    assert_eq!(payload["results"][0]["status"], "budget_failed");
    let _ = fs::remove_file(&report);
}
