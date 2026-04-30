#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import socketserver
import subprocess
import tempfile
import threading
import time
from functools import partial
from http.server import SimpleHTTPRequestHandler
from pathlib import Path
from urllib.parse import parse_qs, urlparse


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Serve the repo over localhost, run a checked browser page in "
            "headless Chrome, and extract one element's rendered text."
        )
    )
    parser.add_argument("--page", required=True, help="Repo-relative page path to open.")
    parser.add_argument(
        "--page-query",
        help="Optional raw query string appended when opening the page.",
    )
    parser.add_argument(
        "--element-id",
        required=True,
        help="DOM element id whose rendered text should be extracted.",
    )
    parser.add_argument(
        "--expect-substring",
        action="append",
        default=[],
        help="Substring that must appear in the extracted element text.",
    )
    parser.add_argument(
        "--reject-substring",
        action="append",
        default=[],
        help="Substring that must not appear in the extracted element text.",
    )
    parser.add_argument(
        "--output",
        help="Optional path that receives the extracted element text.",
    )
    parser.add_argument(
        "--artifact-element-id",
        help="Optional secondary DOM element id reported through the CI callback path.",
    )
    parser.add_argument(
        "--artifact-output",
        help="Optional path that receives the secondary artifact element text.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=120,
        help="Browser timeout and virtual time budget in seconds.",
    )
    parser.add_argument(
        "--browser-binary",
        help="Explicit browser binary. Defaults to $GOWASM_CHROME_BIN or google-chrome.",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def resolve_browser_binary(explicit: str | None) -> str:
    if explicit:
        return explicit
    env_value = os.environ.get("GOWASM_CHROME_BIN")
    if env_value:
        return env_value
    discovered = shutil.which("google-chrome") or shutil.which("chromium") or shutil.which(
        "chromium-browser"
    )
    if discovered:
        return discovered
    raise SystemExit(
        "could not find a Chromium-derived browser; pass --browser-binary or set GOWASM_CHROME_BIN"
    )


class CompletionState:
    def __init__(self) -> None:
        self.condition = threading.Condition()
        self.text_by_element_id: dict[str, str] = {}

    def store(self, element_id: str, text: str) -> None:
        with self.condition:
            self.text_by_element_id[element_id] = text
            self.condition.notify_all()

    def wait_for(self, element_id: str, timeout_seconds: int) -> str:
        deadline = time.monotonic() + timeout_seconds
        with self.condition:
            while element_id not in self.text_by_element_id:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError(element_id)
                self.condition.wait(timeout=remaining)
            return self.text_by_element_id[element_id]

    def get(self, element_id: str) -> str | None:
        with self.condition:
            return self.text_by_element_id.get(element_id)


class QuietHttpHandler(SimpleHTTPRequestHandler):
    completion_state: CompletionState

    def log_message(self, format: str, *args: object) -> None:
        del format, args

    def copyfile(self, source, outputfile) -> None:  # type: ignore[override]
        try:
            super().copyfile(source, outputfile)
        except (BrokenPipeError, ConnectionResetError):
            return

    def do_POST(self) -> None:  # type: ignore[override]
        if self.path != "/__gowasm_ci_complete":
            self.send_error(404)
            return

        content_length = int(self.headers.get("Content-Length", "0"))
        payload = json.loads(self.rfile.read(content_length) or b"{}")
        element_id = payload.get("elementId")
        text = payload.get("text")
        if not isinstance(element_id, str) or not element_id:
            self.send_error(400, "completion payload should include non-empty elementId")
            return
        if not isinstance(text, str) or not text:
            self.send_error(400, "completion payload should include non-empty text")
            return

        self.completion_state.store(element_id, text)
        self.send_response(204)
        self.end_headers()

    def do_GET(self) -> None:  # type: ignore[override]
        parsed = urlparse(self.path)
        if parsed.path != "/__gowasm_test_delay":
            super().do_GET()
            return

        query = parse_qs(parsed.query)
        delay_millis = clamp_delay_millis(query.get("ms", ["0"])[0])
        status_code = clamp_http_status(query.get("status", ["200"])[0])
        body = query.get("body", [""])[0]
        content_type = query.get("content_type", ["text/plain; charset=utf-8"])[0]

        time.sleep(delay_millis / 1000)
        encoded_body = body.encode("utf-8")
        self.send_response(status_code)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(encoded_body)))
        self.end_headers()
        try:
            self.wfile.write(encoded_body)
        except (BrokenPipeError, ConnectionResetError):
            return


def clamp_delay_millis(raw_value: str) -> int:
    try:
        numeric = int(raw_value)
    except (TypeError, ValueError):
        return 0
    return max(0, min(numeric, 120_000))


def clamp_http_status(raw_value: str) -> int:
    try:
        numeric = int(raw_value)
    except (TypeError, ValueError):
        return 200
    if 100 <= numeric <= 599:
        return numeric
    return 200


def serve_repo(
    root: Path, completion_state: CompletionState
) -> tuple[socketserver.TCPServer, threading.Thread]:
    class ThreadedTcpServer(socketserver.ThreadingTCPServer):
        allow_reuse_address = True

    class Handler(QuietHttpHandler):
        pass

    Handler.completion_state = completion_state
    handler = partial(Handler, directory=str(root))
    server = ThreadedTcpServer(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, thread


def browser_arguments(
    browser_binary: str, url: str, user_data_dir: Path
) -> list[str]:
    arguments = [
        browser_binary,
        "--headless=new",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--enable-precise-memory-info",
        "--hide-scrollbars",
        "--mute-audio",
        "--no-first-run",
        "--no-default-browser-check",
        "--remote-debugging-port=0",
        f"--user-data-dir={user_data_dir}",
        "--window-size=1440,1200",
        url,
    ]
    if hasattr(os, "geteuid") and os.geteuid() == 0:
        arguments.insert(1, "--no-sandbox")
    return arguments


def wait_for_completion(
    browser_binary: str,
    url: str,
    timeout_seconds: int,
    completion_state: CompletionState,
    primary_element_id: str,
) -> str:
    with tempfile.TemporaryDirectory(prefix="gowasm-browser-page-runner-") as temp_dir:
        process = subprocess.Popen(
            browser_arguments(browser_binary, url, Path(temp_dir)),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            try:
                text = completion_state.wait_for(primary_element_id, timeout_seconds)
            except TimeoutError:
                process.kill()
                stdout, stderr = process.communicate(timeout=10)
                raise SystemExit(
                    "timed out waiting for browser page completion callback\n"
                    f"stdout:\n{stdout}\nstderr:\n{stderr}"
                )
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
            process.communicate(timeout=10)

    return text


def write_output(path: str | None, text: str) -> None:
    if not path:
        return
    output_path = Path(path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(f"{text}\n", encoding="utf-8")


def validate_text(text: str, expect_substrings: list[str], reject_substrings: list[str]) -> None:
    for substring in expect_substrings:
        if substring not in text:
            raise SystemExit(
                f"rendered element text did not contain required substring {substring!r}\n{text}"
            )
    for substring in reject_substrings:
        if substring in text:
            raise SystemExit(
                f"rendered element text unexpectedly contained substring {substring!r}\n{text}"
            )


def main() -> int:
    args = parse_args()
    if bool(args.artifact_element_id) != bool(args.artifact_output):
        raise SystemExit(
            "--artifact-element-id and --artifact-output must be provided together"
        )

    root = repo_root()
    page_path = root / args.page
    if not page_path.is_file():
        raise SystemExit(f"page not found: {page_path}")

    browser_binary = resolve_browser_binary(args.browser_binary)
    completion_state = CompletionState()
    server, thread = serve_repo(root, completion_state)
    try:
        query = "ci=1"
        if args.page_query:
            query = f"{query}&{args.page_query}"
        url = f"http://127.0.0.1:{server.server_address[1]}/{args.page}?{query}"
        text = wait_for_completion(
            browser_binary,
            url,
            args.timeout_seconds,
            completion_state,
            args.element_id,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

    artifact_text = None
    if args.artifact_element_id:
        artifact_text = completion_state.get(args.artifact_element_id)
        if artifact_text is None:
            raise SystemExit(
                f"browser page completed without reporting artifact element {args.artifact_element_id!r}"
            )

    write_output(args.output, text)
    write_output(args.artifact_output, artifact_text)
    validate_text(text, args.expect_substring, args.reject_substring)
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
