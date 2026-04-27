#!/usr/bin/env python3

from __future__ import annotations

import json
import pathlib
import re
import sys
from typing import Iterable


REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
STDLIB_ROOT = REPO_ROOT / "crates" / "vm" / "src"
STDLIB_DIR = STDLIB_ROOT / "stdlib"
STDLIB_METHOD_PACKAGE_MAP = {
    "BASE64_METHODS": "encoding/base64",
    "CONTEXT_METHODS": "context",
    "IO_FS_METHODS": "io/fs",
    "NET_HTTP_METHODS": "net/http",
    "NET_HTTP_REQUEST_METHODS": "net/http",
    "NET_HTTP_REQUEST_BODY_METHODS": "net/http",
    "NET_HTTP_RESPONSE_METHODS": "net/http",
    "NET_HTTP_TRANSPORT_METHODS": "net/http",
    "NET_URL_METHODS": "net/url",
    "REFLECT_METHODS": "reflect",
    "REGEXP_METHODS": "regexp",
    "STRINGS_REPLACER_METHODS": "strings",
    "SYNC_METHODS": "sync",
    "TIME_METHODS": "time",
}


def compact_name(path: pathlib.Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8")


def stdlib_source_files() -> list[pathlib.Path]:
    files = sorted(STDLIB_DIR.rglob("*.rs"))
    files.append(STDLIB_ROOT / "stdlib.rs")
    return files


def extract_rust_block(source: str, marker: str) -> str:
    start = source.index(marker)
    brace_start = source.index("{", start)
    depth = 0
    for index in range(brace_start, len(source)):
        char = source[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return source[brace_start + 1 : index]
    raise ValueError(f"unterminated block for {marker}")


def extract_enum_variants(source: str, enum_name: str) -> list[str]:
    body = extract_rust_block(source, f"pub enum {enum_name}")
    variants: list[str] = []
    depth = 0
    token = []
    for char in body:
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
        if depth == 0:
            if char.isalnum() or char == "_":
                token.append(char)
                continue
            if token:
                name = "".join(token)
                if name[0].isupper():
                    variants.append(name)
                token.clear()
        elif token:
            token.clear()
    return variants


def extract_const_u32(source: str, const_name: str) -> int:
    match = re.search(rf"pub const {const_name}: u32 = (\d+);", source)
    if not match:
        raise ValueError(f"missing const {const_name}")
    return int(match.group(1))


def parser_inventory() -> dict:
    parser_dir = REPO_ROOT / "crates" / "parser" / "src"
    source_modules = sorted(
        path.stem
        for path in parser_dir.glob("*.rs")
        if not path.stem.startswith("tests") and path.stem not in {"lib"}
    )
    test_categories = sorted(
        path.stem.removeprefix("tests_")
        for path in parser_dir.glob("tests*.rs")
        if path.stem != "tests"
    )
    return {
        "source_modules": source_modules,
        "test_categories": test_categories,
    }


def compiler_inventory() -> dict:
    compiler_dir = REPO_ROOT / "crates" / "compiler" / "src"
    lowering_modules = sorted(
        path.stem
        for path in compiler_dir.glob("*.rs")
        if not path.stem.startswith("tests")
        and path.stem not in {"lib", "test_modules"}
    )
    return {"lowering_modules": lowering_modules}


def vm_instruction_inventory() -> dict:
    instruction_source = read_text(REPO_ROOT / "crates" / "vm" / "src" / "instruction.rs")
    return {
        "compare_ops": extract_enum_variants(instruction_source, "CompareOp"),
        "select_case_op_kinds": extract_enum_variants(instruction_source, "SelectCaseOpKind"),
        "instructions": extract_enum_variants(instruction_source, "Instruction"),
    }


def extract_function_arrays() -> dict[str, list[str]]:
    arrays: dict[str, list[str]] = {}
    array_pattern = re.compile(
        r"const\s+([A-Z0-9_]+FUNCTIONS):\s*&\[\s*StdlibFunction\s*]\s*=\s*&\[(.*?)\];",
        re.DOTALL,
    )
    symbol_pattern = re.compile(r'symbol:\s*"([^"]+)"')
    for path in stdlib_source_files():
        source = read_text(path)
        for match in array_pattern.finditer(source):
            arrays[match.group(1)] = symbol_pattern.findall(match.group(2))
    return arrays


def render_method_name(receiver_type: str, method: str) -> str:
    if receiver_type.startswith("*"):
        return f"({receiver_type}).{method}"
    return f"{receiver_type}.{method}"


def extract_string_constants(source: str) -> dict[str, str]:
    pattern = re.compile(r'const\s+([A-Z0-9_]+|[A-Za-z0-9_]+):\s*&str\s*=\s*"([^"]+)";')
    return {name: value for name, value in pattern.findall(source)}


def extract_method_arrays() -> dict[str, list[str]]:
    arrays: dict[str, list[str]] = {}
    array_pattern = re.compile(
        r"const\s+([A-Z0-9_]+METHODS):\s*&\[\s*StdlibMethod\s*]\s*=\s*&\[(.*?)\];",
        re.DOTALL,
    )
    method_pattern = re.compile(
        r'receiver_type:\s*([^,\n]+),\s*method:\s*"([^"]+)"',
        re.DOTALL,
    )
    for path in stdlib_source_files():
        source = read_text(path)
        string_constants = extract_string_constants(source)
        for match in array_pattern.finditer(source):
            arrays[match.group(1)] = [
                render_method_name(
                    string_constants.get(receiver_token.strip(), receiver_token.strip().strip('"')),
                    method,
                )
                for receiver_token, method in method_pattern.findall(match.group(2))
            ]
    return arrays


def stdlib_inventory() -> dict:
    package_registry = read_text(STDLIB_DIR / "package_registry.rs")
    function_arrays = extract_function_arrays()
    method_arrays = extract_method_arrays()
    packages: dict[str, dict[str, object]] = {}
    package_pattern = re.compile(
        r'StdlibPackage\s*\{\s*name:\s*"([^"]+)",\s*functions:\s*([A-Za-z0-9_:]+),',
        re.DOTALL,
    )
    for name, function_array in package_pattern.findall(package_registry):
        array_name = function_array.split("::")[-1]
        functions = function_arrays.get(array_name, [])
        packages[name] = {
            "name": name,
            "function_count": len(functions),
            "functions": functions,
            "method_count": 0,
            "methods": [],
        }

    for array_name, package_name in STDLIB_METHOD_PACKAGE_MAP.items():
        methods = method_arrays.get(array_name, [])
        if not methods:
            continue
        package = packages.setdefault(
            package_name,
            {
                "name": package_name,
                "function_count": 0,
                "functions": [],
                "method_count": 0,
                "methods": [],
            },
        )
        package["methods"] = methods
        package["method_count"] = len(methods)

    return {"packages": list(packages.values())}


def protocol_inventory() -> dict:
    host_types = read_text(REPO_ROOT / "crates" / "host-types" / "src" / "lib.rs")
    module_protocol = read_text(
        REPO_ROOT / "crates" / "host-types" / "src" / "module_protocol.rs"
    )
    return {
        "engine_protocol_version": extract_const_u32(host_types, "ENGINE_PROTOCOL_VERSION"),
        "engine_requests": extract_enum_variants(host_types, "EngineRequest"),
        "engine_responses": extract_enum_variants(host_types, "EngineResponse"),
        "capability_requests": extract_enum_variants(host_types, "CapabilityRequest"),
        "capability_results": extract_enum_variants(host_types, "CapabilityResult"),
        "module_requests": extract_enum_variants(module_protocol, "ModuleRequest"),
        "module_results": extract_enum_variants(module_protocol, "ModuleResult"),
    }


def browser_inventory() -> dict:
    browser_files = [
        REPO_ROOT / "web" / "engine-worker.js",
        REPO_ROOT / "web" / "engine-worker-runtime.js",
        REPO_ROOT / "web" / "engine-worker-modules.js",
        REPO_ROOT / "web" / "engine-worker-module-cache.js",
    ]
    worker_message_kinds: dict[str, set[str]] = {}
    for path in browser_files:
        matches = set(re.findall(r'kind:\s*"([^"]+)"', read_text(path)))
        worker_message_kinds[compact_name(path)] = matches
    return {
        "worker_message_kinds": {
            path: sorted(kinds) for path, kinds in sorted(worker_message_kinds.items())
        }
    }


def inventory_sources() -> list[str]:
    sources = [
        "crates/compiler/src",
        "crates/host-types/src/lib.rs",
        "crates/host-types/src/module_protocol.rs",
        "crates/parser/src",
        "crates/vm/src/instruction.rs",
        "crates/vm/src/stdlib",
        "web/engine-worker.js",
        "web/engine-worker-module-cache.js",
        "web/engine-worker-modules.js",
        "web/engine-worker-runtime.js",
    ]
    return sources


def build_inventory() -> dict:
    return {
        "inventory_version": 1,
        "generated_from": inventory_sources(),
        "parser": parser_inventory(),
        "compiler": compiler_inventory(),
        "vm": vm_instruction_inventory(),
        "stdlib": stdlib_inventory(),
        "protocol": protocol_inventory(),
        "browser": browser_inventory(),
    }


def main(argv: Iterable[str]) -> int:
    if len(list(argv)) != 1:
        print("usage: generate-implementation-inventory.py", file=sys.stderr)
        return 2
    json.dump(build_inventory(), sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
