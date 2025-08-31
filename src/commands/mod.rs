use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod init;
pub mod install;
pub mod lint;

/// Doc comment
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormats {
    String,
    Junit,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generates a new .forseti.toml with the base engine and its built in rulesets
    Init {
        /// Target directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Force overwrite if config already exists
        #[arg(short, long)]
        force: bool,
    },
    /// Initialize a Forseti config in the current directory (or provided path)
    Install {
        /// Target directory
        #[arg(short, long, default_value = "~/.forseti/cache")]
        cache_path: PathBuf,

        /// Force overwrite if config already exists
        #[arg(long)]
        enable_cache: bool,

        /// Target directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Force overwrite if config already exists
        #[arg(long)]
        force: bool,
    },
    /// Lint files in a directory or file path
    Lint {
        /// Path to lint (file or directory). Defaults to current directory.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Apply automatic fixes where possible
        #[arg(long, hide = true)]
        fix: bool,

        ///  Looks through all subdirectories recursively (if the engine supports it)
        #[arg(short, long)]
        recursive: bool,

        ///  Specifies the output format
        #[arg(short, long, required = false, default_value = "string")]
        output: OutputFormats,

        ///  Specifies the output format
        #[arg(long, required = false, default_value = "string")]
        output_file: PathBuf,
    },
}
