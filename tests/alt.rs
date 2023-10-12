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
    // Path to begin cleaning
    #[arg(short, long, default_value_os = ".", help = "path for default cleanup")]
    path: String,

    // glob options
    #[arg(short, long, help = "use glob patterns")]
    glob: Option<String>,
}

#[derive()]
struct Job<'a>  {
    root: &'a std::path::Path,
    patterns: Vec<&'a str>,
}

    #[time("info")]
    fn cleanup(&self) -> std::io::Result<()> {
        let mut size = 0;
        let mut counter = 0;
        for entry in WalkDir::new(self.root).into_iter().filter_map(|e| e.ok()) {
            if self.is_removable(entry.clone()) {
                size += entry.path().metadata()?.len();
                counter += 1;
                self.remove_direntry(entry.clone());
            }
        }
    
        info!(
            "Deleted {} item(s) totalling {:.2} MB",
            counter,
            (size as f64) / 1000000.
        );
        Ok(())
    }

    fn remove_direntry(&self, entry: walkdir::DirEntry) {
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

}



// --------------------------------------------------------------------
// remove functions



fn remove_pathbuf(entry: PathBuf) {
    println!("Deleting {}", entry.display());
    if entry.metadata().unwrap().is_file() {
        fs::remove_file(entry);
    } else {
        fs::remove_dir_all(entry);
    }
}



// --------------------------------------------------------------------
// cleaning functions


#[time("info")]
fn glob_cleanup(glob_pattern: String) {
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
    let confirmation = Confirm::new()
        .with_prompt("Do you want to delete the above?")
        .interact()
        .unwrap();

    if confirmation {
        println!("Looks like you want to continue");
        for name in xs.iter() {
            println!("deleting {:?}", name);
        }
    } else {
        println!("nevermind then.");
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
        glob_cleanup(args.glob.unwrap());
    } else {
        let job = Job {
            root: std::path::Path::new(&args.path),
            patterns:  vec![
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
                ]
        };
        job.cleanup();
    }
}
