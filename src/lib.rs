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
}

impl CleaningJob {
    pub fn new(path: String, patterns: Vec<String>, dry_run: bool, skip_confirmation: bool) -> Self { 
        Self { path, patterns, dry_run, skip_confirmation }
    }

    #[time("info")]
    pub fn run(&self) {
        let mut targets: Vec<walkdir::DirEntry> = Vec::new();
        let mut size = 0;
        let mut counter = 0;
        let mut builder = GlobSetBuilder::new();
        for pattern in self.patterns.iter() {
            builder.add(Glob::new(pattern).unwrap());
        }
        let gset = builder.build().unwrap();
        let path = Path::new(&self.path);
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if gset.is_match(entry.path()) {
                targets.push(entry.clone());
                println!("Matched: {:?}", entry.path().display());
                match entry.path().metadata() {
                    Ok(info) => size += info.len(),
                    Err(e) => eprintln!("metadata not found: {:?}", e),
                }
                counter += 1;
            }
        }

        if !targets.is_empty() {
            let mut confirmation = false;
            assert!(!confirmation); // to silence warning
            if self.skip_confirmation {
                confirmation = true;
            } else {
                confirmation = Confirm::new()
                    .with_prompt("Do you want to delete the above?")
                    .interact()
                    .unwrap();
            }

            if confirmation {
                // println!("Looks like you want to continue");
                for name in targets.iter() {
                    if !self.dry_run {
                        self.remove(name);
                    }
                }
                if !self.dry_run {
                    info!(
                        "Deleted {} item(s) totalling {:.2} MB",
                        counter,
                        (size as f64) / 1000000.
                    );
                }
            } else {
                println!("nevermind then.");
            }
        } else {
            warn!("no matches found.");
        }
    }

    pub fn remove(&self, entry: &walkdir::DirEntry) {
        let p = entry.path();
        // println!("Deleting {}", p.display());
        if entry.metadata().unwrap().is_file() {
            fs::remove_file(p).expect("could not remove file: {p}");
        } else {
            fs::remove_dir_all(p).expect("could not remove directory: {p}");
        }
    }
}
