#!/usr/bin/env python3
from __future__ import annotations

import argparse
import difflib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

MODULE_CACHE_PATH_RE = re.compile(
    r"^__module_cache__/(?P<module_path>.+?)/@(?P<version>[^/]+)/(?P<relative_path>.+)$"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the checked-in representative parity corpus through native Go "
            "and compare stdout against the shared expected outputs."
        )
    )
    parser.add_argument(
        "--case",
        dest="case_ids",
        action="append",
        default=[],
        help="Only run the named corpus case id. May be provided more than once.",
    )
    parser.add_argument(
        "--go",
        default="go",
        help="Go binary to execute. Defaults to `go`.",
    )
    parser.add_argument(
        "--keep-temp",
        action="store_true",
        help="Keep successful temp workspaces instead of cleaning them up.",
    )
    parser.add_argument(
        "--list-cases",
        action="store_true",
        help="Print the available corpus case ids and exit.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    index = load_index(repo_root)

    if args.list_cases:
        for case in index["cases"]:
            print(f'{case["id"]}: {case["name"]}')
        return 0

    cases = select_cases(index["cases"], args.case_ids)
    failures: list[str] = []
    passes = 0

    for case in cases:
        native_go_outcome = (
            case.get("expected_outcomes", {}).get("native_go", {"status": "pass"})
        )
        if native_go_outcome.get("status") != "pass":
            failure_task = native_go_outcome.get("failure_task")
            print(
                f'SKIP {case["id"]}: native_go is tracked as '
                f'{native_go_outcome.get("status")} via {failure_task!r}'
            )
            continue

        error = run_case(repo_root, case, args.go, args.keep_temp)
        if error is None:
            passes += 1
            print(f'PASS {case["id"]}: {case["name"]}')
            continue
        failures.append(error)

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    print(f"native-go parity corpus: {passes} case(s) passed")
    return 0


def load_index(repo_root: Path) -> dict:
    index_path = repo_root / "testdata" / "parity-corpus" / "index.json"
    index = json.loads(index_path.read_text())
    if index.get("schema_version") != 2:
        raise SystemExit(
            f"unexpected parity corpus schema_version {index.get('schema_version')!r}"
        )
    if not index.get("cases"):
        raise SystemExit("parity corpus should contain at least one case")
    return index


def select_cases(cases: list[dict], requested_ids: list[str]) -> list[dict]:
    if not requested_ids:
        return cases

    requested = set(requested_ids)
    selected = [case for case in cases if case["id"] in requested]
    missing = sorted(requested - {case["id"] for case in selected})
    if missing:
        raise SystemExit(f"unknown parity corpus case ids: {', '.join(missing)}")
    return selected


def run_case(
    repo_root: Path, case: dict, go_binary: str, keep_temp: bool
) -> str | None:
    temp_root = Path(tempfile.mkdtemp(prefix=f'native-go-parity-{case["id"]}-'))
    try:
        workspace_root = materialize_workspace(repo_root, temp_root, case)
        command = [go_binary, "run", "."]
        completed = subprocess.run(
            command,
            cwd=workspace_root,
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            return (
                f'FAIL {case["id"]}: native Go exited with status '
                f"{completed.returncode}\n"
                f"  workspace: {workspace_root}\n"
                f"  stderr:\n{indent_text(completed.stderr.rstrip())}"
            )

        expected_stdout = case["expected_stdout"]
        if completed.stdout != expected_stdout:
            diff = "\n".join(
                difflib.unified_diff(
                    expected_stdout.splitlines(),
                    completed.stdout.splitlines(),
                    fromfile="expected_stdout",
                    tofile="native_go_stdout",
                    lineterm="",
                )
            )
            return (
                f'FAIL {case["id"]}: native Go stdout diverged from the shared '
                f"parity corpus output\n"
                f"  workspace: {workspace_root}\n"
                f"  diff:\n{indent_text(diff)}"
            )

        if keep_temp:
            print(f"  kept temp workspace: {workspace_root}")
            return None

        shutil.rmtree(temp_root)
        return None
    except FileNotFoundError as exc:
        return (
            f'FAIL {case["id"]}: could not execute {go_binary!r}\n'
            f"  missing executable: {exc.filename}"
        )
    except Exception as exc:  # pragma: no cover - defensive script path
        return f'FAIL {case["id"]}: {exc}\n  workspace: {temp_root / "workspace"}'


def materialize_workspace(repo_root: Path, temp_root: Path, case: dict) -> Path:
    workspace_root = temp_root / "workspace"
    workspace_root.mkdir(parents=True, exist_ok=True)
    corpus_workspace_root = (
        repo_root / "testdata" / "parity-corpus" / case["id"] / "workspace"
    )

    main_rewrites = 0
    module_cache_files: dict[tuple[str, str], list[tuple[str, str]]] = {}
    for relative_path in case["workspace_files"]:
        source_path = corpus_workspace_root / relative_path
        module_cache_path = parse_module_cache_path(relative_path)
        if module_cache_path is not None:
            module_key = (
                module_cache_path["module_path"],
                module_cache_path["version"],
            )
            module_cache_files.setdefault(module_key, []).append(
                (
                    module_cache_path["relative_path"],
                    source_path.read_text(),
                )
            )
            continue

        destination_path = workspace_root / relative_path
        destination_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_path, destination_path)
        if destination_path.suffix != ".go":
            continue

        source = destination_path.read_text()
        source, rewritten = rewrite_main_function(source)
        main_rewrites += rewritten
        source = rewrite_host_time(source, case.get("host_time_unix_millis"))
        source = rewrite_workspace_path_calls(source)
        destination_path.write_text(source)

    if main_rewrites != 1:
        raise RuntimeError(
            f'expected exactly one `func main()` in case {case["id"]}, '
            f"found {main_rewrites}"
        )

    ensure_workspace_go_mod(workspace_root)
    materialize_module_cache_files(temp_root, workspace_root, module_cache_files)
    support_path = workspace_root / "gowasm_native_go_parity_support.go"
    support_path.write_text(render_support_source(case, workspace_root))
    return workspace_root


def rewrite_main_function(source: str) -> tuple[str, int]:
    return re.subn(
        r"(?m)^func\s+main\(\)\s*\{",
        "func gowasmNativeGoUserMain() {",
        source,
        count=1,
    )


def rewrite_host_time(source: str, host_time_unix_millis: int | None) -> str:
    if host_time_unix_millis is None:
        return source
    return source.replace(
        "time.Now().UnixMilli()",
        "time.UnixMilli(gowasmNativeGoUnixMilli()).UnixMilli()",
    )


def rewrite_workspace_path_calls(source: str) -> str:
    absolute_arg_patterns = {
        r'os\.DirFS\("(/[^"]*)"\)': lambda path: f'os.DirFS(gowasmNativeGoAbs("{path}"))',
        r'os\.Setenv\("HOME",\s*"(/[^"]*)"\)': (
            lambda path: f'os.Setenv("HOME", gowasmNativeGoAbs("{path}"))'
        ),
    }
    for pattern, render in absolute_arg_patterns.items():
        source = re.sub(pattern, lambda match: render(match.group(1)), source)
    return source


def parse_module_cache_path(path: str) -> dict[str, str] | None:
    match = MODULE_CACHE_PATH_RE.match(path)
    if match is None:
        return None
    return match.groupdict()


def ensure_workspace_go_mod(workspace_root: Path) -> None:
    go_mod_path = workspace_root / "go.mod"
    if go_mod_path.exists():
        return
    go_mod_path.write_text("module paritycorpus\n\ngo 1.21\n")


def materialize_module_cache_files(
    temp_root: Path,
    workspace_root: Path,
    module_cache_files: dict[tuple[str, str], list[tuple[str, str]]],
) -> None:
    if not module_cache_files:
        return

    modules_root = temp_root / "native-go-module-cache"
    replacements = []
    for (module_path, version), files in sorted(module_cache_files.items()):
        module_root = modules_root / module_path / f"@{version}"
        for relative_path, contents in files:
            destination_path = module_root / relative_path
            destination_path.parent.mkdir(parents=True, exist_ok=True)
            destination_path.write_text(contents)

        replacements.append(
            (
                module_path,
                version,
                (Path("..") / module_root.relative_to(workspace_root.parent)).as_posix(),
            )
        )

    patch_workspace_go_mod(workspace_root, replacements)


def patch_workspace_go_mod(
    workspace_root: Path, replacements: list[tuple[str, str, str]]
) -> None:
    go_mod_path = workspace_root / "go.mod"
    if not go_mod_path.exists():
        raise RuntimeError("native-go parity replay needs a workspace go.mod for module-cache files")

    go_mod = go_mod_path.read_text()
    require_lines = []
    replace_lines = []
    for module_path, version, relative_path in replacements:
        requirement = f"\t{module_path} {version}"
        replacement = f"\t{module_path} {version} => {relative_path}"
        if requirement not in go_mod:
            require_lines.append(requirement)
        if replacement not in go_mod:
            replace_lines.append(replacement)

    blocks = []
    if require_lines:
        blocks.append("require (\n" + "\n".join(require_lines) + "\n)")
    if replace_lines:
        blocks.append("replace (\n" + "\n".join(replace_lines) + "\n)")
    if not blocks:
        return

    go_mod_path.write_text(go_mod.rstrip() + "\n\n" + "\n\n".join(blocks) + "\n")


def render_support_source(case: dict, workspace_root: Path) -> str:
    fixtures = render_fetch_fixtures(case.get("steps", []))
    host_time_unix_millis = case.get("host_time_unix_millis")
    if host_time_unix_millis is None:
        host_time_unix_millis = 0

    return f"""package main

import (
    "fmt"
    "io"
    "net/http"
    "net/url"
    "path/filepath"
    "strings"
)

type gowasmNativeGoFetchFixture struct {{
    Method string
    URL string
    ResponseStatusCode int
    ResponseStatus string
    ResponseURL string
    ResponseHeaders http.Header
    ResponseBody string
}}

var gowasmNativeGoWorkspaceRoot = {go_string(workspace_root.as_posix())}
var gowasmNativeGoHostTimeUnixMillis int64 = {host_time_unix_millis}

var gowasmNativeGoFetchFixtures = []gowasmNativeGoFetchFixture{{
{fixtures}
}}

var gowasmNativeGoNextFetch int

func main() {{
    gowasmNativeGoInstall()
    gowasmNativeGoUserMain()
    gowasmNativeGoVerifyFetchesConsumed()
}}

func gowasmNativeGoInstall() {{
    transport := gowasmNativeGoReplayTransport{{}}
    http.DefaultTransport = transport
    if http.DefaultClient != nil {{
        http.DefaultClient.Transport = transport
    }}
}}

type gowasmNativeGoReplayTransport struct{{}}

func (gowasmNativeGoReplayTransport) RoundTrip(req *http.Request) (*http.Response, error) {{
    if gowasmNativeGoNextFetch >= len(gowasmNativeGoFetchFixtures) {{
        return nil, fmt.Errorf(
            "unexpected fetch %s %s with no remaining replay fixture",
            req.Method,
            req.URL.String(),
        )
    }}

    fixture := gowasmNativeGoFetchFixtures[gowasmNativeGoNextFetch]
    gowasmNativeGoNextFetch++
    if req.Method != fixture.Method || req.URL.String() != fixture.URL {{
        return nil, fmt.Errorf(
            "unexpected fetch %s %s at replay step %d, expected %s %s",
            req.Method,
            req.URL.String(),
            gowasmNativeGoNextFetch,
            fixture.Method,
            fixture.URL,
        )
    }}

    responseURL, err := url.Parse(fixture.ResponseURL)
    if err != nil {{
        return nil, fmt.Errorf("invalid replay response url %q: %w", fixture.ResponseURL, err)
    }}

    headers := make(http.Header, len(fixture.ResponseHeaders))
    for name, values := range fixture.ResponseHeaders {{
        headers[name] = append([]string(nil), values...)
    }}

    responseRequest := req.Clone(req.Context())
    responseRequest.URL = responseURL

    return &http.Response{{
        StatusCode: fixture.ResponseStatusCode,
        Status: fixture.ResponseStatus,
        Header: headers,
        Body: io.NopCloser(strings.NewReader(fixture.ResponseBody)),
        ContentLength: int64(len(fixture.ResponseBody)),
        Request: responseRequest,
    }}, nil
}}

func gowasmNativeGoVerifyFetchesConsumed() {{
    if gowasmNativeGoNextFetch != len(gowasmNativeGoFetchFixtures) {{
        panic(fmt.Sprintf(
            "parity replay left %d unconsumed fetch fixture(s)",
            len(gowasmNativeGoFetchFixtures)-gowasmNativeGoNextFetch,
        ))
    }}
}}

func gowasmNativeGoAbs(path string) string {{
    return filepath.Join(gowasmNativeGoWorkspaceRoot, strings.TrimPrefix(path, "/"))
}}

func gowasmNativeGoUnixMilli() int64 {{
    return gowasmNativeGoHostTimeUnixMillis
}}
"""


def render_fetch_fixtures(steps: list[dict]) -> str:
    rendered = []
    for step in steps:
        if step.get("kind") != "fetch":
            raise RuntimeError(f"unsupported native-go parity step kind {step.get('kind')!r}")

        headers = step.get("response_headers", [])
        if headers:
            header_lines = []
            for header in headers:
                values = ", ".join(go_string(value) for value in header["values"])
                header_lines.append(
                    f'                {go_string(header["name"])}: []string{{{values}}},'
                )
            rendered_headers = "http.Header{\n" + "\n".join(header_lines) + "\n            }"
        else:
            rendered_headers = "http.Header{}"

        rendered.append(
            "\n".join(
                [
                    "    {",
                    f'        Method: {go_string(step["method"])},',
                    f'        URL: {go_string(step["url"])},',
                    f'        ResponseStatusCode: {int(step["response_status_code"])},',
                    f'        ResponseStatus: {go_string(step["response_status"])},',
                    f'        ResponseURL: {go_string(step["response_url"])},',
                    f"        ResponseHeaders: {rendered_headers},",
                    f'        ResponseBody: {go_string(step["response_body"])},',
                    "    },",
                ]
            )
        )
    return "\n".join(rendered)


def go_string(value: str) -> str:
    return json.dumps(value)


def indent_text(text: str) -> str:
    if not text:
        return "    <empty>"
    return "\n".join(f"    {line}" for line in text.splitlines())


if __name__ == "__main__":
    raise SystemExit(main())
