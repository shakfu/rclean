pub mod constants;

use dialoguer::Confirm;
use fs_extra::dir::get_size;
use globset::{Glob, GlobSetBuilder};
use log::{info, warn};
use logging_timer::time;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// --------------------------------------------------------------------
// core

/// Main configuration object for cleaning jobs with partial
/// with selective (de)serialization
#[derive(Serialize, Deserialize)]
pub struct CleaningJob {
    pub path: String,
    pub patterns: Vec<String>,
    pub dry_run: bool,
    pub skip_confirmation: bool,
    pub include_symlinks: bool,
    #[serde(skip_serializing, skip_deserializing)]
    targets: Vec<walkdir::DirEntry>,
    #[serde(skip_serializing, skip_deserializing)]
    size: u64,
    #[serde(skip_serializing, skip_deserializing)]
    counter: i32,
}

/// Default values for a cleaningjob instance
impl Default for CleaningJob {
    /// default values for a cleaningjob instance
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            patterns: vec![],
            dry_run: true,
            skip_confirmation: false,
            include_symlinks: false,
            targets: Vec::new(),
            size: 0,
            counter: 0,
        }
    }
}

/// CleaningJob methods
impl CleaningJob {
    /// constructor
    pub fn new(
        path: String,
        patterns: Vec<String>,
        dry_run: bool,
        skip_confirmation: bool,
        include_symlinks: bool,
    ) -> Self {
        Self {
            path,
            patterns,
            dry_run,
            skip_confirmation,
            include_symlinks,
            targets: Vec::new(),
            size: 0,
            counter: 0,
        }
    }

    /// run the cleaning job
    #[time("info")]
    pub fn run(&mut self) {
        // path cases
        let path = Path::new(&self.path);
        let current_path = Path::new(".");
        let parent_path = Path::new("..");

        let mut builder = GlobSetBuilder::new();
        for pattern in self.patterns.iter() {
            builder.add(Glob::new(pattern).unwrap());
        }
        let gset = builder.build().unwrap();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            // silently handle "." || ".." cases
            if entry_path == current_path || entry_path == parent_path {
                continue;
            }
            // skip paths which startwith ".."
            if entry_path.starts_with("..") {
                warn!("skipping {:?}", entry_path.display());
                continue;
            }
            if gset.is_match(entry_path) {
                match entry.path().metadata() {
                    // Ok(info) => self.size += info.len(),
                    Ok(_info) => self.size += get_size(entry_path).unwrap(),
                    Err(e) => eprintln!("metadata not found: {:?}", e),
                }
                self.counter += 1;
                if self.skip_confirmation {
                    self.remove_entry(&entry);
                    info!("Deleted: {:?}", entry_path.display());
                } else {
                    self.targets.push(entry.clone());
                    info!("Matched: {:?}", entry_path.display());
                }
            }
        }

        if !self.targets.is_empty() && !self.skip_confirmation {
            let confirmation = Confirm::new()
                .with_prompt("Do you want to delete the above?")
                .interact()
                .unwrap();

            if confirmation {
                self.remove_targets();
            } else {
                warn!("Cleaning operation cancelled.");
                return;
            }
        }

        if !self.dry_run {
            info!(
                "Deleted {} item(s) totalling {:.2} MB",
                self.counter,
                (self.size as f64) / 1000000.
            );
        }
    }

    /// remove collected targets
    pub fn remove_targets(&self) {
        for name in self.targets.iter() {
            if !self.dry_run {                
                self.remove_entry(name);
            }
        }
    }

    /// remove file or directory with some safety measures
    pub fn remove_entry(&self, entry: &walkdir::DirEntry) {
        let p = entry.path();
        let target = entry.metadata().unwrap();
        if target.is_symlink() {
            if self.include_symlinks {
                fs::remove_file(p).expect("could not remove symlink: {p}");
            } else {
                warn!("skipping symlink: {:?}", entry.path().display());
            }
        } else if target.is_file() {
            fs::remove_file(p).expect("could not remove file: {p}");
        } else if target.is_dir() {
            fs::remove_dir_all(p).expect("could not remove directory: {p}");
        } else {
            warn!("skipping unknowm: {:?}", entry.path().display());
        }
    }
}
