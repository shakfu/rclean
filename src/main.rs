#![allow(unused)]

use clap::Parser;
use toml::Table;
use serde::Deserialize;
use dialoguer::Confirm;
use globset::{Glob, GlobSetBuilder};
use log::{debug, error, info, trace, warn};
use logging_timer::{stime, time};
use simplelog::{Color, ColorChoice, ConfigBuilder, Level, LevelFilter, TermLogger, TerminalMode};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;


// --------------------------------------------------------------------
// constants

const SETTINGS_FILENAME: &str = "rclean.toml";

/// list of glob patterns of files / directories to remove.
const PATTERNS: [&str; 14] = [
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
    /// Working Directory
    #[arg(short, long, default_value_os = ".")]
    path: String,

    /// Skip confirmation
    #[arg(short = 'y', long)]
    skip_confirmation: bool,

    /// Dry-run without actual removal
    #[arg(short, long)]
    dry_run: bool,

    /// Specify custom glob pattern(s)
    #[arg(short, long)]
    glob: Option<Vec<String>>,

    /// list default glob patterns
    #[arg(short, long)]
    list: bool,

    /// Configure from 'rclean.toml' file
    #[arg(short, long)]
    configfile: bool,
}

// --------------------------------------------------------------------
// core
#[derive(Deserialize)]
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
    let logging_config = ConfigBuilder::new()
        .set_level_color(Level::Info, Some(Color::Green))
        .set_level_color(Level::Trace, Some(Color::Magenta))
        .build();

    TermLogger::init(
        LevelFilter::Trace,
        logging_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    let args = Args::parse();
    if args.configfile {
        let settings_file = std::path::Path::new(SETTINGS_FILENAME);
        if settings_file.exists() {
            info!("using settings file: {:?}", SETTINGS_FILENAME);
            let contents = fs::read_to_string(SETTINGS_FILENAME)
                .expect("cannot read file");
            let job: CleaningJob = toml::from_str(&contents).expect("cannot read");
            job.run();
        } else {
            error!("Error: settings file {:?} not found", SETTINGS_FILENAME);
        }
    } else if args.list {
        info!("default patterns: {:?}", PATTERNS);
    } else {
        let mut job = CleaningJob {
            path: args.path,
            patterns: args.glob.unwrap_or(vec![]),
            dry_run: args.dry_run,
            skip_confirmation: args.skip_confirmation,
        };
        if job.patterns.is_empty() {
            for p in PATTERNS {
                job.patterns.push(String::from(p));
            }
        }
        job.run();
    }
}
