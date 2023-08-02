#![allow(unused)]

use clap::Parser;
use std::fs;
use walkdir::WalkDir;

// --------------------------------------------------------------------
// cli

/// Program to cleanup non-essential files or directories
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to begin cleaning
    #[arg(short, long, default_value_os = ".")]
    path: String,
    // /// Add start_swith patterns
    // #[arg(short, long)]
    // spattern: Vec<String>,

    // /// Add ends_with patterns
    // #[arg(short, long)]
    // epattern: Vec<String>
}

// --------------------------------------------------------------------
// remove functions

fn remove(entry: walkdir::DirEntry) {
    let p = entry.path();
    println!("Deleting {}", p.display());
    if entry.metadata().unwrap().is_file() {
        fs::remove_file(p);
    } else {
        fs::remove_dir_all(p);
    }
}

// --------------------------------------------------------------------
// matching functions

fn is_removable(entry: walkdir::DirEntry) -> bool {
    let endswith_patterns = vec![
        // directory
        ".coverage",
        ".DS_Store",
        ".egg-info",
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
        if entry
            .file_name()
            .to_str()
            .map_or(false, |s| s.ends_with(pattern))
        {
            return true;
        }
    }
    return false;
}

// --------------------------------------------------------------------
// cleaning functions

fn cleanup(root: &std::path::Path) -> std::io::Result<()> {
    let mut size = 0;
    let mut counter = 0;

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if is_removable(entry.clone()) {
            size += entry.path().metadata()?.len();
            counter += 1;
            remove(entry.clone());
        }
    }

    println!(
        "Deleted {} item(s) totalling {:.2} MB",
        counter,
        (size as f64) / 1000000.
    );
    Ok(())
}

// --------------------------------------------------------------------
// main function

fn main() {
    let args = Args::parse();
    // println!("path: '{}'", args.path);
    let path = std::path::Path::new(&args.path);
    cleanup(&path);
}
