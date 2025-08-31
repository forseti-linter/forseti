use anyhow::Result;
use std::path::PathBuf;

use crate::commands::OutputFormats;

/// Placeholder lint command.
/// Later, this will:
///  - Load config (from --config or default resolution)
///  - Load engines (from config)
///  - Load rulesets per engine
///  - Walk target path, route files to engines
///  - Aggregate and print results; return non-zero on errors
pub fn run(
    path: &PathBuf,
    fix: bool,
    recursive: bool,
    output: OutputFormats,
    output_file: PathBuf,
) -> Result<()> {
    println!("Forseti lint");
    println!("       path: {:#?}", path);
    println!("        fix: {}", fix);
    println!("  recursive: {}", recursive);
    println!("     output: {:#?}", output);
    println!("output_file: {:#?}", output_file);

    // TODO:
    // - Parse config
    // - Initialize engine processes (JSON over stdio)
    // - Send files / receive diagnostics
    // - Apply fixes if --fix
    // - Report summary and exit code

    Ok(())
}
