use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn validator_script() -> PathBuf {
    repo_root().join("scripts/check-ci-matrix.py")
}

fn temp_workflow_file(name: &str, contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("gowasm-{name}-{}-{unique}.yml", process::id()));
    fs::write(&path, contents).expect("workflow file should be written");
    path
}

fn run_validator(args: &[&str]) -> Output {
    Command::new("python3")
        .arg(validator_script())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("ci matrix validator should run")
}

#[test]
fn ci_matrix_validator_accepts_checked_workflow() {
    let output = run_validator(&[]);
    assert!(
        output.status.success(),
        "checked workflow should validate: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("ci matrix validation passed"),
        "validator should report success, got:\n{stdout}"
    );
}

#[test]
fn ci_matrix_validator_lists_required_jobs() {
    let output = run_validator(&["--list-jobs"]);
    assert!(
        output.status.success(),
        "validator should list jobs: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for job_id in [
        "portable-rust",
        "linux-pinned-rust",
        "native-go-differential",
        "fuzz-property",
        "browser-worker",
        "browser-shell",
        "browser-performance",
    ] {
        assert!(
            stdout.contains(job_id),
            "job list should contain `{job_id}`, got:\n{stdout}"
        );
    }
}

#[test]
fn ci_matrix_validator_rejects_workflow_missing_browser_job() {
    let workflow = temp_workflow_file(
        "ci-matrix-missing-browser",
        r#"name: ci
on:
  pull_request:
jobs:
  portable-rust:
    runs-on: ubuntu-latest
"#,
    );
    let output = run_validator(&["--workflow", workflow.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&workflow);

    assert!(
        !output.status.success(),
        "validator should fail for an incomplete workflow"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("workflow should contain top-level snippet")
            || stderr.contains("browser-worker")
            || stderr.contains("workflow should define job"),
        "failure should mention the missing job, got:\n{stderr}"
    );
}
