use anyhow::{Result, anyhow};
use crate::context::GlobalContext;
use std::fs;
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = r#"# Forseti Configuration File
# This file configures the Forseti linter with engines and rulesets

# Global linter settings
[linter]
log_level = "info"
output_format = "json"
parallelism = 0
fail_on_error = true

# Base engine configuration
[engine.base]
enabled = true

# Base engine ruleset configuration
[engine.base.rulesets.base]
"no-trailing-whitespace" = "warn"
"max-line-length" = ["warn", { limit = 120 }]
"no-empty-files" = "error"
"require-final-newline" = "warn"
"#;

pub fn run(ctx: &GlobalContext, path: &PathBuf, force: bool) -> Result<()> {
    ctx.log_verbose(&format!("Initializing Forseti config in: {}", path.display()));
    let dir = PathBuf::from(path);
    let cfg_path = dir.join(".forseti.toml");

    if cfg_path.exists() && !force {
        return Err(anyhow!(
            "Config already exists at {} (use --force to overwrite)",
            cfg_path.display()
        ));
    }

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    fs::write(&cfg_path, DEFAULT_CONFIG)?;
    println!("Initialized Forseti config at {}", cfg_path.display());
    ctx.log_verbose("Config initialization completed successfully");
    Ok(())
}
