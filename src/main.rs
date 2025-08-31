use crate::config::VERSION;
use anyhow::Result;
// use clap::{ArgAction, Parser, Subcommand};
// use openlinter_sdk::{RunOptions, Verbosity};
// use std::path::PathBuf;

// use crate::config::LinterConfig;

mod config;

// #[path = "commands/init.rs"]
// mod init;

// #[path = "commands/lint.rs"]
// mod lint;

// #[derive(Debug, Parser)]
// #[command(name = "openlinter", version, about = "OpenLinter CLI")]
// struct Cli {
//     /// Path to openlinter.toml (defaults to ./openlinter.toml)
//     #[arg(short, long, default_value = "openlinter.toml")]
//     config: PathBuf,

//     /// Project root directory (where .openlinter/ will live)
//     #[arg(short, long, default_value = ".")]
//     project_root: PathBuf,

//     /// Recurse into directories when scanning (default: false)
//     #[arg(short = 'r', long, action = ArgAction::SetTrue)]
//     recursive: bool,

//     /// Increase verbosity (-v, -vv). Use -q for quiet.
//     #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
//     verbose: u8,

//     /// Quiet mode (overrides -v)
//     #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue)]
//     quiet: bool,

//     #[command(subcommand)]
//     command: Commands,
// }

// #[derive(Debug, Subcommand)]
// enum Commands {
//     /// Install/lock engines and rulesets into .openlinter/
//     Init,

//     /// (stub) Run lint using the configured engines
//     Lint {
//         /// Files or directories to scan (optional; falls back to [files] globs)
//         #[arg()]
//         path: PathBuf,
//     },
// }

fn main() -> Result<()> {
    // let cli = Cli::parse();

    println!("OpenLinter v{}", VERSION);

    // // Load and parse openlinter.toml
    // let cfg_text = std::fs::read_to_string(&cli.config)
    //     .with_context(|| format!("failed to read {}", cli.config.display()))?;
    // let cfg: LinterConfig = toml::from_str(&cfg_text)
    //     .with_context(|| format!("failed to parse {}", cli.config.display()))?;

    // // Map -q/-v* to Verbosity
    // let verbosity = if cli.quiet {
    //     Verbosity::Quiet
    // } else {
    //     match cli.verbose {
    //         0 => Verbosity::Normal,
    //         1 => Verbosity::Verbose,
    //         _ => Verbosity::Trace,
    //     }
    // };

    // let run = RunOptions {
    //     recursive: cli.recursive,
    //     verbosity,
    // };

    // match cli.command {
    //     Commands::Init => init::run_init(&cfg, &cli.project_root, run),
    //     Commands::Lint { path } => lint::run_lint(&cfg, path, run),
    // }

    return Ok(());
}
