use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn gate_script() -> PathBuf {
    repo_root().join("scripts/check-release-gate.sh")
}

fn temp_commands_file(name: &str, contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("gowasm-{name}-{}-{unique}.json", process::id()));
    fs::write(&path, contents).expect("commands file should be written");
    path
}

fn run_gate(args: &[&str]) -> Output {
    Command::new("bash")
        .arg(gate_script())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("release gate script should run")
}

#[test]
fn release_gate_script_lists_required_suites() {
    let output = run_gate(&["--list"]);
    assert!(
        output.status.success(),
        "script should list suites successfully: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for suite in [
        "compile_checks",
        "release_artifact_reproducibility",
        "unit_tests",
        "differential_corpora",
        "fuzz_property_tests",
        "browser_worker_tests",
        "browser_shell_tests",
        "capture_performance_metrics",
        "performance_budgets",
    ] {
        assert!(
            stdout.contains(suite),
            "suite list should contain `{suite}`, got:\n{stdout}"
        );
    }
}

#[test]
fn release_gate_script_succeeds_for_matching_command_file() {
    let commands = temp_commands_file(
        "release-gate-success",
        r#"{
  "schema_version": 1,
  "suites": [
    { "name": "success_one", "argv": ["/bin/sh", "-lc", "exit 0"] },
    { "name": "success_two", "argv": ["/bin/sh", "-lc", "printf ok >/dev/null"] }
  ]
}"#,
    );
    let output = run_gate(&["--commands-file", commands.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&commands);

    assert!(
        output.status.success(),
        "script should succeed, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("release gate: 2 suite(s) passed"),
        "success output should summarize passing suites, got:\n{stdout}"
    );
}

#[test]
fn release_gate_script_fails_for_failing_subgate() {
    let commands = temp_commands_file(
        "release-gate-failure",
        r#"{
  "schema_version": 1,
  "suites": [
    { "name": "success_before_failure", "argv": ["/bin/sh", "-lc", "exit 0"] },
    { "name": "forced_failure", "argv": ["/bin/sh", "-lc", "exit 7"] }
  ]
}"#,
    );
    let output = run_gate(&["--commands-file", commands.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&commands);

    assert!(
        !output.status.success(),
        "script should fail for a failing suite"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("forced_failure"),
        "failure output should mention the failing suite, got:\n{stderr}"
    );
}
