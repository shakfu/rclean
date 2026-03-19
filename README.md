# drclean

A fast, safe Rust command-line utility for recursively removing files and directories matching glob patterns. Designed for cleaning development artifacts with multiple safety measures and performance optimizations.

## Features

- **Pattern Matching**: Include and exclude glob patterns with full wildcard support
- **Presets**: Named pattern groups for Python, Node.js, Rust, Java, C, Go, and more
- **Safety First**: Path traversal protection, symlink guards, confirmation prompts, and dry-run mode
- **Performance**: Metadata caching, pre-compiled glob matchers, and optimized traversal
- **Statistics**: Optional breakdown of deletions by pattern with size reporting
- **Configuration**: `.drclean.toml` with automatic discovery (upward search + global fallback)
- **JSON Output**: Machine-readable output for scripting and automation
- **Shell Completions**: Generated completions for bash, zsh, fish, elvish, powershell
- **Error Handling**: Graceful error recovery with clear diagnostics; non-zero exit on failures

## Installation

```sh
# Build and install to /usr/local/bin
make install

# Or build release binary
cargo build --release
```

## Usage

```sh
% drclean --help
Safely remove files and directories matching a set of glob patterns.

Usage: drclean [OPTIONS]

Options:
  -p, --path <PATH>               Working directory [default: .]
  -g, --glob <GLOB>               Include glob pattern(s) (can specify multiple)
  -e, --exclude <EXCLUDE>         Exclude glob pattern(s) (can specify multiple)
      --preset <PRESET>           Use a named preset (common, python, node, rust, java, c, go, all)
  -c, --configfile [PATH]         Load config (searches upward, then ~/.config/drclean/)
  -w, --write-configfile          Write default '.drclean.toml' file
  -d, --dry-run                   Preview deletions without removing
  -y, --skip-confirmation         Skip confirmation prompt
  -s, --stats                     Display statistics by pattern
  -o, --older-than <DURATION>     Only remove files older than duration (e.g., "30d", "7d", "24h")
  -P, --progress                  Show progress bar during scanning
  -i, --include-symlinks          Include matched symlinks for removal
  -b, --remove-broken-symlinks    Remove broken symlinks
  -v, --verbose                   Increase verbosity (debug-level logging)
  -q, --quiet                     Suppress all output except errors
  -l, --list                      List default glob patterns
      --completions <SHELL>       Generate shell completions (bash, zsh, fish, elvish, powershell)
      --format <FORMAT>           Output format: text (default) or json
  -h, --help                      Print help
  -V, --version                   Print version
```

### Examples

```bash
# Preview what would be deleted (dry-run)
drclean -d

# Remove with default patterns (requires confirmation)
drclean

# Custom patterns with multiple includes
drclean -g "*.log" -g "**/*.tmp"

# Exclude specific patterns
drclean -g "*.cache" -e "**/important.cache"

# Use presets for specific ecosystems
drclean --preset node
drclean --preset rust
drclean --preset python --preset common

# Combine presets with custom patterns
drclean --preset python -g "**/*.log"

# List available presets and their patterns
drclean -l --preset python

# Show statistics breakdown
drclean -s

# Only remove files older than 30 days
drclean -o 30d

# Remove broken symlinks
drclean -b

# Skip confirmation (use with caution)
drclean -y

# Use config file (auto-discovers .drclean.toml upward or ~/.config/drclean/)
drclean -c

# Use config file with CLI overrides
drclean -c --dry-run --stats

# Use explicit config file path
drclean -c configs/my-cleanup.toml

# JSON output for scripting
drclean -d --format json | jq '.summary'

# Quiet mode for scripting
drclean -y -q
```

## Presets

Named pattern groups for common ecosystems. Use `--preset` to select one or more:

| Preset   | Targets |
|----------|---------|
| `common` | `.DS_Store`, `Thumbs.db`, `*.swp`, `*~`, history files |
| `python` | `__pycache__`, `*.pyc`, `.coverage`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, etc. |
| `node`   | `node_modules`, `.next`, `.nuxt`, `.cache`, `.parcel-cache`, `coverage`, etc. |
| `rust`   | `target` |
| `java`   | `*.class`, `target`, `.gradle`, `build`, `.settings`, etc. |
| `c`      | `*.o`, `*.obj`, `*.a`, `*.lib`, `*.so`, `*.dylib`, `*.dll` |
| `go`     | `vendor` |
| `all`    | All of the above combined (deduplicated) |

Default patterns (no `--preset` or `--glob`): `common` + `python` combined.

View any preset's patterns with `drclean -l --preset <name>`.

## Configuration

### Config File

Create a `.drclean.toml` file to persist your settings:

```bash
# Generate default config
drclean -w
```

Example `.drclean.toml`:

```toml
path = "."
patterns = [
    "**/__pycache__",
    "**/*.pyc",
    "**/.DS_Store"
]
exclude_patterns = [
    "**/important/**",
    "**/keep.pyc"
]
dry_run = false
skip_confirmation = false
include_symlinks = false
remove_broken_symlinks = false
stats_mode = true
```

### Config Discovery

When you run `drclean -c` (without a path), the tool searches for configuration in this order:

1. `.drclean.toml` in the current directory, then each parent directory upward
2. `~/.config/drclean/config.toml` (global config)

You can also specify an explicit path: `drclean -c path/to/config.toml`.

CLI flags always override config file values (e.g., `drclean -c --dry-run` forces dry-run even if the config says `dry_run = false`).

### Shell Completions

Generate shell completions for your shell:

```bash
# Bash
drclean --completions bash > ~/.bash_completions/drclean

# Zsh
drclean --completions zsh > ~/.zfunc/_drclean

# Fish
drclean --completions fish > ~/.config/fish/completions/drclean.fish
```

### JSON Output

Use `--format json` for machine-readable output:

```bash
drclean -d --format json | jq '.summary'
```

The JSON output includes four sections:

- `matches` - Array of matched items with path, size, and pattern
- `summary` - Total count, size (bytes and human-readable), dry-run flag
- `stats` - Per-pattern breakdown (count, size) when `--stats` is enabled
- `failures` - Array of failed deletions with path and error message

## Safety Measures

- Safe defaults with curated pattern list
- Dry-run mode to preview deletions (`-d`)
- Confirmation prompts (skippable with `-y`)
- Path traversal protection via canonicalization
  - All paths validated to be within the working directory
  - Protects against malicious patterns like `../../etc/passwd`
- Paths starting with `..` are automatically skipped
- Symlinks only removed with explicit `--include-symlinks` flag
- Broken symlinks only removed with `--remove-broken-symlinks` flag

## Development

```bash
# Run tests
cargo test
# or
make test

# Run with clippy
cargo clippy -- -W clippy::all

# Format code
cargo fmt

# Build release
cargo build --release
```

## Testing

Comprehensive test suite with 54 tests:
- 12 integration tests (dry-run, deletion, directories, patterns, symlinks, security, age filtering)
- 10 duration parsing tests (all units, edge cases)
- 9 preset resolution tests (all presets, deduplication, unknown handling)
- 9 glob matching and TOML serialization tests
- 7 config discovery tests (upward search, global fallback, edge cases)
- 5 size formatting tests (B through TiB)
- 2 JSON output structure tests

All tests use `tempfile` for safe temporary directory creation.

## Links

- [Stack Overflow reference](https://stackoverflow.com/questions/76797185/how-to-write-a-recursive-file-directory-code-cleanup-function-in-rust)

## License

MIT
