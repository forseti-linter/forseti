use crate::context::GlobalContext;
use anyhow::{Context, Result, anyhow};
use forseti_sdk::config::{Config, RulesetCfg};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

pub fn run(
    ctx: &GlobalContext,
    cache_path: &Path,
    enable_cache: bool,
    path: &Path,
    force: bool,
) -> Result<()> {
    let config_path = ctx.resolve_config_path(path);
    ctx.log_verbose(&format!("Using config file: {}", config_path.display()));

    if !config_path.exists() {
        return Err(anyhow!(
            "No .forseti.toml found at {}. Run 'forseti init' first.",
            path.display()
        ));
    }

    if !ctx.verbose {
        println!("Loading configuration from {}...", config_path.display());
    }
    let config = Config::load_from_path(&config_path).context("Failed to load configuration")?;

    let cache_dir = if enable_cache {
        Some(cache_path.to_path_buf())
    } else {
        None
    };

    install_dependencies(&config, cache_dir.as_ref(), force)?;

    println!("Everything installed successfully!");
    Ok(())
}

fn install_dependencies(config: &Config, cache_dir: Option<&PathBuf>, force: bool) -> Result<()> {
    println!("Installing rulesets...");
    for (ruleset_id, ruleset_cfg) in &config.ruleset {
        if ruleset_cfg.enabled {
            install_ruleset(ruleset_id, ruleset_cfg, cache_dir, force)
                .with_context(|| format!("Failed to install ruleset '{}'", ruleset_id))?;
        } else {
            println!("Skipping disabled ruleset: {}", ruleset_id);
        }
    }

    Ok(())
}


fn install_ruleset(
    id: &str,
    cfg: &RulesetCfg,
    cache_dir: Option<&PathBuf>,
    force: bool,
) -> Result<()> {
    println!("Installing ruleset: {}", id);

    if let Some(local_path) = &cfg.path {
        install_from_local("ruleset", id, local_path, cache_dir, force)?;
    } else if let Some(git_url) = &cfg.git {
        install_from_git("ruleset", id, git_url, cache_dir, force)?;
    } else {
        install_from_crates_io("ruleset", id, cache_dir, force)?;
    }

    Ok(())
}

fn install_from_local(
    component_type: &str,
    id: &str,
    local_path: &str,
    cache_dir: Option<&PathBuf>,
    force: bool,
) -> Result<()> {
    println!("  Installing from local path: {}", local_path);

    let cache_path = get_cache_path(cache_dir, id)?;
    let binary_name = format!("forseti_{}_{}", component_type, id);
    let binary_path = cache_path.join("bin").join(&binary_name);

    // Check if binary already exists
    if binary_path.exists() && !force {
        println!("  Binary already exists (use --force to overwrite)");
        return Ok(());
    }

    let source_path = Path::new(local_path);
    if !source_path.exists() {
        return Err(anyhow!("Local path does not exist: {}", local_path));
    }

    if !source_path.is_file() {
        return Err(anyhow!("Local path is not a file: {}", local_path));
    }

    // Check if source is executable (on Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(source_path)?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(anyhow!("Local file is not executable: {}", local_path));
        }
    }

    // Create destination directory
    fs::create_dir_all(binary_path.parent().unwrap())?;

    // Copy the binary to the cache location
    fs::copy(source_path, &binary_path)?;

    // Make sure it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(&binary_path, perms)?;
    }

    println!("  Copied and installed to: {}", binary_path.display());
    Ok(())
}

fn install_from_git(
    component_type: &str,
    id: &str,
    git_url: &str,
    cache_dir: Option<&PathBuf>,
    force: bool,
) -> Result<()> {
    println!("  Installing from git: {}", git_url);

    let cache_path = get_cache_path(cache_dir, id)?;
    let repo_path = cache_path.join(format!("{}-repo", id));
    let binary_name = format!("forseti_{}_{}", component_type, id);
    let binary_path = cache_path.join("bin").join(&binary_name);

    // Check if binary already exists
    if binary_path.exists() && !force {
        println!("  Binary already exists (use --force to overwrite)");
        return Ok(());
    }

    // Clone or update repository
    if repo_path.exists() && !force {
        println!("  Repository already exists, pulling latest changes...");
        let output = Command::new("git")
            .args(["pull"])
            .current_dir(&repo_path)
            .output()
            .context("Failed to run git pull")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to pull from git: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    } else {
        if repo_path.exists() {
            fs::remove_dir_all(&repo_path)?;
        }
        fs::create_dir_all(&cache_path)?;

        println!("  Cloning Rust project repository...");
        let output = Command::new("git")
            .args(["clone", git_url, repo_path.to_str().unwrap()])
            .output()
            .context("Failed to run git clone. Make sure git is installed.")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to clone from git: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    // Verify this is a Rust project
    let cargo_toml = repo_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(anyhow!(
            "Repository does not contain a Cargo.toml file. Expected a Rust project."
        ));
    }

    // Build with cargo
    println!("  Building Rust project with cargo...");
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&repo_path)
        .output()
        .context("Failed to run cargo build")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to build Rust project: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Find the built binary in target/release
    let release_dir = repo_path.join("target").join("release");
    if !release_dir.exists() {
        return Err(anyhow!("Release directory not found after build"));
    }

    // Look for executable files in the release directory
    let entries = fs::read_dir(&release_dir)?;
    let mut binary_found = false;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();

            // Skip known non-binary files
            if file_name.ends_with(".d") || file_name.contains(".rlib") {
                continue;
            }

            // Check if it's executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&path)?;
                if metadata.permissions().mode() & 0o111 != 0 {
                    // Copy and rename the binary to our standardized name
                    fs::create_dir_all(binary_path.parent().unwrap())?;
                    fs::copy(&path, &binary_path)?;
                    binary_found = true;
                    break;
                }
            }

            #[cfg(not(unix))]
            {
                // On Windows, look for .exe files or assume first file is executable
                if file_name.ends_with(".exe") || !binary_found {
                    fs::create_dir_all(binary_path.parent().unwrap())?;
                    fs::copy(&path, &binary_path)?;
                    binary_found = true;
                    if file_name.ends_with(".exe") {
                        break;
                    }
                }
            }
        }
    }

    if !binary_found {
        return Err(anyhow!(
            "No executable binary found after building Rust project"
        ));
    }

    println!("  Built and installed to: {}", binary_path.display());
    Ok(())
}

fn install_from_crates_io(
    component_type: &str,
    id: &str,
    cache_dir: Option<&PathBuf>,
    force: bool,
) -> Result<()> {
    println!("  Installing from crates.io: {}", id);

    let cache_path = get_cache_path(cache_dir, id)?;
    let binary_name = format!("forseti_{}_{}", component_type, id);
    let binary_path = cache_path.join("bin").join(&binary_name);

    // Check if binary already exists
    if binary_path.exists() && !force {
        println!("  Binary already exists (use --force to overwrite)");
        return Ok(());
    }

    fs::create_dir_all(&cache_path)?;

    // First try to use cargo-binstall for precompiled binaries
    println!("  Attempting to download precompiled binary...");
    let binstall_result = try_cargo_binstall(id, &cache_path, force);

    match binstall_result {
        Ok(_) => {
            // Find the installed binary and rename it to our standard format
            let bin_dir = cache_path.join("bin");
            if bin_dir.exists() {
                let entries = fs::read_dir(&bin_dir)?;
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.file_name().unwrap().to_string_lossy() != binary_name
                    {
                        // Rename to our standard format
                        fs::rename(&path, &binary_path)?;
                        println!("  Downloaded and renamed to: {}", binary_path.display());
                        return Ok(());
                    }
                }
            }

            // If we can't find the binary after binstall, fall back to building
            println!("  Precompiled binary not found, falling back to building from source...");
        }
        Err(_) => {
            println!("  Precompiled binary not available, building from source...");
        }
    }

    // Fallback to cargo install (build from source)
    let mut args = vec!["install", id];

    if force {
        args.push("--force");
    }

    let cache_path_str = cache_path.to_string_lossy().to_string();
    args.extend(["--root", &cache_path_str]);

    let output = Command::new("cargo")
        .args(&args)
        .output()
        .context("Failed to run cargo install")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to install from crates.io: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Find the installed binary and rename it to our standard format
    let bin_dir = cache_path.join("bin");
    if bin_dir.exists() {
        let entries = fs::read_dir(&bin_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap().to_string_lossy() != binary_name {
                // Rename to our standard format
                fs::rename(&path, &binary_path)?;
                break;
            }
        }
    }

    println!("  Built and installed to: {}", binary_path.display());
    Ok(())
}

fn try_cargo_binstall(crate_name: &str, install_path: &Path, force: bool) -> Result<()> {
    let mut args = vec!["binstall", crate_name, "-y"];

    if force {
        args.push("--force");
    }

    let install_path_str = install_path.to_string_lossy().to_string();
    args.extend(["--install-path", &install_path_str]);

    let output = Command::new("cargo")
        .args(&args)
        .output()
        .context("cargo-binstall not available")?;

    if !output.status.success() {
        return Err(anyhow!(
            "cargo-binstall failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn get_cache_path(cache_dir: Option<&PathBuf>, id: &str) -> Result<PathBuf> {
    let base_path = if let Some(cache) = cache_dir {
        cache.clone()
    } else {
        // Default cache location
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .context("Could not determine home directory")?;
        PathBuf::from(home).join(".forseti").join("cache")
    };

    Ok(base_path.join(id))
}
