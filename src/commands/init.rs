//! `openlinter init`
//! - Ensures .openlinter/ folder structure
//! - Normalizes sources for engines and rulesets
//! - (Stub) Prints what it *would* install and where
//! - (TODO) Implement download/extract, --plugin-info probe, and lockfile write

use crate::config::{LinterConfig, Source, parse_ruleset_key};
use anyhow::{Context, Result};
use openlinter_sdk::RunOptions;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Plan {
    engines: BTreeMap<String, InstallItem>,
    rulesets: BTreeMap<String, InstallItem>, // key: "<engine>-<ruleset>"
}

#[allow(dead_code)]
#[derive(Clone)]
struct InstallItem {
    id: String,    // engine id or "<engine>-<ruleset>"
    src: Source,   // normalized source hint
    dest: PathBuf, // where it will be installed
    kind: ItemKind,
}

#[derive(Clone, Copy)]
enum ItemKind {
    Engine,
    Ruleset,
}

pub fn run_init(cfg: &LinterConfig, project_root: &Path, _run: RunOptions) -> Result<()> {
    let base = project_root.join(".openlinter");
    let engines_dir = base.join("bin/engines");
    let rulesets_dir = base.join("bin/rulesets");

    // 1) Make folders
    std::fs::create_dir_all(&engines_dir)
        .with_context(|| format!("creating {}", engines_dir.display()))?;
    std::fs::create_dir_all(&rulesets_dir)
        .with_context(|| format!("creating {}", rulesets_dir.display()))?;

    // 2) Build a plan (normalize sources + compute destinations)
    let mut plan = Plan::default();

    for (engine_id, decl) in &cfg.engines {
        let src = normalize_source_engine(engine_id, decl);
        let dest = engines_dir.join(engine_id).join("engine");
        plan.engines.insert(
            engine_id.clone(),
            InstallItem {
                id: engine_id.clone(),
                src,
                dest,
                kind: ItemKind::Engine,
            },
        );
    }

    for (key, decl) in &cfg.rulesets {
        let src = normalize_source_ruleset(key, decl)?;
        let (eng, rs) = parse_ruleset_key(key)?;
        let dest = rulesets_dir.join(eng).join(rs);
        plan.rulesets.insert(
            key.clone(),
            InstallItem {
                id: key.clone(),
                src,
                dest,
                kind: ItemKind::Ruleset,
            },
        );
    }

    // 3) Show the plan (stub)
    eprintln!("OpenLinter init plan:\n  base: {}", base.display());

    if plan.engines.is_empty() {
        eprintln!("  (no engines declared under [engines])");
    } else {
        eprintln!("  engines:");
        for (id, it) in &plan.engines {
            eprintln!(
                "    - {id} -> {}  (source: {})",
                it.dest.display(),
                describe_source(&it.src)
            );
        }
    }

    if plan.rulesets.is_empty() {
        eprintln!("  (no rulesets declared under [rulesets])");
    } else {
        eprintln!("  rulesets:");
        for (key, it) in &plan.rulesets {
            eprintln!(
                "    - {key} -> {}  (source: {})",
                it.dest.display(),
                describe_source(&it.src)
            );
        }
    }

    // 4) TODO: For each InstallItem:
    //    - fetch/copy artifact (path/src/github/gitlab)
    //    - make executable
    //    - run `--plugin-info` to probe (id/version/engine_target/api ranges)
    //    - validate: ruleset key matches probe (engine_target + id)
    //    - compute checksum
    //    - write/update .openlinter/lock.toml with probed facts and paths
    //    - optionally cache manifests in .openlinter/manifests/

    // For now, just succeed after printing the plan.
    Ok(())
}

fn describe_source(s: &Source) -> String {
    if let Some(p) = &s.path {
        return format!("path: {p}");
    }
    if let Some(u) = &s.git {
        return format!("git: {u}");
    }

    format!("git: {}", s.name)
}
