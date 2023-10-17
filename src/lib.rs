pub mod constants;

use dialoguer::Confirm;
use globset::{Glob, GlobSetBuilder};
use log::{info, warn};
use logging_timer::time;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// --------------------------------------------------------------------
// core

#[derive(Serialize, Deserialize)]
pub struct CleaningJob {
    pub path: String,
    pub patterns: Vec<String>,
    pub dry_run: bool,
    pub skip_confirmation: bool,
    #[serde(skip_serializing, skip_deserializing)]
    targets: Vec<walkdir::DirEntry>,
    #[serde(skip_serializing, skip_deserializing)]
    size: u64,
    #[serde(skip_serializing, skip_deserializing)]
    counter: i32,
}

impl CleaningJob {
    pub fn new(path: String, patterns: Vec<String>, dry_run: bool, skip_confirmation: bool) -> Self { 
        Self { 
            path, 
            patterns,
            dry_run,
            skip_confirmation,
            targets: Vec::new(),
            size: 0,
            counter: 0, 
        }
    }

    #[time("info")]
    pub fn run(&mut self) {
        let mut builder = GlobSetBuilder::new();
        for pattern in self.patterns.iter() {
            builder.add(Glob::new(pattern).unwrap());
        }
        let gset = builder.build().unwrap();
        let path = Path::new(&self.path);
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if gset.is_match(entry.path()) {
                self.targets.push(entry.clone());
                info!("Matched: {:?}", entry.path().display());
                match entry.path().metadata() {
                    Ok(info) => self.size += info.len(),
                    Err(e) => eprintln!("metadata not found: {:?}", e),
                }
                self.counter += 1;
            }
        }

        if !self.targets.is_empty() {
            if self.skip_confirmation {
                self.remove_targets();
            } else {
                let confirmation = Confirm::new()
                    .with_prompt("Do you want to delete the above?")
                    .interact()
                    .unwrap();

                if confirmation {
                    self.remove_targets();
                } else {
                    println!("nevermind then.");
                }
            }
        } else {
            warn!("no matches found.");
        }
    }
    
    pub fn remove_targets(&self) {
        for name in self.targets.iter() {
            if !self.dry_run {
                self.remove_target(name);
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

    pub fn remove_target(&self, entry: &walkdir::DirEntry) {
        let p = entry.path();
        // println!("Deleting {}", p.display());
        if entry.metadata().unwrap().is_file() {
            fs::remove_file(p).expect("could not remove file: {p}");
        } else {
            fs::remove_dir_all(p).expect("could not remove directory: {p}");
        }
    }
}
