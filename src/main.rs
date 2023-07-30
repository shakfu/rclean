#![allow(unused)]

use std::fs;
use walkdir::WalkDir;

fn remove(entry: walkdir::DirEntry) {
    let p = entry.path();
    println!("Deleting {}", p.display());
    if entry.metadata().unwrap().is_file() {
        fs::remove_file(p);
    } else {
        fs::remove_dir_all(p);
    }
}

fn main() -> std::io::Result<()> {

    let startswith_patterns  = vec![
        // directory
        ".DS_Store",
        "__pycache__",
        ".mypy_cache/",
        ".ruff_cache",
        ".pylint_cache",

        // file
        ".bash_history",
        ".python_history",
        "pip-log.txt",
    ];

    let endswith_patterns  = vec![
        // directory
        ".egg-info",
        ".coverage",

        // file
        ".pyc",
        ".log",
    ];

    let mut counter = 0;

    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        for pattern in &startswith_patterns {
            if entry.file_name().to_str().map_or(false, |s| s.starts_with(pattern)) {
                counter += 1;
                remove(entry.clone());
            }
        }

        for pattern in &endswith_patterns {
            if entry.file_name().to_str().map_or(false, |s| s.ends_with(pattern)) {
                counter += 1;
                remove(entry.clone());
            }
        }
    }

    println!("Deleted {} items of detritus", counter);
    Ok(())
}
