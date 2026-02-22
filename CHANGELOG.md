# CHANGELOG

All notable project-wide changes will be documented in this file. Note that each subproject has its own CHANGELOG.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Commons Changelog](https://common-changelog.org). This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Types of Changes

- Added: for new features.
- Changed: for changes in existing functionality.
- Deprecated: for soon-to-be removed features.
- Removed: for now removed features.
- Fixed: for any bug fixes.
- Security: in case of vulnerabilities.

---

## [0.2.2]

### Fixed

- **Critical: Spurious ENOENT failures when directory and child patterns overlap**: When glob patterns matched both a directory (e.g., `**/__pycache__`) and files inside it (e.g., `**/*.pyc`), `remove_dir_all` on the directory would recursively delete all children. Subsequent attempts to delete those same children individually produced hundreds of "No such file or directory" errors. `execute_deletion()` now tracks which directories have been recursively removed and skips any targets that are descendants of an already-deleted directory.

### Added

- **Regression test**: `test_no_failures_when_dir_and_children_both_match` covering the overlapping directory/child pattern scenario (55 tests total)

---

## [0.2.1]

### Added

- **Builder Pattern**: `CleanConfig::builder()` fluent API replaces direct struct construction
  - All fields configurable via chained methods (e.g., `.path(".")..dry_run(true).build()`)
  - Eliminates need for `#[allow(clippy::too_many_arguments)]`

- **Config Discovery**: `rclean -c` (without a path) now searches for config automatically
  - Searches upward from the current directory for `.rclean.toml`
  - Falls back to global config at `~/.config/rclean/config.toml`
  - Explicit path still supported: `rclean -c path/to/config.toml`
  - New public functions: `find_config_upward()`, `global_config_path()`, `discover_config()`

- **CLI Flag Overrides**: When using `-c`, CLI flags now override config file values
  - `--dry-run`, `--stats`, `--progress`, `--exclude`, `--older-than`, etc.
  - Example: `rclean -c --dry-run` forces dry-run even if config says `dry_run = false`

- **Pattern Presets**: Named pattern groups via `--preset` flag
  - Available presets: `common`, `python`, `node`, `rust`, `java`, `c`, `go`, `all`
  - Combinable: `--preset python --preset node`
  - Combinable with custom patterns: `--preset python -g "**/*.log"`
  - List preset contents: `rclean -l --preset python`

- **JSON Output**: `--format json` for machine-readable structured output
  - Includes `matches`, `summary`, `stats`, and `failures` sections
  - Human-readable sizes in summary and stats
  - Suppresses text logging when active

- **Shell Completions**: `--completions <SHELL>` generates completions
  - Supports bash, zsh, fish, elvish, powershell
  - Uses `clap_complete` crate

- **Human-Readable Sizes**: All size output now uses IEC binary units
  - Format: B, KiB, MiB, GiB, TiB
  - Applied to statistics, summary, and JSON output
  - Public `format_size()` function available in library API

- **Verbose/Quiet Modes**: `--verbose` (`-v`) and `--quiet` (`-q`) flags
  - Verbose enables debug-level logging
  - Quiet suppresses all output except errors

- **New Dependencies**: `clap_complete`, `serde_json`, `dirs`

- **New Tests** (54 total, up from 19):
  - 7 config discovery tests (upward search, global fallback, edge cases)
  - 5 size formatting tests (B through TiB)
  - 10 duration parsing tests (all units, edge cases)
  - 9 preset resolution tests (all presets, deduplication, unknown handling)
  - 2 JSON output tests (structure, empty results)
  - 2 age-based filtering integration tests

### Changed

- **Architecture**: Split `CleaningJob` into `CleanConfig` (serializable) + `CleaningJob` (runtime)
  - `CleanConfig` holds all configuration with serde support
  - `CleaningJob` holds runtime state (targets, stats, counters)
  - Clean separation of concerns

- **Pre-compiled Matchers**: Glob patterns compiled once in `build_globsets()`
  - `PatternMatchers` type alias: `Vec<(String, GlobMatcher)>`
  - Eliminates per-entry glob recompilation for stats attribution

- **Progress Bar + Output**: Progress bar no longer suppresses `--stats` or match logging
  - Uses `pb.println()` to interleave output with spinner
  - All match/exclude/delete messages route through progress-aware logging

- **Counter Type**: Changed `counter` from `i32` to `usize`, stats values from `(i32, u64)` to `(usize, u64)`

- **Memory**: `execute_deletion()` uses `std::mem::take` instead of `.clone()` for targets

- **Config File Path**: `--configfile` (`-c`) now accepts optional path argument
  - `-c` alone triggers config discovery
  - `-c path/to/config.toml` uses specified config file

- **Non-Zero Exit Code**: Process exits with code 1 when any deletions fail

- **Default Patterns**: Now defined as `common` + `python` presets combined (deduplicated)

### Fixed

- **Dry-run Default Confusion**: `CleanConfig::default()` now sets `dry_run = false`
  - Previously inconsistent between Default impl and CLI behavior
  - CLI `-d` flag still works as expected for explicit dry-run

- **License Contradiction**: Fixed README stating "Unlicense" while Cargo.toml and LICENSE file specified MIT
  - README now correctly states MIT

- **Makefile**: Added `test` target (`cargo test`)

---

## [0.2.0]

### Added

- **Progress Bar**: New `--progress` (`-P`) flag to show real-time scanning progress
  - Displays elapsed time and items scanned
  - Updates every 100 items for efficiency, and doesn't display INFO log
  - Shows final summary on completion
  - Uses `indicatif` crate for smooth spinner animation

- **Age-Based Filtering**: New `--older-than` (`-o`) flag to only remove old files
  - Example: `rclean -g "*.log" --older-than "30d"`
  - Supports time units: s (seconds), m (minutes), h (hours), d (days), w (weeks)
  - Checks file modification time against threshold
  - Skips files newer than specified duration

- **Error Tracking**: Failed deletions are now tracked and reported
  - `failed_deletions` field tracks all deletion errors
  - Error summary displayed at end of operation
  - Shows path and error message for each failure
  - Helps identify permission issues and locked files

- **Exclude Patterns Feature**: New `--exclude` (`-e`) flag to skip files matching specific patterns
  - Example: `rclean -g "*.log" --exclude "important.log"`
  - Supports multiple exclude patterns
  - Works with config file via `exclude_patterns` field

- **Statistics Mode**: New `--stats` (`-s`) flag to display breakdown of matches by pattern
  - Shows count and total size for each pattern
  - Sorted by match count (descending)
  - Useful for understanding what's being cleaned

- **Custom Error Handling**: Comprehensive `CleanError` enum for better error messages
  - IoError, GlobError, PathTraversal, PermissionDenied, ConfigError variants
  - Proper `Display` and `Error` trait implementations
  - Graceful error propagation throughout codebase

- **Integration Tests**: 10 comprehensive integration tests using tempfile
  - Tests for dry-run, actual deletion, directory removal, multiple patterns
  - Tests for broken symlinks, invalid patterns, size calculation
  - Tests for path traversal protection, exclude patterns, statistics mode

- **Dependencies**: Added `indicatif = "0.17"` for progress bars, `tempfile = "3"` for testing

### Changed

- **Major Refactoring**: Extracted `run()` method into focused helper methods
  - `build_globsets()` - Constructs include and exclude glob matchers
  - `should_process()` - Path validation and security checks
  - `find_matching_pattern()` - Pattern identification for statistics
  - `collect_targets()` - Directory traversal and matching
  - `handle_matched_entry()` - Entry processing with stats tracking
  - `execute_deletion()` - Batch file removal
  - `display_stats()` - Statistics reporting
  - Reduced `run()` from 78 lines to 44 lines (43% reduction)

- **Performance Optimization**: Changed `targets` from `Vec<DirEntry>` to `Vec<(PathBuf, Metadata)>`
  - Metadata cached during collection, eliminating redundant syscalls
  - Files use direct `metadata.len()` instead of `get_size()` for instant calculation
  - ~2-3x performance improvement for large directory trees

- **CleaningJob Constructor**: Now accepts `exclude_patterns` and `stats_mode` parameters
  - Note: This is a breaking change to the API

- **Error Handling**: All functions now return `Result<T>` instead of panicking
  - Main function exits cleanly with exit code 1 on errors
  - Removed all `unwrap()` and `expect()` calls in favor of proper error handling

### Fixed

- **Critical: Skip Confirmation Logic**: Fixed bug where entries were added to targets after deletion
  - When `skip_confirmation` was true, code would delete entries AND add them to targets list
  - Now properly uses `if/else` blocks to prevent double-processing
  - Same fix applied to broken symlink removal flow

- **Critical: Size Double-Counting**: Fixed inflated size calculations
  - Previously used `get_size()` for all entries, causing recursive counting
  - Now uses `metadata.len()` for files and `get_size()` only for directories
  - Eliminates 2-10x size inflation for directory-heavy cleanups

- **Critical: Test Schema Mismatch**: Fixed test struct missing `include_symlinks` and `remove_broken_symlinks` fields
  - Test deserialization now works correctly with real config files

- **Dry-Run Bug**: Fixed `dry_run` being ignored when `skip_confirmation` is true
  - Now properly checks both flags before deletion

- **Broken Symlink Counter**: Fixed counter not being incremented for broken symlinks

- **Duplicate Code**: Optimized `remove_entry()` to eliminate duplicate removal logic
  - Combined file and symlink removal (both use `remove_file()`)

- **Typo**: Fixed "deerialize" -> "deserialize" in error message

### Security

- **Path Traversal Protection**: Added canonicalization checks to prevent directory traversal attacks
  - Base path is canonicalized at start of operation
  - All entry paths are validated to be within base directory
  - Paths outside working directory are skipped with warnings
  - Protects against malicious patterns like `../../etc/passwd`

- **Config File Safety**: Improved validation of `.rclean.toml` contents
  - Better error messages for malformed configs
  - Pattern validation before execution

---

## [0.1.x]

- updated dependencies

## [0.1.3]

- Update dependencies and added `.ropeproject` as a cleanup pattern

## [0.1.2]

- Fixed size reporting which was faulty in earlier versions.

## [0.1.1]

- Added initial rclean code
