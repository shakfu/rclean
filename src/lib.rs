pub mod constants;

use dialoguer::Confirm;
use fs_extra::dir::get_size;
use globset::{Glob, GlobSet, GlobSetBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use logging_timer::time;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

// --------------------------------------------------------------------
// error types

/// Custom error type for cleaning operations
#[derive(Debug)]
pub enum CleanError {
    IoError(std::io::Error),
    GlobError(globset::Error),
    PathTraversal(PathBuf),
    PermissionDenied(PathBuf),
    ConfigError(String),
}

impl std::fmt::Display for CleanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CleanError::IoError(e) => write!(f, "IO error: {}", e),
            CleanError::GlobError(e) => write!(f, "Glob pattern error: {}", e),
            CleanError::PathTraversal(p) => write!(f, "Path traversal detected: {:?}", p),
            CleanError::PermissionDenied(p) => write!(f, "Permission denied: {:?}", p),
            CleanError::ConfigError(s) => write!(f, "Configuration error: {}", s),
        }
    }
}

impl std::error::Error for CleanError {}

impl From<std::io::Error> for CleanError {
    fn from(err: std::io::Error) -> Self {
        CleanError::IoError(err)
    }
}

impl From<globset::Error> for CleanError {
    fn from(err: globset::Error) -> Self {
        CleanError::GlobError(err)
    }
}

pub type Result<T> = std::result::Result<T, CleanError>;

// --------------------------------------------------------------------
// core

/// Main configuration object for cleaning jobs with partial
/// with selective (de)serialization
#[derive(Serialize, Deserialize)]
pub struct CleaningJob {
    pub path: String,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    pub dry_run: bool,
    pub skip_confirmation: bool,
    pub include_symlinks: bool,
    pub remove_broken_symlinks: bool,
    #[serde(default)]
    pub stats_mode: bool,
    #[serde(default)]
    pub older_than_secs: Option<u64>,
    #[serde(default)]
    pub show_progress: bool,
    #[serde(skip_serializing, skip_deserializing)]
    targets: Vec<(PathBuf, Metadata)>,
    #[serde(skip_serializing, skip_deserializing)]
    pub size: u64,
    #[serde(skip_serializing, skip_deserializing)]
    pub counter: i32,
    #[serde(skip_serializing, skip_deserializing)]
    pub stats: HashMap<String, (i32, u64)>,
    #[serde(skip_serializing, skip_deserializing)]
    pub failed_deletions: Vec<(PathBuf, String)>,
}

/// Default values for a cleaningjob instance
impl Default for CleaningJob {
    /// default values for a cleaningjob instance
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            patterns: vec![],
            exclude_patterns: vec![],
            dry_run: true,
            skip_confirmation: false,
            include_symlinks: false,
            remove_broken_symlinks: false,
            stats_mode: false,
            older_than_secs: None,
            show_progress: false,
            targets: Vec::new(),
            size: 0,
            counter: 0,
            stats: HashMap::new(),
            failed_deletions: Vec::new(),
        }
    }
}

/// CleaningJob methods
impl CleaningJob {
    /// constructor
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: String,
        patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        dry_run: bool,
        skip_confirmation: bool,
        include_symlinks: bool,
        remove_broken_symlinks: bool,
        stats_mode: bool,
        older_than_secs: Option<u64>,
        show_progress: bool,
    ) -> Self {
        Self {
            path,
            patterns,
            exclude_patterns,
            dry_run,
            skip_confirmation,
            include_symlinks,
            remove_broken_symlinks,
            stats_mode,
            older_than_secs,
            show_progress,
            targets: Vec::new(),
            size: 0,
            counter: 0,
            stats: HashMap::new(),
            failed_deletions: Vec::new(),
        }
    }

    /// run the cleaning job
    #[time("info")]
    pub fn run(&mut self) -> Result<()> {
        let path_str = self.path.clone();
        let path = Path::new(&path_str);

        // Canonicalize base path for security checks
        let base_path = path.canonicalize().map_err(|e| {
            CleanError::ConfigError(format!("Invalid path '{}': {}", path_str, e))
        })?;

        // Build globsets
        let (include_set, exclude_set) = self.build_globsets()?;

        // Collect targets
        self.collect_targets(path, &base_path, &include_set, &exclude_set)?;

        // Display statistics if enabled
        if self.stats_mode {
            self.display_stats();
        }

        // Confirm deletion if needed
        if !self.targets.is_empty() && !self.skip_confirmation {
            let confirmation = Confirm::new()
                .with_prompt("Do you want to delete the above?")
                .interact()
                .map_err(|e| CleanError::ConfigError(format!("Confirmation failed: {}", e)))?;

            if confirmation {
                self.execute_deletion();
            } else {
                warn!("Cleaning operation cancelled.");
                return Ok(());
            }
        }

        // Display summary
        if !self.dry_run && self.counter > 0 {
            info!(
                "Deleted {} item(s) totalling {:.2} MB",
                self.counter,
                (self.size as f64) / 1000000.
            );
        }

        Ok(())
    }

    /// Build globsets for include and exclude patterns
    fn build_globsets(&self) -> Result<(GlobSet, Option<GlobSet>)> {
        let mut builder = GlobSetBuilder::new();
        for pattern in self.patterns.iter() {
            builder.add(Glob::new(pattern)?);
        }
        let include_set = builder.build()?;

        let exclude_set = if !self.exclude_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in self.exclude_patterns.iter() {
                builder.add(Glob::new(pattern)?);
            }
            Some(builder.build()?)
        } else {
            None
        };

        Ok((include_set, exclude_set))
    }

    /// Check if path should be processed
    fn should_process(&self, entry_path: &Path, base_path: &Path) -> bool {
        let current_path = Path::new(".");
        let parent_path = Path::new("..");

        // Skip current and parent paths
        if entry_path == current_path || entry_path == parent_path {
            return false;
        }

        // Skip paths starting with ".."
        if entry_path.starts_with("..") {
            warn!("skipping {:?}", entry_path.display());
            return false;
        }

        // Security check: Verify path is within base directory
        if let Ok(canonical) = entry_path.canonicalize() {
            if !canonical.starts_with(base_path) {
                warn!("Skipping path outside working directory: {:?}", entry_path.display());
                return false;
            }
        }

        true
    }

    /// Find which pattern matched the entry (for statistics)
    fn find_matching_pattern(&self, entry_path: &Path) -> Option<String> {
        for pattern in self.patterns.iter() {
            if let Ok(glob) = Glob::new(pattern) {
                if glob.compile_matcher().is_match(entry_path) {
                    return Some(pattern.clone());
                }
            }
        }
        None
    }

    /// Collect targets for deletion
    fn collect_targets(
        &mut self,
        path: &Path,
        base_path: &Path,
        include_set: &GlobSet,
        exclude_set: &Option<GlobSet>,
    ) -> Result<()> {
        // Create progress bar if requested
        let progress = if self.show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
            );
            pb.set_message("Scanning files...");
            pb.enable_steady_tick(Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        let mut processed = 0u64;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();

            // Update progress
            if let Some(ref pb) = progress {
                processed += 1;
                if processed.is_multiple_of(100) {
                    pb.set_message(format!("Scanned {} items, found {} matches", processed, self.counter));
                }
            }

            if !self.should_process(entry_path, base_path) {
                continue;
            }

            // Handle broken symlinks
            if self.remove_broken_symlinks && entry_path.is_symlink() {
                if let Err(_e) = fs::metadata(entry_path) {
                    self.handle_matched_entry(&entry, "broken-symlink".to_string())?;
                    continue;
                }
            }

            // Check if path matches include patterns
            if !include_set.is_match(entry_path) {
                continue;
            }

            // Check if path matches exclude patterns
            if let Some(ref exclude) = exclude_set {
                if exclude.is_match(entry_path) {
                    info!("Excluded: {:?}", entry_path.display());
                    continue;
                }
            }

            // Skip symlinks unless explicitly included
            if entry.path_is_symlink() && !self.include_symlinks {
                continue;
            }

            // Find matching pattern for statistics
            let pattern = self.find_matching_pattern(entry_path)
                .unwrap_or_else(|| "unknown".to_string());

            self.handle_matched_entry(&entry, pattern)?;
        }

        // Finish progress bar
        if let Some(pb) = progress {
            pb.finish_with_message(format!("Scan complete: {} items scanned, {} matches found", processed, self.counter));
        }

        Ok(())
    }

    /// Handle a matched entry (add to targets, update stats, or delete immediately)
    fn handle_matched_entry(&mut self, entry: &walkdir::DirEntry, pattern: String) -> Result<()> {
        let entry_path = entry.path();

        // Get and cache metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to get metadata for {:?}: {}", entry_path.display(), e);
                return Ok(());
            }
        };

        // Check age-based filtering
        if let Some(older_than_secs) = self.older_than_secs {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed.as_secs() < older_than_secs {
                        // File is too new, skip it
                        return Ok(());
                    }
                }
            }
        }

        // Calculate size
        let item_size = if metadata.is_file() {
            metadata.len()
        } else if metadata.is_dir() {
            get_size(entry_path).unwrap_or(0)
        } else {
            0
        };

        self.size += item_size;
        self.counter += 1;

        // Update statistics
        if self.stats_mode {
            let stat = self.stats.entry(pattern.clone()).or_insert((0, 0));
            stat.0 += 1;
            stat.1 += item_size;
        }

        // Either delete immediately or add to targets
        if self.skip_confirmation && !self.dry_run {
            self.remove_entry(entry);
            info!("Deleted: {:?}", entry_path.display());
        } else {
            self.targets.push((entry_path.to_path_buf(), metadata));
            info!("Matched: {:?}", entry_path.display());
        }

        Ok(())
    }

    /// Execute deletion of collected targets
    fn execute_deletion(&mut self) {
        // Clone targets to avoid borrow checker issues
        let targets_to_delete: Vec<_> = self.targets.clone();

        for (path, metadata) in targets_to_delete.iter() {
            if !self.dry_run {
                self.remove_path(path, metadata);
            }
        }

        // Display error summary if there were failures
        if !self.failed_deletions.is_empty() {
            error!("\n=== Deletion Failures ===");
            for (path, err_msg) in &self.failed_deletions {
                error!("  {:?}: {}", path.display(), err_msg);
            }
            error!("Total failures: {}\n", self.failed_deletions.len());
        }
    }

    /// Display statistics about matches
    fn display_stats(&self) {
        if !self.stats_mode {
            return;
        }

        info!("\n=== Statistics ===");
        let mut patterns: Vec<_> = self.stats.iter().collect();
        patterns.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Sort by count descending

        for (pattern, (count, size)) in patterns {
            info!(
                "  {}: {} item(s), {:.2} MB",
                pattern,
                count,
                (*size as f64) / 1000000.
            );
        }
        info!("==================\n");
    }

    /// remove collected targets (kept for compatibility)
    pub fn remove_targets(&mut self) {
        self.execute_deletion();
    }

    /// remove file or directory with path and metadata
    fn remove_path(&mut self, path: &Path, metadata: &Metadata) {
        let result = if metadata.is_dir() {
            fs::remove_dir_all(path)
        } else if metadata.is_file() || metadata.is_symlink() {
            fs::remove_file(path)
        } else {
            warn!("skipping unknown file type: {:?}", path.display());
            return;
        };

        if let Err(e) = result {
            let error_msg = format!("{}", e);
            self.failed_deletions.push((path.to_path_buf(), error_msg));
            error!("Failed to remove {:?}: {}", path.display(), e);
        }
    }

    /// remove file or directory with some safety measures (kept for compatibility)
    pub fn remove_entry(&mut self, entry: &walkdir::DirEntry) {
        let p = entry.path();

        let target = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to get metadata for {:?}: {}", p.display(), e);
                return;
            }
        };

        self.remove_path(p, &target);
    }
}
