use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "why")]
#[command(
    author,
    version,
    about = "Identify which package manager installed a command"
)]
pub struct Cli {
    /// The command to investigate
    pub command: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Output as JSON (shortcut for --format json)
    #[arg(long, conflicts_with = "format")]
    pub json: bool,

    /// Show detailed package information
    #[arg(short = 'i', long)]
    pub info: bool,

    /// Verbose output (show detection steps)
    #[arg(short, long)]
    pub verbose: bool,

    /// Skip package manager verification queries
    #[arg(long)]
    pub no_verify: bool,

    /// Path to custom database file
    #[arg(long)]
    pub database: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output
    Json,
    /// Just the package manager name
    Short,
}
