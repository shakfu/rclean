#![allow(unused)]

use clap::Parser;
use dialoguer::Confirm;
use globset::{Glob, GlobSetBuilder};
use log::{debug, error, info, trace, warn};
use logging_timer::{stime, time};
use simplelog::*;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;


// --------------------------------------------------------------------
// constants

/// list of glob patterns of files / directories to remove.
const PATTERNS: [&str;14] = [
        // directory
        "**/.coverage",
        "**/.DS_Store",
        // ".egg-info",
        "**/.cache",
        "**/.mypy_cache",
        "**/.pylint_cache",
        "**/.pytest_cache",
        "**/.ruff_cache",
        "**/__pycache__",
        // file
        "**/.bash_history",
        "*.log",
        "*.o",
        "*.py[co]",
        "**/.python_history",
        "**/pip-log.txt",
    ];
 
// --------------------------------------------------------------------
// cli api

/// Program to cleanup non-essential files or directories
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to begin default cleaning
    #[arg(short, long, default_value_os = ".")]
    path: String,

    /// Skip confirmation
    #[arg(short='y', long)]
    skip_confirmation: bool,

    /// Dry-run without actual removal
    #[arg(short, long)]
    dry_run: bool,
}

// --------------------------------------------------------------------
// core

struct CleaningJob {
    path: String,
    patterns: Vec<String>,
    dry_run: bool,
    skip_confirmation: bool,
}

impl CleaningJob {

    #[time("info")]
    fn run(&self) {
        let mut xs: Vec<walkdir::DirEntry> = Vec::new();
        let mut size = 0;
        let mut counter = 0;
        let mut builder = GlobSetBuilder::new();
        for pattern in self.patterns.iter() {
            builder.add(Glob::new(pattern).unwrap());
        }
        let gset = builder.build().unwrap();
        let path = std::path::Path::new(&self.path);
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if gset.is_match(entry.path()) {
                xs.push(entry.clone());
                println!("Matched: {:?}", entry.path().display());
                match entry.path().metadata() {
                    Ok(info) => size += info.len(),
                    Err(e) => eprintln!("metadata not found"),
                }
                counter += 1;
            }
        }

        if !xs.is_empty() {
            let mut confirmation = false;
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
                for name in xs.iter() {
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

    fn remove(&self, entry: &walkdir::DirEntry) {
        let p = entry.path();
        // println!("Deleting {}", p.display());
        if entry.metadata().unwrap().is_file() {
            fs::remove_file(p);
        } else {
            fs::remove_dir_all(p);
        }
    }
    
}

// --------------------------------------------------------------------
// main function

fn main() {
    let config = ConfigBuilder::new()
        .set_level_color(Level::Info, Some(Color::Green))
        .set_level_color(Level::Trace, Some(Color::Magenta))
        .build();

    TermLogger::init(
        LevelFilter::Trace,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
    let args = Args::parse();
    let mut job = CleaningJob {
        path: args.path,
        patterns: vec![],
        dry_run: args.dry_run,
        skip_confirmation: args.skip_confirmation,
    };
    for p in PATTERNS {
        job.patterns.push(String::from(p));
    }
    job.run();

}
