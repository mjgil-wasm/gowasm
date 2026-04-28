use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn gate_script() -> PathBuf {
    repo_root().join("scripts/check-differential-release-gates.sh")
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

fn run_native_go_parity(args: &[&str]) -> Output {
    Command::new("bash")
        .arg(repo_root().join("scripts/run-native-go-parity-corpus.sh"))
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("native-go parity runner should run")
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
        "compiler_vm_representative_parity",
        "native_go_representative_parity",
        "compiler_vm_imported_package_release_gate",
        "compiler_vm_json_differential",
        "compiler_vm_reflect_fmt_differential",
        "compiler_vm_semantic_differential",
        "native_go_semantic_differential",
        "native_go_stdlib_differential",
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
        stdout.contains("differential release gates: 2 suite(s) passed"),
        "success output should summarize passing suites, got:\n{stdout}"
    );
}

#[test]
fn release_gate_script_fails_for_mismatched_command_file() {
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

#[test]
fn native_go_parity_runner_replays_local_package_chain_without_checked_in_go_mod() {
    let output = run_native_go_parity(&["--case", "local_package_chain"]);
    assert!(
        output.status.success(),
        "native-go parity runner should synthesize a workspace go.mod for module-free cases: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("PASS local_package_chain:"),
        "runner should pass the selected case, got:\n{stdout}"
    );
}
