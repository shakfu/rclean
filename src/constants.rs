// --------------------------------------------------------------------
// constants

pub const SETTINGS_FILENAME: &str = ".rclean.toml";

/// list of glob patterns of files / directories to remove.
pub const PATTERNS: [&str; 10] = [
    // directory
    "**/__pycache__",
    "**/.coverage",
    "**/.DS_Store",
    "**/.mypy_cache",
    "**/.pylint_cache",
    "**/.pytest_cache",
    "**/.ruff_cache",
    // file
    "**/.bash_history",
    "**/.python_history",
    "**/pip-log.txt",
];
