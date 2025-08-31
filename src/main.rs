use crate::commands::Commands;
use anyhow::Result;
use clap::{Parser, command};
use std::path::PathBuf;

mod commands;
mod config;

#[derive(Parser)]
#[command(
    name = "forseti",
    version,
    about = "Forseti — a pluggable, multi-language linter",
    propagate_version = true
)]
struct Cli {
    /// Enable verbose output (reserved for future use)
    #[arg(short, long, global = true, hide = true)]
    verbose: bool,

    /// Disable colorized output (Happens automatically when an output format other than string is used)
    #[arg(short, long, global = true)]
    no_color: bool,

    /// Optional config path (otherwise default resolution is used)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, force } => commands::init::run(&path, force),
        Commands::Install {
            cache_path,
            enable_cache,
            path,
            force,
        } => commands::install::run(&cache_path, enable_cache, &path, force),
        Commands::Lint {
            path,
            fix,
            recursive,
            output,
            output_file,
        } => commands::lint::run(&path, fix, recursive, output, output_file),
    }
}
