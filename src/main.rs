// --------------------------------------------------------------------
// imports

use clap::Parser;
use log::{error, info};
use std::fs;
use std::path::Path;
use std::process;

use rclean::constants::{get_default_patterns, SETTINGS_FILENAME};
use rclean::{CleaningJob, Result};

// --------------------------------------------------------------------
// cli api

/// Program to cleanup non-essential files or directories
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Working Directory
    #[arg(short, long, default_value_os = ".")]
    path: String,

    /// Specify custom glob pattern(s)
    #[arg(short, long)]
    glob: Option<Vec<String>>,

    /// Exclude patterns (files matching these will not be deleted)
    #[arg(short, long)]
    exclude: Option<Vec<String>>,

    /// Configure from config file (defaults to '.rclean.toml' if no path specified)
    #[arg(short, long, value_name = "PATH")]
    configfile: Option<String>,

    /// Write default '.rclean.toml' file
    #[arg(short, long)]
    write_configfile: bool,

    /// Dry-run without actual removal
    #[arg(short, long)]
    dry_run: bool,

    /// Skip confirmation
    #[arg(short = 'y', long)]
    skip_confirmation: bool,

    /// Include matched symlinks for removal
    #[arg(short, long)]
    include_symlinks: bool,

    /// Remove broken symlinks
    #[arg(short, long)]
    remove_broken_symlinks: bool,

    /// Display statistics by pattern
    #[arg(short = 's', long)]
    stats: bool,

    /// Only remove files older than specified duration (e.g., "30d", "7d", "24h", "3600s")
    #[arg(short = 'o', long)]
    older_than: Option<String>,

    /// Show progress bar during scanning
    #[arg(short = 'P', long)]
    progress: bool,

    /// list default glob patterns
    #[arg(short, long)]
    list: bool,
}

// --------------------------------------------------------------------
// helper functions

/// Parse duration string like "30d", "7d", "24h", "3600s" into seconds
///
/// # Errors
///
/// Returns error if the duration string is invalid
fn parse_duration(duration: &str) -> Result<u64> {
    let duration = duration.trim();
    if duration.is_empty() {
        return Err(rclean::CleanError::ConfigError("Duration cannot be empty".to_string()));
    }

    let (num_part, unit_part) = duration.split_at(duration.len() - 1);
    let number: u64 = num_part.parse()
        .map_err(|_| rclean::CleanError::ConfigError(format!("Invalid number in duration: {}", num_part)))?;

    let multiplier = match unit_part {
        "s" => 1,                    // seconds
        "m" => 60,                   // minutes
        "h" => 3600,                 // hours
        "d" => 86400,                // days
        "w" => 604800,               // weeks
        _ => return Err(rclean::CleanError::ConfigError(
            format!("Invalid duration unit '{}'. Use 's', 'm', 'h', 'd', or 'w'", unit_part)
        )),
    };

    Ok(number * multiplier)
}

// --------------------------------------------------------------------
// main function

/// configure and initialize logging
///
/// # Errors
///
/// This function will return an error if the logging cannot be initialized.
fn init_logging() {
    let logging_config = simplelog::ConfigBuilder::new()
        .set_level_color(simplelog::Level::Info, Some(simplelog::Color::Green))
        .set_level_color(simplelog::Level::Trace, Some(simplelog::Color::Magenta))
        .build();

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        logging_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("could not initialize logging");
}

/// generate default config file: '.rclean.toml'
///
/// # Errors
///
/// This function will return an error if the file cannot be written.
fn write_configfile(job: &CleaningJob) -> Result<()> {
    let toml = toml::to_string(&job)
        .map_err(|e| rclean::CleanError::ConfigError(format!("Failed to serialize config: {}", e)))?;

    let cfg_out = Path::new(SETTINGS_FILENAME);
    if !Path::new(cfg_out).exists() {
        info!("generating default '{SETTINGS_FILENAME}' file");
        fs::write(cfg_out, toml)?;
        Ok(())
    } else {
        Err(rclean::CleanError::ConfigError(
            format!("Cannot overwrite existing '{SETTINGS_FILENAME}' file")
        ))
    }
}

/// run cleanup job using configuration from config file
///
/// # Errors
///
/// This function will return an error if the file cannot be read.
fn run_job_from_configfile(config_path: Option<String>) -> Result<()> {
    let config_file = config_path.as_deref().unwrap_or(SETTINGS_FILENAME);
    let settings_file = Path::new(config_file);

    if !settings_file.exists() {
        return Err(rclean::CleanError::ConfigError(
            format!("Settings file '{}' not found", config_file)
        ));
    }

    info!("using settings file: {config_file:?}");
    let contents = fs::read_to_string(config_file)?;
    let mut job: CleaningJob = toml::from_str(&contents)
        .map_err(|e| rclean::CleanError::ConfigError(format!("Cannot deserialize from .toml: {}", e)))?;

    job.run()
}

/// main function
fn main() {
    init_logging();
    let args = Args::parse();

    let result = if args.configfile.is_some() {
        run_job_from_configfile(args.configfile)
    } else if args.list {
        info!("default patterns: {:?}", get_default_patterns());
        Ok(())
    } else {
        // Parse duration if provided
        let older_than_result = if let Some(ref duration_str) = args.older_than {
            parse_duration(duration_str).map(Some)
        } else {
            Ok(None)
        };

        match older_than_result {
            Ok(older_than_secs) => {
                let mut job = CleaningJob::new(
                    args.path,
                    args.glob
                        .unwrap_or_else(|| get_default_patterns().iter().map(|x| x.to_string()).collect()),
                    args.exclude.unwrap_or_default(),
                    args.dry_run,
                    args.skip_confirmation,
                    args.include_symlinks,
                    args.remove_broken_symlinks,
                    args.stats,
                    older_than_secs,
                    args.progress,
                );
                if args.write_configfile {
                    write_configfile(&job)
                } else {
                    job.run()
                }
            }
            Err(e) => Err(e),
        }
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        process::exit(1);
    }
}
