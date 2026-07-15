# healthpulse

Code health and technical debt analyzer — scans repositories for complexity, style, and maintainability issues.

## What It Does

healthpulse scans your codebase and produces a health report covering:

- **Complexity analysis** — Estimates cyclomatic complexity from control flow statements
- **File length** — Flags files that exceed configurable line limits
- **Function length** — Detects overly long functions
- **Test ratio** — Warns when test file coverage is low relative to source files

## Quick Start

```bash
# Install
cargo install --path .

# Scan current directory
healthpulse

# Scan with custom limits
healthpulse --path ./my-project --max-complexity 8 --max-function-length 40

# Output as JSON
healthpulse --format json --output report.json

# Output as Markdown
healthpulse --format markdown --output health.md

# Use a config file
healthpulse --config healthpulse.json

# Ignore specific paths
healthpulse --ignore vendor --ignore node_modules
```

## Configuration

Create a `healthpulse.json` config file:

```json
{
    "max_complexity": 10,
    "max_function_length": 50,
    "max_file_length": 500,
    "min_test_ratio": 0.3,
    "ignore": ["vendor", "third_party"]
}
```

Or use CLI flags:

| Flag | Default | Description |
|------|---------|-------------|
| `--path` | `.` | Path to scan |
| `--format` | `text` | Output format: text, json, markdown |
| `--max-complexity` | `10` | Maximum allowed cyclomatic complexity |
| `--max-function-length` | `50` | Maximum function length in lines |
| `--max-file-length` | `500` | Maximum file length in lines |
| `--min-test-ratio` | `0.3` | Minimum test file ratio (0.0–1.0) |
| `--config` | | Config file path |
| `--ignore` | | Ignore patterns (repeatable) |

## Exit Codes

- `0` — No critical issues found
- `1` — Error (e.g., config file not found)
- `2` — Critical issues found

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Check code health
  run: |
    cargo install --path .
    healthpulse --format json --max-complexity 15 || exit 2
```

## Supported Languages

The complexity analysis works best with Go code (function detection via `func ` prefix), but file length and test ratio checks work for any language.

## License

MIT