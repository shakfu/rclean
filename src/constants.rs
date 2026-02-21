// --------------------------------------------------------------------
// constants

pub const SETTINGS_FILENAME: &str = ".rclean.toml";

/// Available preset names
pub const PRESET_NAMES: &[&str] = &[
    "common", "python", "node", "rust", "java", "c", "go", "all",
];

/// Get patterns for a named preset
pub fn get_preset_patterns(name: &str) -> Option<Vec<String>> {
    let patterns: Vec<&str> = match name {
        "common" => vec![
            "**/.DS_Store",
            "**/.bash_history",
            "**/.python_history",
            "**/Thumbs.db",
            "**/*.swp",
            "**/*.swo",
            "**/*~",
        ],
        "python" => vec![
            "**/__pycache__",
            "**/.coverage",
            "**/.mypy_cache",
            "**/.pylint_cache",
            "**/.pytest_cache",
            "**/.ruff_cache",
            "**/.rumdl_cache",
            "**/.pyscn",
            "**/.ropeproject",
            "**/.python_history",
            "**/pip-log.txt",
            "**/*.pyc",
            "**/*.pyo",
            "**/*.egg-info",
            "**/dist",
        ],
        "node" => vec![
            "**/node_modules",
            "**/.next",
            "**/.nuxt",
            "**/.cache",
            "**/dist",
            "**/.parcel-cache",
            "**/.turbo",
            "**/.eslintcache",
            "**/coverage",
            "**/.nyc_output",
        ],
        "rust" => vec![
            "**/target",
        ],
        "java" => vec![
            "**/*.class",
            "**/target",
            "**/.gradle",
            "**/build",
            "**/.settings",
            "**/.classpath",
            "**/.project",
        ],
        "c" => vec![
            "**/*.o",
            "**/*.obj",
            "**/*.a",
            "**/*.lib",
            "**/*.so",
            "**/*.dylib",
            "**/*.dll",
        ],
        "go" => vec![
            "**/vendor",
        ],
        "all" => {
            let mut all = Vec::new();
            for preset in &["common", "python", "node", "rust", "java", "c", "go"] {
                if let Some(p) = get_preset_patterns(preset) {
                    for pattern in p {
                        if !all.contains(&pattern) {
                            all.push(pattern);
                        }
                    }
                }
            }
            return Some(all);
        }
        _ => return None,
    };

    Some(patterns.into_iter().map(String::from).collect())
}

/// Get the default patterns (python + common for backwards compatibility)
pub fn get_default_patterns() -> Vec<String> {
    let mut patterns = get_preset_patterns("common").unwrap_or_default();
    patterns.extend(get_preset_patterns("python").unwrap_or_default());

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    patterns.retain(|p| seen.insert(p.clone()));

    patterns
}
