use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = r#"# Forseti configuration
# [linter]
# log_level = "info"
# output_format = "default" # default, json, junit
# parallelism = 0
# fail_on_error = true
[engine.base]
"#;

pub fn run(path: &PathBuf, force: bool) -> Result<()> {
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
    Ok(())
}
