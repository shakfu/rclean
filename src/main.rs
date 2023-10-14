#![allow(unused)]

use clap::Parser;
use dialoguer::Confirm;
use glob::glob;
use log::{debug, error, info, trace, warn};
use logging_timer::{stime, time};
use simplelog::*;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;


// --------------------------------------------------------------------
// constants
const PATTERNS: [&str;11] = [
        // directory
        ".coverage",
        ".DS_Store",
        // ".egg-info",
        ".mypy_cache/",
        ".pylint_cache",
        ".ruff_cache",
        "__pycache__",
        // file
        ".bash_history",
        ".log",
        ".pyc",
        ".python_history",
        "pip-log.txt",
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

    /// Dry-run without actual removal
    #[arg(short, long)]
    dry_run: bool,

    /// Glob-based removal
    #[arg(short, long)]
    glob: Option<String>,
}

// --------------------------------------------------------------------
// model

type Matcher<T> = fn(T) -> T;

struct Job {
    path: String,
    patterns: Vec<String>,
    dry_run: bool,
}

trait Cleaner<T> {
    fn run(&self);
    fn remove(&self, entry: T);
    fn is_removable(&self, entry: T) -> bool;
}

impl Cleaner<walkdir::DirEntry> for Job {

    #[time("info")]
    fn run(&self) {
        let mut size = 0;
        let mut counter = 0;
        let path = std::path::Path::new(&self.path);
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if self.is_removable(entry.clone()) {
                match entry.path().metadata() {
                    Ok(info) => size += info.len(),
                    Err(e) => eprintln!("metadata not found"),
                }
                counter += 1;
                if !self.dry_run {
                    self.remove(entry.clone());
                }
            }
        }
    
        info!(
            "Deleted {} item(s) totalling {:.2} MB",
            counter,
            (size as f64) / 1000000.
        );
    }

    fn remove(&self, entry: walkdir::DirEntry) {
        let p = entry.path();
        println!("Deleting {}", p.display());
        if entry.metadata().unwrap().is_file() {
            fs::remove_file(p);
        } else {
            fs::remove_dir_all(p);
        }
    }

    fn is_removable(&self, entry: walkdir::DirEntry) -> bool {
        for pattern in &self.patterns {
            if (entry
                .file_name()
                .to_str()
                .map_or(false, |s| s.ends_with(pattern)))
            {
                return true;
            }
        }
        false
    }  
}

impl Cleaner<&PathBuf> for Job {

    fn is_removable(&self, entry: &std::path::PathBuf) -> bool {
        true
    }

    #[time("info")]
    fn run(&self) {
        let mut xs: Vec<PathBuf> = Vec::new();
        let mut process = |e: PathBuf| {
            println!("{:?}", e.display());
            xs.push(e);
        };
        for pattern in &self.patterns {
            for entry in glob(pattern).expect("Failed to read glob pattern") {
                match entry {
                    Ok(ref path) => process(entry.unwrap()),
                    Err(e) => println!("{:?}", e),
                }
            }
        }
        if !xs.is_empty() {
            let confirmation = Confirm::new()
            .with_prompt("Do you want to delete the above?")
            .interact()
            .unwrap();
    
            if confirmation {
                println!("Looks like you want to continue");
                for name in xs.iter() {
                    println!("deleting {:?}", name);
                    if !self.dry_run {
                        self.remove(name);
                    }
                }
            } else {
                println!("nevermind then.");
            }
        } else {
            warn!("no matches found.");
        }
    }

    fn remove(&self, entry: &PathBuf) {
        println!("Deleting {}", entry.display());
        if entry.metadata().unwrap().is_file() {
            fs::remove_file(entry);
        } else {
            fs::remove_dir_all(entry);
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
    if (args.glob.is_some()) {
        let mut job = Job {
            path: args.path,
            patterns: vec![],
            dry_run: args.dry_run,
        };
        job.patterns.push(args.glob.unwrap());
        // job.run();
        <Job as Cleaner<&PathBuf>>::run(&job);
    } else {
        let mut job = Job {
            path: args.path,
            patterns: vec![],
            dry_run: args.dry_run,
        };
        for p in PATTERNS {
            job.patterns.push(String::from(p));
        }
        // job.run();
        <Job as Cleaner<walkdir::DirEntry>>::run(&job)
    }
}
