//! Config types + helpers for openlinter.toml
//!
//! - Cargo-style top-level sections: [engines], [rulesets]
//! - Ruleset keys use "<engine>-<ruleset>" (hyphen separator)
//! - Runtime overrides under [engine.<id>] and [engine.<id>.ruleset.<rid>]
//!
//! This module only parses/normalizes; install/probe/lock lives in `init`.

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value as Json;
use std::collections::BTreeMap;

/// Top-level config for openlinter.toml
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct LinterConfig {
    #[serde(default)]
    pub profile: Option<String>,

    // What to install (Cargo-like)
    #[serde(default)]
    pub engines: BTreeMap<String, Decl>,

    #[serde(default)]
    pub rulesets: BTreeMap<String, Decl>, // keys like "terraform-unused"

    // Which engines to run and runtime overrides
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,

    #[serde(default)]
    pub engine: BTreeMap<String, EngineRuntime>,

    #[serde(default)]
    pub files: Files,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub engines: Vec<String>,
    #[serde(default)]
    pub engine: BTreeMap<String, EngineProfileOverride>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct Files {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Dependency-style declaration: either "1.2.3" or a detailed table.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Decl {
    #[allow(dead_code)]
    VersionOnly(String),
    #[allow(dead_code)]
    Detailed(Source),
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Source {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub refr: Option<String>,
    #[serde(default)]
    pub path: Option<String>, // local path (dev)
    #[serde(default)]
    pub checksum: Option<String>, // e.g., "sha256:..."
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
pub struct EngineRuntime {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config: Json,
    #[serde(default)]
    pub ruleset: BTreeMap<String, RulesetRuntime>, // short ruleset ids (after the hyphen)
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Clone)]
pub struct EngineProfileOverride {
    #[serde(default)]
    pub config: Json,
    #[serde(default)]
    pub ruleset: BTreeMap<String, RulesetRuntime>,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Clone)]
pub struct RulesetRuntime {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config: Json,
}

/// Split "<engine>-<ruleset>" into ("engine","ruleset").
/// Ruleset id may contain additional hyphens; only the first is used to split.
#[allow(dead_code)]
pub fn parse_ruleset_key(key: &str) -> Result<(&str, &str)> {
    let mut it = key.splitn(2, '-');
    let eng = it.next().unwrap_or_default();
    let rs = it.next().unwrap_or_default();
    anyhow::ensure!(
        !eng.is_empty() && !rs.is_empty(),
        "invalid ruleset key '{key}', expected '<engine>-<ruleset>'"
    );
    Ok((eng, rs))
}

/// Convenience that reads a file and parses `LinterConfig`.
#[allow(dead_code)]
pub fn load_config(path: &std::path::Path) -> Result<LinterConfig> {
    let txt = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let cfg: LinterConfig =
        toml::from_str(&txt).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(cfg)
}
