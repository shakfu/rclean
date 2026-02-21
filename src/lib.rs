pub mod constants;

use dialoguer::Confirm;
use fs_extra::dir::get_size;
use globset::{Glob, GlobMatcher, GlobSet, GlobSetBuilder};
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
// utilities

/// Parse duration string like "30d", "7d", "24h", "3600s" into seconds.
///
/// Supported units: s (seconds), m (minutes), h (hours), d (days), w (weeks).
pub fn parse_duration(duration: &str) -> Result<u64> {
    let duration = duration.trim();
    if duration.is_empty() {
        return Err(CleanError::ConfigError(
            "Duration cannot be empty".to_string(),
        ));
    }

    if duration.len() < 2 {
        return Err(CleanError::ConfigError(format!(
            "Invalid duration '{}': must be a number followed by a unit (s, m, h, d, w)",
            duration
        )));
    }

    let (num_part, unit_part) = duration.split_at(duration.len() - 1);
    let number: u64 = num_part.parse().map_err(|_| {
        CleanError::ConfigError(format!("Invalid number in duration: {}", num_part))
    })?;

    let multiplier = match unit_part {
        "s" => 1,      // seconds
        "m" => 60,     // minutes
        "h" => 3600,   // hours
        "d" => 86400,  // days
        "w" => 604800, // weeks
        _ => {
            return Err(CleanError::ConfigError(format!(
                "Invalid duration unit '{}'. Use 's', 'm', 'h', 'd', or 'w'",
                unit_part
            )))
        }
    };

    Ok(number * multiplier)
}

/// Format a byte count as a human-readable string using binary units.
///
/// Uses IEC binary prefixes: B, KiB, MiB, GiB, TiB.
pub fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;

    let size = bytes as f64;
    if size >= TIB {
        format!("{:.2} TiB", size / TIB)
    } else if size >= GIB {
        format!("{:.2} GiB", size / GIB)
    } else if size >= MIB {
        format!("{:.2} MiB", size / MIB)
    } else if size >= KIB {
        format!("{:.2} KiB", size / KIB)
    } else {
        format!("{} B", bytes)
    }
}

/// Search upward from `start_dir` for a file named `filename`.
/// Returns the path to the first match, or None.
pub fn find_config_upward(start_dir: &Path, filename: &str) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(filename);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Return the path to the global config file, if it exists.
/// Checks `~/.config/rclean/config.toml`.
pub fn global_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("rclean").join("config.toml")).filter(|p| p.is_file())
}

/// Discover a config file: first search upward for `.rclean.toml`, then fall back to global.
pub fn discover_config(start_dir: &Path) -> Option<PathBuf> {
    find_config_upward(start_dir, constants::SETTINGS_FILENAME)
        .or_else(global_config_path)
}

// --------------------------------------------------------------------
// configuration

/// Serializable configuration for a cleaning job
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CleanConfig {
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
    #[serde(skip)]
    pub json_mode: bool,
}

impl Default for CleanConfig {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            patterns: vec![],
            exclude_patterns: vec![],
            dry_run: false,
            skip_confirmation: false,
            include_symlinks: false,
            remove_broken_symlinks: false,
            stats_mode: false,
            older_than_secs: None,
            show_progress: false,
            json_mode: false,
        }
    }
}

impl CleanConfig {
    /// Start building a new configuration
    pub fn builder() -> CleanConfigBuilder {
        CleanConfigBuilder::default()
    }
}

/// Builder for constructing CleanConfig with a fluent API
#[derive(Default)]
pub struct CleanConfigBuilder {
    config: CleanConfig,
}

impl CleanConfigBuilder {
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.config.path = path.into();
        self
    }

    pub fn patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.patterns = patterns;
        self
    }

    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.exclude_patterns = patterns;
        self
    }

    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.config.dry_run = dry_run;
        self
    }

    pub fn skip_confirmation(mut self, skip: bool) -> Self {
        self.config.skip_confirmation = skip;
        self
    }

    pub fn include_symlinks(mut self, include: bool) -> Self {
        self.config.include_symlinks = include;
        self
    }

    pub fn remove_broken_symlinks(mut self, remove: bool) -> Self {
        self.config.remove_broken_symlinks = remove;
        self
    }

    pub fn stats_mode(mut self, stats: bool) -> Self {
        self.config.stats_mode = stats;
        self
    }

    pub fn older_than_secs(mut self, secs: Option<u64>) -> Self {
        self.config.older_than_secs = secs;
        self
    }

    pub fn show_progress(mut self, progress: bool) -> Self {
        self.config.show_progress = progress;
        self
    }

    pub fn json_mode(mut self, json: bool) -> Self {
        self.config.json_mode = json;
        self
    }

    pub fn build(self) -> CleanConfig {
        self.config
    }
}

// --------------------------------------------------------------------
// core

/// Pre-compiled pattern matchers for statistics attribution
type PatternMatchers = Vec<(String, GlobMatcher)>;

/// A matched item for structured output
#[derive(Serialize, Debug)]
pub struct MatchedItem {
    pub path: String,
    pub size: u64,
    pub pattern: String,
}

/// Runtime executor for cleaning jobs
pub struct CleaningJob {
    pub config: CleanConfig,
    targets: Vec<(PathBuf, Metadata)>,
    pub size: u64,
    pub counter: usize,
    pub stats: HashMap<String, (usize, u64)>,
    pub failed_deletions: Vec<(PathBuf, String)>,
    pub matched_items: Vec<MatchedItem>,
}

impl CleaningJob {
    /// Create a new cleaning job from a configuration
    pub fn new(config: CleanConfig) -> Self {
        Self {
            config,
            targets: Vec::new(),
            size: 0,
            counter: 0,
            stats: HashMap::new(),
            failed_deletions: Vec::new(),
            matched_items: Vec::new(),
        }
    }

    /// Return whether any deletions failed
    pub fn has_failures(&self) -> bool {
        !self.failed_deletions.is_empty()
    }

    /// Produce JSON output summarizing the run
    pub fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        let stats: Vec<_> = self
            .stats
            .iter()
            .map(|(pattern, (count, size))| {
                serde_json::json!({
                    "pattern": pattern,
                    "count": count,
                    "size": size,
                    "size_human": format_size(*size),
                })
            })
            .collect();

        let failures: Vec<_> = self
            .failed_deletions
            .iter()
            .map(|(path, err)| {
                serde_json::json!({
                    "path": path.display().to_string(),
                    "error": err,
                })
            })
            .collect();

        let output = serde_json::json!({
            "matches": self.matched_items,
            "summary": {
                "total_count": self.counter,
                "total_size": self.size,
                "total_size_human": format_size(self.size),
                "dry_run": self.config.dry_run,
            },
            "stats": stats,
            "failures": failures,
        });

        serde_json::to_string_pretty(&output)
    }

    /// Run the cleaning job
    #[time("info")]
    pub fn run(&mut self) -> Result<()> {
        let path_str = self.config.path.clone();
        let path = Path::new(&path_str);

        // Canonicalize base path for security checks
        let base_path = path.canonicalize().map_err(|e| {
            CleanError::ConfigError(format!("Invalid path '{}': {}", path_str, e))
        })?;

        // Build globsets
        let (include_set, exclude_set, matchers) = self.build_globsets()?;

        // Collect targets
        self.collect_targets(path, &base_path, &include_set, &exclude_set, &matchers)?;

        // Display statistics if enabled (suppressed in JSON mode)
        if self.config.stats_mode && !self.config.json_mode {
            self.display_stats();
        }

        // Confirm deletion if needed
        if !self.targets.is_empty() && !self.config.skip_confirmation {
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

        // Display summary (suppressed in JSON mode)
        if !self.config.dry_run && self.counter > 0 && !self.config.json_mode {
            info!(
                "Deleted {} item(s) totalling {}",
                self.counter,
                format_size(self.size)
            );
        }

        Ok(())
    }

    /// Build globsets for include and exclude patterns, plus individual matchers for stats
    fn build_globsets(&self) -> Result<(GlobSet, Option<GlobSet>, PatternMatchers)> {
        let mut builder = GlobSetBuilder::new();
        let mut matchers = Vec::new();

        for pattern in self.config.patterns.iter() {
            let glob = Glob::new(pattern)?;
            builder.add(glob.clone());
            matchers.push((pattern.clone(), glob.compile_matcher()));
        }
        let include_set = builder.build()?;

        let exclude_set = if !self.config.exclude_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in self.config.exclude_patterns.iter() {
                builder.add(Glob::new(pattern)?);
            }
            Some(builder.build()?)
        } else {
            None
        };

        Ok((include_set, exclude_set, matchers))
    }

    /// Check if path should be processed (basic filtering only)
    fn should_process(&self, entry_path: &Path) -> bool {
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

        true
    }

    /// Security check: verify path is within base directory
    /// For symlinks, we skip canonicalization since symlinks are already
    /// protected by the include_symlinks flag
    fn is_path_safe(&self, entry_path: &Path, base_path: &Path, is_symlink: bool) -> bool {
        // Skip canonicalization for symlinks - they're protected by include_symlinks flag
        // Canonicalizing symlinks follows them to their target, which may be outside
        // the working directory even though the symlink itself is inside
        if is_symlink {
            return true;
        }

        // Security check: Verify path is within base directory
        if let Ok(canonical) = entry_path.canonicalize() {
            if !canonical.starts_with(base_path) {
                warn!(
                    "Skipping path outside working directory: {:?}",
                    entry_path.display()
                );
                return false;
            }
        }

        true
    }

    /// Find which pattern matched the entry using pre-compiled matchers
    fn find_matching_pattern(
        matchers: &[(String, GlobMatcher)],
        entry_path: &Path,
    ) -> Option<String> {
        for (pattern, matcher) in matchers {
            if matcher.is_match(entry_path) {
                return Some(pattern.clone());
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
        matchers: &[(String, GlobMatcher)],
    ) -> Result<()> {
        // Create progress bar if requested
        let progress = if self.config.show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap(),
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
                    pb.set_message(format!(
                        "Scanned {} items, found {} matches",
                        processed, self.counter
                    ));
                }
            }

            // Basic path filtering (skip ".", "..", paths starting with "..")
            if !self.should_process(entry_path) {
                continue;
            }

            // Handle broken symlinks
            if self.config.remove_broken_symlinks && entry_path.is_symlink() {
                if let Err(_e) = fs::metadata(entry_path) {
                    self.handle_matched_entry(
                        &entry,
                        "broken-symlink".to_string(),
                        &progress,
                    )?;
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
                    let msg = format!("Excluded: {:?}", entry_path.display());
                    if let Some(ref pb) = progress {
                        pb.println(&msg);
                    } else {
                        info!("{}", msg);
                    }
                    continue;
                }
            }

            // Determine if entry is a symlink
            let is_symlink = entry.path_is_symlink();

            // Skip symlinks unless explicitly included
            if is_symlink && !self.config.include_symlinks {
                continue;
            }

            // Security check: verify path is within base directory
            if !self.is_path_safe(entry_path, base_path, is_symlink) {
                continue;
            }

            // Find matching pattern for statistics
            let pattern = Self::find_matching_pattern(matchers, entry_path)
                .unwrap_or_else(|| "unknown".to_string());

            self.handle_matched_entry(&entry, pattern, &progress)?;
        }

        // Finish progress bar
        if let Some(pb) = progress {
            pb.finish_with_message(format!(
                "Scan complete: {} items scanned, {} matches found",
                processed, self.counter
            ));
        }

        Ok(())
    }

    /// Handle a matched entry (add to targets, update stats, or delete immediately)
    fn handle_matched_entry(
        &mut self,
        entry: &walkdir::DirEntry,
        pattern: String,
        progress: &Option<ProgressBar>,
    ) -> Result<()> {
        let entry_path = entry.path();

        // Get and cache metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                error!(
                    "Failed to get metadata for {:?}: {}",
                    entry_path.display(),
                    e
                );
                return Ok(());
            }
        };

        // Check age-based filtering
        if let Some(older_than_secs) = self.config.older_than_secs {
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
        if self.config.stats_mode {
            let stat = self.stats.entry(pattern.clone()).or_insert((0, 0));
            stat.0 += 1;
            stat.1 += item_size;
        }

        // Collect for JSON output
        if self.config.json_mode {
            self.matched_items.push(MatchedItem {
                path: entry_path.display().to_string(),
                size: item_size,
                pattern: pattern.clone(),
            });
        }

        // Either delete immediately or add to targets
        if self.config.skip_confirmation && !self.config.dry_run {
            self.remove_entry(entry);
            if !self.config.json_mode {
                let msg = format!("Deleted: {:?}", entry_path.display());
                if let Some(ref pb) = progress {
                    pb.println(&msg);
                } else {
                    info!("{}", msg);
                }
            }
        } else {
            self.targets.push((entry_path.to_path_buf(), metadata));
            if !self.config.json_mode {
                let msg = format!("Matched: {:?}", entry_path.display());
                if let Some(ref pb) = progress {
                    pb.println(&msg);
                } else {
                    info!("{}", msg);
                }
            }
        }

        Ok(())
    }

    /// Execute deletion of collected targets
    fn execute_deletion(&mut self) {
        let targets_to_delete = std::mem::take(&mut self.targets);

        for (path, metadata) in targets_to_delete.iter() {
            if !self.config.dry_run {
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
        if !self.config.stats_mode {
            return;
        }

        info!("\n=== Statistics ===");
        let mut patterns: Vec<_> = self.stats.iter().collect();
        patterns.sort_by(|a, b| b.1 .0.cmp(&a.1 .0)); // Sort by count descending

        for (pattern, (count, size)) in patterns {
            info!(
                "  {}: {} item(s), {}",
                pattern,
                count,
                format_size(*size)
            );
        }
        info!("==================\n");
    }

    /// Remove file or directory with path and metadata
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

    /// Remove file or directory from a walkdir entry
    fn remove_entry(&mut self, entry: &walkdir::DirEntry) {
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
