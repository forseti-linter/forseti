use std::path::{Path, PathBuf};

/// Global context passed to all commands
#[derive(Debug, Clone)]
pub struct GlobalContext {
    /// Enable verbose output
    pub verbose: bool,
    /// Disable colorized output
    #[allow(dead_code)]
    pub no_color: bool,
    /// Custom config path (overrides default resolution)
    pub config_path: Option<PathBuf>,
}

impl GlobalContext {
    pub fn new(verbose: bool, no_color: bool, config_path: Option<PathBuf>) -> Self {
        Self {
            verbose,
            no_color,
            config_path,
        }
    }

    /// Get the config path to use (either custom or default)
    pub fn resolve_config_path(&self, base_path: &Path) -> PathBuf {
        if let Some(config) = &self.config_path {
            config.clone()
        } else {
            base_path.join(".forseti.toml")
        }
    }

    /// Log verbose message if verbose mode is enabled
    pub fn log_verbose(&self, message: &str) {
        if self.verbose {
            eprintln!("[VERBOSE] {}", message);
        }
    }
}