# healthpulse — AGENTS.md

## Build & Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run tests in release mode
cargo test --release

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## Project Structure

- `src/main.rs` — CLI entry point using clap
- `src/config.rs` — Configuration management (JSON config file, CLI overrides)
- `src/analyzer.rs` — Core analysis engine (file scanning, complexity estimation, issue detection)
- `src/output.rs` — Output formatters (text, JSON, markdown)
- `tests/integration.rs` — Integration tests

## Key Design Decisions

- **Language detection via heuristics** — Function detection uses `func ` prefix (Go-style), file length is language-agnostic
- **Cyclomatic complexity estimation** — Simple count of `if`, `for`, `switch`, `case`, `&&`, `||` statements as a proxy for complexity
- **WalkDir-based scanning** — Uses `walkdir` for recursive directory traversal with skip logic for common non-code dirs (`.git`, `node_modules`, etc.)
- **Zero-config defaults** — Sensible defaults that match common developer expectations
- **Exit code signaling** — Returns `2` when critical issues found, `0` when clean

## Adding a New Check

1. Add a new variant to `Category` enum in `src/analyzer.rs`
2. Add detection logic in `analyze_file()` function
3. Add a new test in `tests/integration.rs`
4. Update the category in the output formatters

## Adding a New Output Format

1. Add a new variant to `Output` enum in `src/output.rs`
2. Implement the `render_*` method
3. Update the match in `main.rs`

## Testing

Run all tests: `cargo test --all`

Each test creates a temporary directory under `/tmp/` and cleans up after itself.