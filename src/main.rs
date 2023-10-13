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
// cli

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
// remove functions

fn remove_direntry(entry: walkdir::DirEntry) {
    let p = entry.path();
    println!("Deleting {}", p.display());
    if entry.metadata().unwrap().is_file() {
        fs::remove_file(p);
    } else {
        fs::remove_dir_all(p);
    }
}

fn remove_pathbuf(entry: &PathBuf) {
    println!("Deleting {}", entry.display());
    if entry.metadata().unwrap().is_file() {
        fs::remove_file(entry);
    } else {
        fs::remove_dir_all(entry);
    }
}

// --------------------------------------------------------------------
// matching functions

fn is_removable(entry: walkdir::DirEntry) -> bool {
    let endswith_patterns = vec![
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

    for pattern in &endswith_patterns {
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

// --------------------------------------------------------------------
// cleaning functions

#[time("info")]
fn cleanup(root: String, dry_run: bool) -> std::io::Result<()> {
    let mut size = 0;
    let mut counter = 0;
    let path = std::path::Path::new(&root);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if is_removable(entry.clone()) {
            size += entry.path().metadata()?.len();
            counter += 1;
            if !dry_run {
                remove_direntry(entry.clone());
            }
        }
    }

    info!(
        "Deleted {} item(s) totalling {:.2} MB",
        counter,
        (size as f64) / 1000000.
    );
    Ok(())
}

#[time("info")]
fn glob_cleanup(glob_pattern: String, dry_run: bool) {
    let mut xs: Vec<PathBuf> = Vec::new();
    let mut process = |e: PathBuf| {
        println!("{:?}", e.display());
        xs.push(e);
    };
    for entry in glob(&glob_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(ref path) => process(entry.unwrap()),
            Err(e) => println!("{:?}", e),
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
                if !dry_run {
                    remove_pathbuf(name);
                }
            }
        } else {
            println!("nevermind then.");
        }
    } else {
        warn!("no matches found.");
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
        match args.glob {
            Some(p) => glob_cleanup(p, args.dry_run),
            None => eprintln!("glob pattern not found"),
        };
    } else {
        cleanup(args.path, args.dry_run);
    }
}
