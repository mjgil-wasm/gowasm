use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn runner_script() -> PathBuf {
    repo_root().join("scripts/run-native-go-stdlib-corpus.py")
}

fn temp_catalog_file(name: &str, contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("gowasm-{name}-{}-{unique}.json", process::id()));
    fs::write(&path, contents).expect("catalog file should be written");
    path
}

fn run_runner(args: &[&str]) -> Output {
    Command::new("python3")
        .arg(runner_script())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("stdlib differential runner should run")
}

#[test]
fn stdlib_differential_runner_lists_required_package_families() {
    let output = run_runner(&["--list-packages"]);
    assert!(
        output.status.success(),
        "runner should list packages: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    for package_id in [
        "fmt_reflect",
        "json",
        "text",
        "strconv",
        "errors",
        "path_filepath",
        "io_fs",
        "net_url",
        "regexp",
        "collections",
        "math",
        "crypto_encoding",
        "time_context_http",
        "log",
    ] {
        assert!(
            stdout.contains(package_id),
            "package list should contain `{package_id}`, got:\n{stdout}"
        );
    }
}

#[test]
fn stdlib_differential_runner_passes_for_selected_checked_packages() {
    let output = run_runner(&[
        "--package",
        "io_fs",
        "--package",
        "time_context_http",
        "--package",
        "log",
    ]);
    assert!(
        output.status.success(),
        "runner should pass for checked packages: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("stdlib native-go differential corpus: 3 package(s) passed"),
        "success output should summarize passing packages, got:\n{stdout}"
    );
}

#[test]
fn stdlib_differential_runner_lists_browser_deviations_for_custom_catalog() {
    let corpus_index = repo_root()
        .join("testdata/json-differential/index.json")
        .display()
        .to_string();
    let catalog = temp_catalog_file(
        "stdlib-differential-browser-deviations",
        &format!(
            r#"{{
  "schema_version": 1,
  "packages": [
    {{
      "id": "json",
      "name": "json",
      "packages": ["encoding/json"],
      "corpus_index": "{corpus_index}",
      "expected_output_field": "stdout",
      "host_independent": true,
      "allowed_browser_deviations": ["browser shell may summarize these cases ahead of raw output"]
    }}
  ]
}}"#,
        ),
    );
    let output = run_runner(&[
        "--index",
        catalog.to_str().expect("utf-8 path"),
        "--list-packages",
        "--show-browser-deviations",
    ]);
    let _ = fs::remove_file(&catalog);

    assert!(
        output.status.success(),
        "runner should list custom browser deviations: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("browser shell may summarize these cases ahead of raw output"),
        "listing should include browser deviations, got:\n{stdout}"
    );
}

#[test]
fn stdlib_differential_runner_rejects_missing_expected_output_field() {
    let corpus_index = repo_root()
        .join("testdata/json-differential/index.json")
        .display()
        .to_string();
    let catalog = temp_catalog_file(
        "stdlib-differential-bad-output-field",
        &format!(
            r#"{{
  "schema_version": 1,
  "packages": [
    {{
      "id": "json",
      "name": "json",
      "packages": ["encoding/json"],
      "corpus_index": "{corpus_index}",
      "expected_output_field": "stderr",
      "host_independent": true,
      "allowed_browser_deviations": []
    }}
  ]
}}"#,
        ),
    );
    let output = run_runner(&["--index", catalog.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&catalog);

    assert!(
        !output.status.success(),
        "runner should fail for a mismatched expected output field"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("expected_native_go_stderr"),
        "failure should mention the missing stderr field, got:\n{stderr}"
    );
}
