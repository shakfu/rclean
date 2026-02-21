use rclean::constants::{get_default_patterns, get_preset_patterns, PRESET_NAMES};

#[test]
fn test_all_preset_names_resolve() {
    for name in PRESET_NAMES {
        let patterns = get_preset_patterns(name);
        assert!(
            patterns.is_some(),
            "Preset '{}' should return Some patterns",
            name
        );
        assert!(
            !patterns.unwrap().is_empty(),
            "Preset '{}' should have at least one pattern",
            name
        );
    }
}

#[test]
fn test_unknown_preset_returns_none() {
    assert!(get_preset_patterns("nonexistent").is_none());
    assert!(get_preset_patterns("").is_none());
}

#[test]
fn test_python_preset_contains_pycache() {
    let patterns = get_preset_patterns("python").unwrap();
    assert!(patterns.contains(&"**/__pycache__".to_string()));
    assert!(patterns.contains(&"**/*.pyc".to_string()));
}

#[test]
fn test_node_preset_contains_node_modules() {
    let patterns = get_preset_patterns("node").unwrap();
    assert!(patterns.contains(&"**/node_modules".to_string()));
}

#[test]
fn test_rust_preset_contains_target() {
    let patterns = get_preset_patterns("rust").unwrap();
    assert!(patterns.contains(&"**/target".to_string()));
}

#[test]
fn test_all_preset_includes_other_presets() {
    let all = get_preset_patterns("all").unwrap();

    // Should contain patterns from multiple presets
    assert!(all.contains(&"**/__pycache__".to_string())); // python
    assert!(all.contains(&"**/node_modules".to_string())); // node
    assert!(all.contains(&"**/target".to_string())); // rust
    assert!(all.contains(&"**/.DS_Store".to_string())); // common
}

#[test]
fn test_all_preset_no_duplicates() {
    let all = get_preset_patterns("all").unwrap();
    let mut seen = std::collections::HashSet::new();
    for pattern in &all {
        assert!(
            seen.insert(pattern),
            "Duplicate pattern in 'all' preset: {}",
            pattern
        );
    }
}

#[test]
fn test_default_patterns_not_empty() {
    let defaults = get_default_patterns();
    assert!(!defaults.is_empty());
}

#[test]
fn test_default_patterns_no_duplicates() {
    let defaults = get_default_patterns();
    let mut seen = std::collections::HashSet::new();
    for pattern in &defaults {
        assert!(
            seen.insert(pattern),
            "Duplicate pattern in defaults: {}",
            pattern
        );
    }
}
