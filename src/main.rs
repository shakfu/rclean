use clap::Parser;
use log::{error, info};
use std::fs;
use std::path::Path;

use rclean::constants::{PATTERNS, SETTINGS_FILENAME};
use rclean::CleaningJob;

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

    /// Configure from 'rclean.toml' file
    #[arg(short, long)]
    configfile: bool,

    /// Write default 'rclean.toml' file
    #[arg(short, long)]
    write_configfile: bool,

    /// Dry-run without actual removal
    #[arg(short, long)]
    dry_run: bool,

    /// Skip confirmation
    #[arg(short = 'y', long)]
    skip_confirmation: bool,

    /// list default glob patterns
    #[arg(short, long)]
    list: bool,
}

// --------------------------------------------------------------------
// main function

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

fn write_configfile(job: &CleaningJob) {
    let toml: String = toml::to_string(&job).unwrap();
    let cfg_out = Path::new(SETTINGS_FILENAME);
    if !Path::new(cfg_out).exists() {
        info!("generating default 'rclean.toml' file");
        fs::write(cfg_out, toml).unwrap();
    } else {
        error!("cannot overwrite existing 'rclean.toml' file");
    }
}

fn run_job_from_configfile() {
    let settings_file = Path::new(SETTINGS_FILENAME);
    if settings_file.exists() {
        info!("using settings file: {:?}", SETTINGS_FILENAME);
        let contents = fs::read_to_string(SETTINGS_FILENAME)
            .expect("cannot read file");
        let mut job: CleaningJob = toml::from_str(&contents).expect("cannot read");
        job.run();
    } else {
        error!("Error: settings file {:?} not found", SETTINGS_FILENAME);
    }
}

fn main() {
    init_logging();
    let args = Args::parse();
    if args.configfile {
        run_job_from_configfile();
    } else if args.list {
        info!("default patterns: {:?}", PATTERNS);
    } else {
        let mut job = CleaningJob::new(
            args.path,
            args.glob.unwrap_or(
                PATTERNS.iter().map(|x| x.to_string()).collect()),
            args.dry_run,
            args.skip_confirmation,
        );
        if args.write_configfile {
            write_configfile(&job);
        } else {
            job.run();
        }
    }
}
