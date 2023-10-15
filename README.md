# rclean

A simple commandline code cleanup script in rust to recursively remove unnecessary files and directories matching a list of glob patterns from a given path.

It has the following api:

```bash
% rclean --help
Program to cleanup non-essential files or directories

Usage: rclean [OPTIONS]

Options:
  -p, --path <PATH>        Working Directory [default: .]
  -y, --skip-confirmation  Skip confirmation
  -d, --dry-run            Dry-run without actual removal
  -g, --glob <GLOB>        Specify custom glob pattern(s)
  -l, --list               list default glob patterns
  -c, --configfile         Configure from settings file
  -h, --help               Print help
  -V, --version            Print version
```

Currently a set of glob patterns are specified in the code itself:

```rust
const PATTERNS: [&str;14] = [
    // directory
    "**/.coverage",
    "**/.DS_Store",
    // ".egg-info",
    "**/.cache",
    "**/.mypy_cache",
    "**/.pylint_cache",
    "**/.pytest_cache",
    "**/.ruff_cache",
    "**/__pycache__",
    // file
    "**/.bash_history",
    "*.log",
    "*.o",
    "*.py[co]",
    "**/.python_history",
    "**/pip-log.txt",
];
```

The design follows to some extent a mature python script `clean.py` in the `scripts` folder which has been used previously for code cleanups. The intention is for the rust version to provide some or all of its features and provide improved preformance.


## TODO

- [ ] Add project, or home directory-level configuration 

- [ ] test on windows
    - see [remove_dir_all](https://crates.io/crates/remove_dir_all)


## Links

- This was referenced and improved in a stack-overflow [question](https://stackoverflow.com/questions/76797185/how-to-write-a-recursive-file-directory-code-cleanup-function-in-rust)

- [Vector of Actions](https://stackoverflow.com/questions/31736656/how-to-implement-a-vector-array-of-functions-in-rust-when-the-functions-co)