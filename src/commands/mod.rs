use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod init;
pub mod install;
pub mod lint;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Junit,
    Sarif,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new .forseti.toml configuration file
    Init {
        /// Target directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Force overwrite if config already exists
        #[arg(short, long)]
        force: bool,
    },
    /// Download and install engines and rulesets from configuration
    Install {
        /// Cache directory for downloaded binaries
        #[arg(short, long, default_value = "~/.forseti/cache")]
        cache_path: PathBuf,

        /// Enable caching of downloaded binaries
        #[arg(long)]
        enable_cache: bool,

        /// Project directory containing .forseti.toml (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Force reinstall even if already exists
        #[arg(long)]
        force: bool,
    },
    /// Lint files in a directory or file path
    Lint {
        /// Path to lint (file or directory). Defaults to current directory.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Apply automatic fixes where possible (experimental)
        #[arg(long)]
        fix: bool,

        /// Recursively scan all subdirectories
        #[arg(short, long)]
        recursive: bool,

        /// Output format for results
        #[arg(short, long, default_value = "text")]
        output: OutputFormat,

        /// Write results to file (defaults to stdout)
        #[arg(long)]
        output_file: Option<PathBuf>,
    },
}
