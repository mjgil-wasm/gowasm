# Package Test Support

Task `076` freezes the current package-test runner surface.

## Ownership

- `crates/host-types`
  - owns the `test_package` request and `test_result` response schema
- `crates/engine`
  - owns same-package runner synthesis, exact-name filtering, `package main`
    rewriting, and structured package-test result details
- `web/`
  - owns the browser worker and shell paths that submit package-test requests
    and render `test_result`

## Supported Slice

- same-package top-level `Test*` functions
- `package main` tests by rewriting user-defined top-level `main` functions out
  of the synthetic test workspace before injecting the generated runner
- exact function-name filtering through `test_package.filter`
- structured result details through:
  - `subject_path`
  - `planned_tests`
  - `completed_tests`
  - `active_test`
- stdout capture from the generated runner and the test bodies it executes
- panic and runtime-budget timeout failures through the ordinary runtime
  diagnostic path, with `active_test` pinned from the last `RUN ...` marker

## Explicit Limits

- external `_test` packages are rejected explicitly
- only top-level same-package `Test*` functions with no receiver, no
  parameters, and no return values are supported
- package-test filtering is intentionally narrow and currently matches one
  exact test function name instead of full `go test -run` regex behavior
