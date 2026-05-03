use gowasm_host_types::{TestResultDetails, TestRunnerKind};

pub(super) fn finalize_test_result_details(
    runner: TestRunnerKind,
    details: TestResultDetails,
    passed: bool,
) -> TestResultDetails {
    finalize_test_result_details_with_stdout(runner, details, "", passed)
}

pub(super) fn finalize_test_result_details_with_stdout(
    runner: TestRunnerKind,
    mut details: TestResultDetails,
    stdout: &str,
    passed: bool,
) -> TestResultDetails {
    match runner {
        TestRunnerKind::Package => {
            let (completed_tests, active_test) =
                package_test_progress(stdout, &details.planned_tests, passed);
            details.completed_tests = completed_tests;
            details.active_test = active_test;
        }
        TestRunnerKind::Snippet => {
            details.completed_tests = if passed {
                details.planned_tests.clone()
            } else {
                Vec::new()
            };
            details.active_test = if passed {
                None
            } else {
                Some(details.subject_path.clone())
            };
        }
    }
    details
}

fn package_test_progress(
    stdout: &str,
    planned_tests: &[String],
    passed: bool,
) -> (Vec<String>, Option<String>) {
    let mut completed_tests = Vec::new();
    let mut active_test = None;
    for line in stdout.lines() {
        if let Some(name) = line.strip_prefix("RUN ") {
            active_test = Some(name.into());
            continue;
        }
        if let Some(name) = line.strip_prefix("PASS ") {
            completed_tests.push(name.into());
            if active_test.as_deref() == Some(name) {
                active_test = None;
            }
        }
    }
    if passed && completed_tests.len() != planned_tests.len() {
        return (planned_tests.to_vec(), None);
    }
    (completed_tests, active_test)
}
