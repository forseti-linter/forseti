use anyhow::Result;
use openlinter_sdk::RunOptions;
use std::path::PathBuf;

use crate::config::LinterConfig;

pub fn run_lint(_cfg: &LinterConfig, path: PathBuf, run: RunOptions) -> Result<()> {
    eprintln!(
        "lint (stub): path: {}, recursive={}, verbosity={:?}",
        std::path::absolute(path).unwrap().display(),
        run.recursive,
        run.verbosity
    );
    // TODO: pass `run` through L2E::ConfigureEngine { run, ... }

    Ok(())
}
