# rclean

A fast, safe Rust command-line utility for recursively removing files and directories matching glob patterns. Designed for cleaning development artifacts with multiple safety measures and performance optimizations.

## Features

- **Pattern Matching**: Include and exclude glob patterns with full wildcard support
- **Safety First**: Path traversal protection, symlink guards, confirmation prompts, and dry-run mode
- **Performance**: Metadata caching and optimized traversal (2-3x faster than v0.1.x)
- **Statistics**: Optional breakdown of deletions by pattern with size reporting
- **Configuration**: Flexible config via `.rclean.toml` or command-line arguments
- **Error Handling**: Graceful error recovery with clear diagnostics

## Safety Measures

- Safe defaults with curated pattern list
- Dry-run mode to preview deletions
- Confirmation prompts (can be skipped with `-y`)
- Path traversal protection via canonicalization (resolving symlinks, `.`, and `..` to absolute paths)
  - All subdirectories within the working directory are processed normally
  - Paths outside the working directory are blocked (e.g., `/etc/passwd` via symlink)
  - Example: If working in `/projects/myapp`, these are allowed: `/projects/myapp/subdir1`, `/projects/myapp/subdir2/nested`
  - Example: These are blocked: `/projects/other_project`, `/etc/passwd`
- Paths starting with `..` are automatically skipped
- Symlinks only removed with explicit `--include-symlinks` flag
- Broken symlinks only removed with `--remove-broken-symlinks` flag

## Installation

```sh
?
# Build and install to /usr/local/bin
make install

# Or build release binary
cargo build --release
```

## Usage

```sh
% rclean --help
Safely remove files and directories matching a set of glob patterns.

Usage: rclean [OPTIONS]

Options:
  -p, --path <PATH>               Working directory [default: .]
  -g, --glob <GLOB>               Include glob pattern(s) (can specify multiple)
  -e, --exclude <EXCLUDE>         Exclude glob pattern(s) (can specify multiple)
  -c, --configfile                Load configuration from '.rclean.toml' file
  -w, --write-configfile          Write default '.rclean.toml' file
  -d, --dry-run                   Preview deletions without removing
  -y, --skip-confirmation         Skip confirmation prompt
  -s, --stats                     Display statistics by pattern
  -i, --include-symlinks          Include matched symlinks for removal
  -b, --remove-broken-symlinks    Remove broken symlinks
  -l, --list                      List default glob patterns
  -h, --help                      Print help
  -V, --version                   Print version
```

### Examples

```bash
# Use default patterns (dry-run by default)
rclean -d

# Remove with default patterns (requires confirmation)
rclean

# Custom patterns with multiple includes
rclean -g "*.log" -g "**/*.tmp"

# Exclude specific patterns
rclean -g "*.cache" -e "**/important.cache"

# Show statistics breakdown
rclean -s

# Remove broken symlinks
rclean -b

# Skip confirmation (use with caution)
rclean -y

# Use configuration file
rclean -c
```

## Default Patterns

A curated set of safe glob patterns for common development artifacts:

```rust
pub fn get_default_patterns() -> Vec<String> {
    vec![
        // Python
        "**/__pycache__",
        "**/.mypy_cache",
        "**/.pylint_cache",
        "**/.pytest_cache",
        "**/.ruff_cache",
        "**/.coverage",
        "**/.python_history",
        "**/pip-log.txt",
        "**/.ropeproject",
        // System
        "**/.DS_Store",
        "**/.bash_history",
    ]
}
```

View the current list with `rclean --list`.

## Configuration File

Create a `.rclean.toml` file to persist your settings:

```bash
# Generate default config
rclean -w
```

Example `.rclean.toml`:

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

Use the config file with `rclean -c`.

## Recent Changes (v0.2.0)

Major release with critical bug fixes and new features:

**Critical Fixes:**
- Path traversal protection via canonicalization
- Fixed skip-confirmation logic (entries were being added after deletion)
- Fixed size double-counting for files
- Fixed dry-run flag being ignored with skip-confirmation

**Performance (2-3x faster):**
- Metadata caching eliminates redundant syscalls
- Files use instant `metadata.len()` instead of recursive traversal
- Single-pass directory traversal

**New Features:**
- Exclude patterns via `--exclude` flag
- Statistics mode via `--stats` flag
- Broken symlink removal via `--remove-broken-symlinks`
- Pattern-based reporting (count and size per pattern)

**Architecture:**
- Refactored monolithic `run()` into 7 focused helper methods
- Custom error types with graceful recovery
- All functions return `Result<T>` instead of panicking
- Improved testability and maintainability

See `CHANGELOG.md` for complete details.

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

Comprehensive test suite with 19 tests:
- 10 integration tests (dry-run, deletion, directories, patterns, symlinks, security)
- 9 unit tests (glob matching, TOML serialization)

All tests use `tempfile` for safe temporary directory creation.

## Links

- [Stack Overflow reference](https://stackoverflow.com/questions/76797185/how-to-write-a-recursive-file-directory-code-cleanup-function-in-rust)

## License

MIT
