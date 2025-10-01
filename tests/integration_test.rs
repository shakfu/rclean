use rclean::CleaningJob;
use std::fs;
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

    let mut job = CleaningJob::new(
        base_path.clone(),
        vec!["**/*.pyc".to_string()],
        vec![],  // exclude_patterns
        true,  // dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

    job.run().unwrap();

    // Files should still exist in dry-run mode
    assert!(temp_dir.path().join("test.pyc").exists());
    assert!(temp_dir.path().join("subdir/test.pyc").exists());
}

#[test]
fn test_actual_file_deletion() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let mut job = CleaningJob::new(
        base_path.clone(),
        vec!["**/*.pyc".to_string()],
        vec![],  // exclude_patterns
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

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

    let mut job = CleaningJob::new(
        base_path.clone(),
        vec!["**/__pycache__".to_string()],
        vec![],  // exclude_patterns
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

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

    let mut job = CleaningJob::new(
        base_path.clone(),
        vec!["**/*.pyc".to_string(), "**/__pycache__".to_string()],
        vec![],  // exclude_patterns
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

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

    let mut job = CleaningJob::new(
        base_path,
        vec![],
        vec![],  // exclude_patterns
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        true,  // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

    job.run().unwrap();

    // Broken symlink should be removed
    assert!(!link.exists());
}

#[test]
fn test_invalid_pattern_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let mut job = CleaningJob::new(
        base_path,
        vec!["[invalid".to_string()], // Invalid glob pattern
        vec![],  // exclude_patterns
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

    let result = job.run();
    assert!(result.is_err());
}

#[test]
fn test_size_calculation() {
    let temp_dir = create_test_structure();
    let base_path = temp_dir.path().to_str().unwrap().to_string();

    let mut job = CleaningJob::new(
        base_path,
        vec!["**/*.pyc".to_string()],
        vec![],  // exclude_patterns
        true,  // dry_run to check size without deleting
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

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

    let mut job = CleaningJob::new(
        base_path,
        vec!["../../*.txt".to_string()],
        vec![],  // exclude_patterns
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

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

    let mut job = CleaningJob::new(
        base_path.clone(),
        vec!["**/*.pyc".to_string()],
        vec!["**/subdir/*.pyc".to_string()],  // exclude subdir .pyc files
        false, // not dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        false, // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

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

    let mut job = CleaningJob::new(
        base_path,
        vec!["**/*.pyc".to_string(), "**/__pycache__".to_string()],
        vec![],  // exclude_patterns
        true,  // dry_run
        true,  // skip_confirmation
        false, // include_symlinks
        false, // remove_broken_symlinks
        true,  // stats_mode
        None,  // older_than_secs
        false, // show_progress
    );

    job.run().unwrap();

    // Statistics should be populated
    assert!(!job.stats.is_empty());

    // Should have entries for both patterns
    assert!(job.stats.contains_key("**/*.pyc") || job.stats.contains_key("**/__pycache__"));

    // Total count should match
    let total_count: i32 = job.stats.values().map(|(count, _)| count).sum();
    assert_eq!(total_count, job.counter);
}
