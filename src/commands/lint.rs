use anyhow::Result;
use crate::commands::OutputFormat;
use crate::context::GlobalContext;
use std::path::PathBuf;

/// Placeholder lint command.
/// Later, this will:
///  - Load config (from --config or default resolution)
///  - Load engines (from config)
///  - Load rulesets per engine
///  - Walk target path, route files to engines
///  - Aggregate and print results; return non-zero on errors
pub fn run(
    ctx: &GlobalContext,
    path: &PathBuf,
    fix: bool,
    recursive: bool,
    output: OutputFormat,
    output_file: Option<PathBuf>,
) -> Result<()> {
    ctx.log_verbose(&format!("Starting lint operation in: {}", path.display()));
    let config_path = ctx.resolve_config_path(path);
    ctx.log_verbose(&format!("Using config file: {}", config_path.display()));
    
    println!("Forseti lint");
    println!("       path: {:#?}", path);
    println!("        fix: {}", fix);
    println!("  recursive: {}", recursive);
    println!("     output: {:#?}", output);
    println!("output_file: {:#?}", output_file);
    println!("config_path: {:#?}", config_path);

    // TODO:
    // - Parse config
    // - Initialize engine processes (JSON over stdio)
    // - Send files / receive diagnostics
    // - Apply fixes if --fix
    // - Report summary and exit code

    Ok(())
}
