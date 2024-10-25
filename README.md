# rclean

A simple commandline code cleanup tool in rust to recursively remove files and directories matching a list of glob patterns from a given path.

It has a number of safety measures:

- Safe defaults.
- Dry-run option.
- By default, confirmation is asked prior to removal.
- Symlinks are not removed except with permission.
- Paths which startwith ".." are skipped.
- Configuration from file can only run with permission.

## Usage

`rclean` has the following api:

```bash
% rclean --help
Safely remove files and directories matching a set of glob patterns.

Usage: rclean [OPTIONS]

Options:
  -p, --path <PATH>        Working Directory [default: .]
  -g, --glob <GLOB>        Specify custom glob pattern(s)
  -c, --configfile         Configure from '.rclean.toml' file
  -w, --write-configfile   Write default '.rclean.toml' file
  -d, --dry-run            Dry-run without actual removal
  -y, --skip-confirmation  Skip confirmation
  -i, --include-symlinks   Include matched symlinks for removal
  -l, --list               list default glob patterns
  -h, --help               Print help
  -V, --version            Print version
```

A `safe` set of glob patterns are provided by default in the code itself:

```rust
const PATTERNS: [&str;14] = [
    // directory
    "**/.coverage",
    "**/.DS_Store",
    "**/.mypy_cache",
    "**/.pylint_cache",
    "**/.pytest_cache",
    "**/.ruff_cache",
    "**/__pycache__",
    // file
    "**/.bash_history",
    "**/.python_history",
    "**/pip-log.txt",
];
```

These defaults can be overriden if `rclean` finds an `.rclean.toml` file in the local directory and the `-c` or `--configfile` option is used.

Otherwise, it is also possible to provided custom glob patterns to remove files and directories as follows:

```bash
rclean -g "*.log" -g "**/*.cache" 
```

## Devnotes

- The design follows to some extent a mature python script `clean.py` in the `scripts` folder which has been used for code cleanups. The intention is for the rust version to provide some or all of its features and provide improved preformance.

## TODO

- Should path field be removed?

- [x] Add project, or home directory-level configuration

- [ ] test on windows

  - see [remove_dir_all](https://crates.io/crates/remove_dir_all)

## Links

- Initial version referenced and improved on [stack-overflow](https://stackoverflow.com/questions/76797185/how-to-write-a-recursive-file-directory-code-cleanup-function-in-rust)

- [Vector of Actions](https://stackoverflow.com/questions/31736656/how-to-implement-a-vector-array-of-functions-in-rust-when-the-functions-co)
