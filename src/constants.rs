// --------------------------------------------------------------------
// constants

pub const SETTINGS_FILENAME: &str = ".rclean.toml";

pub fn get_default_patterns() -> Vec<String> {
    vec![
        // directory
        String::from("**/__pycache__"),
        String::from("**/.coverage"),
        String::from("**/.DS_Store"),
        String::from("**/.mypy_cache"),
        String::from("**/.pylint_cache"),
        String::from("**/.pytest_cache"),
        String::from("**/.ruff_cache"),
        String::from("**/.rumdl_cache"),
        String::from("**/.pyscn"),
        // file
        String::from("**/.bash_history"),
        String::from("**/.python_history"),
        String::from("**/pip-log.txt"),
        String::from("**/.ropeproject"),
    ]
}
