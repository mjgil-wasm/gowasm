# Contributing to gowasm

Thank you for your interest in contributing to gowasm!

## Getting Started

1. Clone the repository
2. Ensure you have Rust 1.94.1 or later
3. Install the WebAssembly target: `rustup target add wasm32-unknown-unknown`
4. Build the project: `./scripts/build-web.sh`

## Development Workflow

### Building

```bash
# Build the browser artifact
./scripts/build-web.sh

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p gowasm-compiler
```

### Running the Browser Shell

```bash
./run.sh
```

Or serve manually:

```bash
cd web
python3 -m http.server 8000
```

Then open `http://127.0.0.1:8000/`.

### Release Gate

Before submitting changes, run the release gate:

```bash
bash ./scripts/check-release-gate.sh \
  --browser-worker-command 'bash ./scripts/check-browser-worker.sh' \
  --browser-shell-command 'bash ./scripts/check-browser-shell.sh'
```

This runs:
- Compile checks
- Release artifact reproducibility
- Unit tests
- Differential corpora
- Fuzz/property tests
- Browser worker tests
- Browser shell tests
- Performance capture
- Performance budgets

## Code Standards

- Run `cargo fmt` before committing
- Ensure all tests pass: `cargo test --workspace`
- Update documentation when changing support surface
- Add tests when adding or modifying features

## Support Surface Changes

When adding new supported features, update the relevant contract document in
`docs/` and add tests in the same change.

## License

By contributing, you agree that your contributions will be licensed under the Apache License, Version 2.0.
