use rclean::{CleanConfig, CleaningJob};
use std::fs;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Helper function to create a temporary directory structure for testing
fn create_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create test files
    fs::write(base.join("test.txt"), "test content").unwrap();
    fs::write(base.join("test.pyc"), "compiled python").unwrap();
    fs::write(base.join("important.log"), "keep this").unwrap();

    // Create __pycache__ directory
    let pycache = base.join("__pycache__");
    fs::create_dir(&pycache).unwrap();
    fs::write(pycache.join("module.pyc"), "cached").unwrap();
    fs::write(pycache.join("another.pyc"), "cached2").unwrap();

    // Create nested directory structure
    let subdir = base.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("test.pyc"), "nested pyc").unwrap();

    temp_dir
}

#[test]
fn test_dry_run_does_not_delete() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/*.pyc".to_string()])
        .dry_run(true)
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // Files should still exist in dry-run mode
    assert!(temp_dir.path().join("test.pyc").exists());
    assert!(temp_dir.path().join("subdir/test.pyc").exists());
}

#[test]
fn test_actual_file_deletion() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/*.pyc".to_string()])
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // .pyc files should be deleted
    assert!(!temp_dir.path().join("test.pyc").exists());
    assert!(!temp_dir.path().join("subdir/test.pyc").exists());

    // Other files should remain
    assert!(temp_dir.path().join("test.txt").exists());
    assert!(temp_dir.path().join("important.log").exists());
}

#[test]
fn test_directory_deletion() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/__pycache__".to_string()])
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // __pycache__ directory should be deleted entirely
    assert!(!temp_dir.path().join("__pycache__").exists());

    // Other files should remain
    assert!(temp_dir.path().join("test.txt").exists());
    assert!(temp_dir.path().join("test.pyc").exists());
}

#[test]
fn test_multiple_patterns() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec![
            "**/*.pyc".to_string(),
            "**/__pycache__".to_string(),
        ])
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // Both .pyc files and __pycache__ directory should be deleted
    assert!(!temp_dir.path().join("test.pyc").exists());
    assert!(!temp_dir.path().join("__pycache__").exists());
    assert!(!temp_dir.path().join("subdir/test.pyc").exists());

    // Other files should remain
    assert!(temp_dir.path().join("test.txt").exists());
    assert!(temp_dir.path().join("important.log").exists());
}

#[test]
fn test_broken_symlink_removal() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create a file and a symlink to it
    let target = base.join("target.txt");
    fs::write(&target, "content").unwrap();
    let link = base.join("link.txt");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target, &link).unwrap();

    // Remove the target to break the symlink
    fs::remove_file(&target).unwrap();

    let base_path = base.to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .remove_broken_symlinks(true)
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // Broken symlink should be removed
    assert!(!link.exists());
}

#[test]
fn test_invalid_pattern_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["[invalid".to_string()])
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);

    let result = job.run();
    assert!(result.is_err());
}

#[test]
fn test_size_calculation() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/*.pyc".to_string()])
        .dry_run(true)
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // Size should be greater than 0 since we have .pyc files
    assert!(job.size > 0);
    // Counter should match the number of .pyc files created
    // test.pyc (root), __pycache__/module.pyc, __pycache__/another.pyc, subdir/test.pyc
    assert_eq!(job.counter, 4);
}

#[test]
fn test_path_traversal_protection() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create a file we should NOT be able to delete
    let outside_file = base.parent().unwrap().join("outside.txt");
    fs::write(&outside_file, "protected").unwrap();

    // Try to use a pattern that would match outside the base directory
    let base_path = base.to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["../../*.txt".to_string()])
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);

    // Run should succeed but not delete files outside base directory
    job.run().unwrap();

    // File outside should still exist (protected by canonicalization check)
    assert!(outside_file.exists());

    // Cleanup
    fs::remove_file(outside_file).unwrap();
}

#[test]
fn test_exclude_patterns() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/*.pyc".to_string()])
        .exclude_patterns(vec!["**/subdir/*.pyc".to_string()])
        .skip_confirmation(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // .pyc files in root should be deleted
    assert!(!temp_dir.path().join("test.pyc").exists());

    // .pyc files in subdir should be excluded
    assert!(temp_dir.path().join("subdir/test.pyc").exists());
}

#[test]
fn test_stats_mode() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec![
            "**/*.pyc".to_string(),
            "**/__pycache__".to_string(),
        ])
        .dry_run(true)
        .skip_confirmation(true)
        .stats_mode(true)
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // Statistics should be populated
    assert!(!job.stats.is_empty());

    // Should have entries for both patterns
    assert!(job.stats.contains_key("**/*.pyc") || job.stats.contains_key("**/__pycache__"));

    // Total count should match
    let total_count: usize = job.stats.values().map(|(count, _)| count).sum();
    assert_eq!(total_count, job.counter);
}

#[test]
fn test_older_than_skips_recent_files() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create a file (will have current timestamp)
    fs::write(base.join("recent.pyc"), "recent content").unwrap();

    let base_path = base.to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/*.pyc".to_string()])
        .dry_run(true)
        .skip_confirmation(true)
        .older_than_secs(Some(3600))
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // File was just created, so it should be skipped (too recent)
    assert_eq!(job.counter, 0);
}

#[test]
fn test_older_than_matches_old_files() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();

    // Create a file and backdate its modification time
    let old_file = base.join("old.pyc");
    fs::write(&old_file, "old content").unwrap();

    // Set modification time to 2 hours ago
    let two_hours_ago = SystemTime::now() - Duration::from_secs(7200);
    filetime::set_file_mtime(
        &old_file,
        filetime::FileTime::from_system_time(two_hours_ago),
    )
    .unwrap();

    let base_path = base.to_str().unwrap().to_string();

    let config = CleanConfig::builder()
        .path(base_path)
        .patterns(vec!["**/*.pyc".to_string()])
        .dry_run(true)
        .skip_confirmation(true)
        .older_than_secs(Some(3600))
        .build();
    let mut job = CleaningJob::new(config);
    job.run().unwrap();

    // File is 2 hours old, threshold is 1 hour, so it should be matched
    assert_eq!(job.counter, 1);
}
