use crate::commands::Commands;
use anyhow::Result;
use clap::{Parser, command};
use std::path::PathBuf;

mod commands;
mod context;

use context::GlobalContext;

#[derive(Parser)]
#[command(
    name = "forseti",
    version,
    about = "Forseti — a pluggable, multi-language linter",
    propagate_version = true
)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
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
    
    // Create global context from CLI args
    let ctx = GlobalContext::new(cli.verbose, cli.no_color, cli.config);

    match cli.command {
        Commands::Init { path, force } => commands::init::run(&ctx, &path, force),
        Commands::Install {
            cache_path,
            enable_cache,
            path,
            force,
        } => commands::install::run(&ctx, &cache_path, enable_cache, &path, force),
        Commands::Lint {
            path,
            fix,
            recursive,
            output,
            output_file,
        } => commands::lint::run(&ctx, &path, fix, recursive, output, output_file),
    }
}
