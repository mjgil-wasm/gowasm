use gowasm_host_types::{EngineRequest, EngineResponse, TestRunnerKind, WorkspaceFile};

use super::{compile_cache::CompileReuse, Engine};

fn workspace_file(path: &str, contents: &str) -> WorkspaceFile {
    WorkspaceFile {
        path: path.into(),
        contents: contents.into(),
    }
}

#[test]
fn snippet_test_request_passes_and_reuses_exact_snapshot() {
    let mut engine = Engine::new();
    let request = EngineRequest::TestSnippet {
        files: vec![workspace_file(
            "main.go",
            "import \"fmt\"\n\nfmt.Println(\"hello\")\n",
        )],
        entry_path: "main.go".into(),
    };

    let first = engine.handle_request(request.clone());
    match first {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Snippet);
            assert!(passed);
            assert_eq!(stdout, "hello\n");
            assert!(diagnostics.is_empty());
            assert_eq!(details.subject_path, "main.go");
            assert_eq!(details.planned_tests, vec!["main.go"]);
            assert_eq!(details.completed_tests, vec!["main.go"]);
            assert_eq!(details.active_test, None);
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected snippet compile session")
            .last_reuse,
        CompileReuse::Recompiled,
    );

    let second = engine.handle_request(request);
    match second {
        EngineResponse::TestResult {
            runner,
            passed,
            diagnostics,
            details,
            ..
        } => {
            assert_eq!(runner, TestRunnerKind::Snippet);
            assert!(passed);
            assert!(diagnostics.is_empty());
            assert_eq!(details.completed_tests, vec!["main.go"]);
        }
        other => panic!("unexpected response: {other:?}"),
    }
    assert_eq!(
        engine
            .compile_cache
            .sessions
            .get("main.go")
            .expect("expected snippet compile session")
            .last_reuse,
        CompileReuse::ReusedExact,
    );
}

#[test]
fn package_test_request_runs_same_package_test_functions() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file(
                "calc.go",
                "package calc\n\nfunc Add(left int, right int) int {\n\treturn left + right\n}\n",
            ),
            workspace_file(
                "calc_test.go",
                "package calc\n\nfunc TestAdd() {\n\tif Add(2, 3) != 5 {\n\t\tpanic(\"expected Add to sum inputs\")\n\t}\n}\n",
            ),
        ],
        target_path: "calc.go".into(),
        filter: None,
    });
    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(passed);
            assert!(diagnostics.is_empty());
            assert!(stdout.contains("RUN TestAdd"));
            assert!(stdout.contains("PASS TestAdd"));
            assert!(stdout.contains("PASS"));
            assert_eq!(details.subject_path, "calc.go");
            assert_eq!(details.planned_tests, vec!["TestAdd"]);
            assert_eq!(details.completed_tests, vec!["TestAdd"]);
            assert_eq!(details.active_test, None);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn package_test_request_runs_tests_without_executing_existing_main() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file(
                "main.go",
                "package main\n\nfunc main() {\n\tpanic(\"original main should not run during package tests\")\n}\n",
            ),
            workspace_file(
                "main_test.go",
                "package main\n\nimport \"fmt\"\n\nfunc TestExample() {\n\tfmt.Println(\"package-main-test\")\n}\n",
            ),
        ],
        target_path: "main.go".into(),
        filter: None,
    });
    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(passed);
            assert!(diagnostics.is_empty());
            assert!(stdout.contains("RUN TestExample"));
            assert!(stdout.contains("PASS TestExample"));
            assert!(stdout.contains("package-main-test"));
            assert!(!stdout.contains("original main should not run"));
            assert_eq!(details.subject_path, "main.go");
            assert_eq!(details.planned_tests, vec!["TestExample"]);
            assert_eq!(details.completed_tests, vec!["TestExample"]);
            assert_eq!(details.active_test, None);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn package_test_request_runs_table_driven_testing_t_main_package_tests() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file(
                "main.go",
                "package main\n\nimport \"fmt\"\n\nfunc Add(a, b int) int {\n\treturn a + b\n}\n\nfunc main() {\n\tresult := Add(5, 7)\n\tfmt.Printf(\"5 + 7 = %d\\n\", result)\n}\n",
            ),
            workspace_file(
                "main_test.go",
                "package main\n\nimport \"testing\"\n\nfunc TestAdd(t *testing.T) {\n\ttests := []struct {\n\t\tname string\n\t\ta int\n\t\tb int\n\t\texpected int\n\t}{\n\t\t{\"positive numbers\", 2, 3, 5},\n\t\t{\"negative numbers\", -2, -4, -6},\n\t\t{\"mixed numbers\", -1, 5, 4},\n\t\t{\"zeroes\", 0, 0, 0},\n\t}\n\n\tfor _, tc := range tests {\n\t\tt.Run(tc.name, func(t *testing.T) {\n\t\t\tresult := Add(tc.a, tc.b)\n\t\t\tif result != tc.expected {\n\t\t\t\tt.Errorf(\"Add(%d, %d) = %d; want %d\", tc.a, tc.b, result, tc.expected)\n\t\t\t}\n\t\t})\n\t}\n}\n",
            ),
        ],
        target_path: "main.go".into(),
        filter: None,
    });
    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(passed);
            assert!(diagnostics.is_empty());
            assert!(stdout.contains("RUN TestAdd"));
            assert!(stdout.contains("PASS TestAdd"));
            assert!(stdout.contains("PASS"));
            assert_eq!(details.subject_path, "main.go");
            assert_eq!(details.planned_tests, vec!["TestAdd"]);
            assert_eq!(details.completed_tests, vec!["TestAdd"]);
            assert_eq!(details.active_test, None);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn package_test_request_reports_active_and_not_run_tests_after_failure() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file(
                "calc.go",
                "package calc\n\nfunc Add(left int, right int) int {\n\treturn left + right\n}\n",
            ),
            workspace_file(
                "calc_test.go",
                "package calc\n\nfunc TestAdd() {}\n\nfunc TestFail() {\n\tpanic(\"boom\")\n}\n\nfunc TestAfter() {}\n",
            ),
        ],
        target_path: "calc.go".into(),
        filter: None,
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(!passed);
            assert!(stdout.contains("RUN TestAdd"));
            assert!(stdout.contains("PASS TestAdd"));
            assert!(stdout.contains("RUN TestFail"));
            assert!(!stdout.contains("PASS TestFail"));
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(details.subject_path, "calc.go");
            assert_eq!(
                details.planned_tests,
                vec!["TestAdd", "TestFail", "TestAfter"]
            );
            assert_eq!(details.completed_tests, vec!["TestAdd"]);
            assert_eq!(details.active_test.as_deref(), Some("TestFail"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn package_test_request_filters_to_an_exact_test_name() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file(
                "calc.go",
                "package calc\n\nfunc Add(left int, right int) int {\n\treturn left + right\n}\nfunc Sub(left int, right int) int {\n\treturn left - right\n}\n",
            ),
            workspace_file(
                "calc_test.go",
                "package calc\n\nimport \"fmt\"\n\nfunc TestAdd() {\n\tfmt.Println(\"add-ran\")\n}\n\nfunc TestSub() {\n\tfmt.Println(\"sub-ran\")\n}\n",
            ),
        ],
        target_path: "calc.go".into(),
        filter: Some("TestSub".into()),
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(passed);
            assert!(diagnostics.is_empty());
            assert!(!stdout.contains("TestAdd"));
            assert!(stdout.contains("RUN TestSub"));
            assert!(stdout.contains("PASS TestSub"));
            assert!(stdout.contains("sub-ran"));
            assert_eq!(details.subject_path, "calc.go");
            assert_eq!(details.planned_tests, vec!["TestSub"]);
            assert_eq!(details.completed_tests, vec!["TestSub"]);
            assert_eq!(details.active_test, None);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn package_test_request_rejects_external_test_packages_explicitly() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file("calc.go", "package calc\n\nfunc Add() {}\n"),
            workspace_file(
                "calc_external_test.go",
                "package calc_test\n\nfunc TestExternal() {}\n",
            ),
        ],
        target_path: "calc_external_test.go".into(),
        filter: None,
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(!passed);
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            assert!(diagnostics[0]
                .message
                .contains("external test packages ending in `_test` are not yet supported"));
            assert_eq!(details.subject_path, "calc_external_test.go");
            assert!(details.planned_tests.is_empty());
            assert!(details.completed_tests.is_empty());
            assert_eq!(details.active_test, None);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn package_test_request_reports_the_active_test_after_budget_timeout() {
    let mut engine = Engine::with_instruction_budget(200);
    let response = engine.handle_request(EngineRequest::TestPackage {
        files: vec![
            workspace_file("calc.go", "package calc\n"),
            workspace_file(
                "calc_test.go",
                "package calc\n\nfunc TestLoop() {\n\ttotal := 0\n\tfor i := 0; i < 1000000; i++ {\n\t\ttotal += i\n\t}\n\t_ = total\n}\n",
            ),
        ],
        target_path: "calc.go".into(),
        filter: Some("TestLoop".into()),
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Package);
            assert!(!passed);
            assert!(stdout.contains("RUN TestLoop"));
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(
                diagnostics[0].category,
                gowasm_host_types::ErrorCategory::RuntimeBudgetExhaustion
            );
            assert_eq!(details.subject_path, "calc.go");
            assert_eq!(details.planned_tests, vec!["TestLoop"]);
            assert!(details.completed_tests.is_empty());
            assert_eq!(details.active_test.as_deref(), Some("TestLoop"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn snippet_test_request_remaps_compile_failure_to_the_original_snippet_line() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestSnippet {
        files: vec![workspace_file(
            "main.go",
            "import \"example.com/missing/tool\"\n\nprintln(\"hello\")\n",
        )],
        entry_path: "main.go".into(),
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Snippet);
            assert!(!passed);
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].file_path.as_deref(), Some("main.go"));
            assert_eq!(
                diagnostics[0]
                    .position
                    .as_ref()
                    .map(|position| position.line),
                Some(1)
            );
            assert!(diagnostics[0].message.contains("--> main.go:1:"));
            assert!(diagnostics[0]
                .message
                .contains("1 | import \"example.com/missing/tool\""));
            assert_eq!(details.subject_path, "main.go");
            assert_eq!(details.planned_tests, vec!["main.go"]);
            assert!(details.completed_tests.is_empty());
            assert_eq!(details.active_test.as_deref(), Some("main.go"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn snippet_test_request_remaps_runtime_failures_to_the_original_snippet_line() {
    let mut engine = Engine::new();
    let response = engine.handle_request(EngineRequest::TestSnippet {
        files: vec![workspace_file("main.go", "panic(\"boom\")\n")],
        entry_path: "main.go".into(),
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            stdout,
            diagnostics,
            details,
        } => {
            assert_eq!(runner, TestRunnerKind::Snippet);
            assert!(!passed);
            assert_eq!(stdout, "");
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].file_path.as_deref(), Some("main.go"));
            assert_eq!(
                diagnostics[0].position,
                Some(gowasm_host_types::Position { line: 1, column: 1 })
            );
            let runtime = diagnostics[0]
                .runtime
                .as_ref()
                .expect("expected runtime diagnostic details");
            let first_frame = runtime
                .stack_trace
                .first()
                .expect("expected a runtime stack frame");
            let source_location = first_frame
                .source_location
                .as_ref()
                .expect("expected source location for the snippet frame");
            assert_eq!(source_location.path, "main.go");
            assert_eq!(source_location.line, 1);
            assert_eq!(source_location.column, 1);
            assert_eq!(details.subject_path, "main.go");
            assert_eq!(details.planned_tests, vec!["main.go"]);
            assert!(details.completed_tests.is_empty());
            assert_eq!(details.active_test.as_deref(), Some("main.go"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn snippet_test_request_reports_budget_timeout_on_the_original_snippet_line() {
    let mut engine = Engine::with_instruction_budget(50);
    let response = engine.handle_request(EngineRequest::TestSnippet {
        files: vec![workspace_file("main.go", "for {\n}\n")],
        entry_path: "main.go".into(),
    });

    match response {
        EngineResponse::TestResult {
            runner,
            passed,
            diagnostics,
            details,
            ..
        } => {
            assert_eq!(runner, TestRunnerKind::Snippet);
            assert!(!passed);
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(
                diagnostics[0].category,
                gowasm_host_types::ErrorCategory::RuntimeBudgetExhaustion
            );
            assert_eq!(details.subject_path, "main.go");
            assert_eq!(details.planned_tests, vec!["main.go"]);
            assert!(details.completed_tests.is_empty());
            assert_eq!(details.active_test.as_deref(), Some("main.go"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
