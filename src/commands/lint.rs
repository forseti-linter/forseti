use crate::commands::OutputFormat;
use crate::context::GlobalContext;
use anyhow::{Context, Result};
use forseti_sdk::config::Config;
use forseti_sdk::linter::EngineManager;
use std::fs;
use std::path::PathBuf;

/// Basic lint command implementation
pub fn run(
    ctx: &GlobalContext,
    path: &PathBuf,
    _fix: bool,
    recursive: bool,
    output: OutputFormat,
    output_file: Option<PathBuf>,
) -> Result<()> {
    ctx.log_verbose(&format!("Starting lint operation in: {}", path.display()));
    let config_path = ctx.resolve_config_path(path);
    ctx.log_verbose(&format!("Using config file: {}", config_path.display()));

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "No .forseti.toml found at {}. Run 'forseti init' first.",
            config_path.display()
        ));
    }

    // Load configuration
    ctx.log_verbose("Loading configuration...");
    let config = Config::load_from_path(&config_path).context("Failed to load configuration")?;

    // Initialize engine manager
    let cache_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Unable to determine home directory"))?
        .join(".forseti")
        .join("cache");

    let mut engine_manager = EngineManager::new(cache_dir);
    ctx.log_verbose("Discovering engines...");

    // Discover and start engines
    let engines = engine_manager.discover_engines()?;
    ctx.log_verbose(&format!("Found {} engine(s)", engines.len()));

    for engine in &engines {
        ctx.log_verbose(&format!(
            "Found engine: {} at {}",
            engine.id,
            engine.binary_path.display()
        ));
        if let Some(engine_cfg) = config.engine.get(&engine.id) {
            if engine_cfg.enabled {
                ctx.log_verbose(&format!(
                    "Starting engine: {} (enabled={})",
                    engine.id, engine_cfg.enabled
                ));
                ctx.log_verbose(&format!("Engine config: {:?}", engine_cfg.config));

                // Convert EngineCfg to EngineConfig
                let engine_config = forseti_sdk::engine::EngineConfig {
                    enabled: Some(engine_cfg.enabled),
                    rulesets: if engine_cfg.config.is_empty() {
                        ctx.log_verbose("No rulesets configuration found, using None");
                        None
                    } else {
                        // Look for the rulesets section in the config
                        if let Some(rulesets_table) = engine_cfg.config.get("rulesets") {
                            if let Ok(rulesets_value) = serde_json::to_value(rulesets_table) {
                                if let Some(rulesets_obj) = rulesets_value.as_object() {
                                    let rulesets: std::collections::HashMap<
                                        String,
                                        serde_json::Value,
                                    > = rulesets_obj
                                        .iter()
                                        .map(|(k, v)| {
                                            ctx.log_verbose(&format!(
                                                "Ruleset config {}: {}",
                                                k, v
                                            ));
                                            (k.clone(), v.clone())
                                        })
                                        .collect();
                                    ctx.log_verbose(&format!("Converted rulesets: {:?}", rulesets));
                                    Some(rulesets)
                                } else {
                                    ctx.log_verbose("rulesets is not an object");
                                    None
                                }
                            } else {
                                ctx.log_verbose("Failed to convert rulesets to JSON");
                                None
                            }
                        } else {
                            ctx.log_verbose("No rulesets section found in config");
                            None
                        }
                    },
                };

                ctx.log_verbose(&format!("Final engine config: {:?}", engine_config));
                match engine_manager.start_engine(&engine.id, Some(engine_config)) {
                    Ok(_) => {
                        ctx.log_verbose(&format!("Successfully started engine: {}", engine.id))
                    }
                    Err(e) => {
                        ctx.log_verbose(&format!("Failed to start engine {}: {}", engine.id, e));
                        return Err(e);
                    }
                }
            } else {
                ctx.log_verbose(&format!("Engine {} is disabled in config", engine.id));
            }
        } else {
            ctx.log_verbose(&format!(
                "No configuration found for discovered engine: {}",
                engine.id
            ));
        }
    }

    // Collect files to lint
    let files = collect_files(path, recursive)?;
    ctx.log_verbose(&format!("Found {} file(s) to lint", files.len()));

    let mut file_results = Vec::new();

    // Process files with engines
    for file_path in files {
        ctx.log_verbose(&format!("Processing: {}", file_path.display()));

        // Read file content
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let file_uri = format!("file://{}", file_path.display());

        // Try each engine that matches this file
        let mut processed_by_engine = false;
        for engine in &engines {
            if let Some(engine_cfg) = config.engine.get(&engine.id) {
                if engine_cfg.enabled {
                    ctx.log_verbose(&format!(
                        "Trying engine {} for file {}",
                        engine.id,
                        file_path.display()
                    ));
                    match engine_manager.analyze_file(&engine.id, &file_uri, &content) {
                        Ok(result) => {
                            ctx.log_verbose(&format!(
                                "Engine {} processed {} and found {} diagnostic(s)",
                                engine.id,
                                file_path.display(),
                                result.diagnostics.len()
                            ));
                            for diagnostic in &result.diagnostics {
                                ctx.log_verbose(&format!(
                                    "  Diagnostic: {} at {}:{} - {}",
                                    diagnostic.rule_id,
                                    diagnostic.range.start.line + 1,
                                    diagnostic.range.start.character + 1,
                                    diagnostic.message
                                ));
                            }
                            file_results.push((
                                file_path.clone(),
                                result.diagnostics,
                                engine.id.clone(),
                            ));
                            processed_by_engine = true;
                            break; // Use first successful engine for now
                        }
                        Err(e) => {
                            ctx.log_verbose(&format!(
                                "Engine {} failed for file {}: {}",
                                engine.id,
                                file_path.display(),
                                e
                            ));
                        }
                    }
                } else {
                    ctx.log_verbose(&format!("Engine {} is disabled", engine.id));
                }
            } else {
                ctx.log_verbose(&format!("No configuration found for engine {}", engine.id));
            }
        }

        if !processed_by_engine {
            ctx.log_verbose(&format!(
                "No engine processed file: {}",
                file_path.display()
            ));
        }
    }

    // Count total diagnostics
    let total_diagnostics = file_results
        .iter()
        .map(|(_, diags, _)| diags.len())
        .sum::<usize>();

    // Output results
    match output {
        OutputFormat::Text => {
            let mut error_count = 0;
            let mut warn_count = 0;
            let mut info_count = 0;
            let mut files_with_issues = std::collections::HashSet::new();

            for (file_path, diagnostics, engine_id) in &file_results {
                for diagnostic in diagnostics {
                    // Count diagnostics by severity
                    match diagnostic.severity.as_str() {
                        "error" => error_count += 1,
                        "warn" => warn_count += 1,
                        "info" => info_count += 1,
                        _ => warn_count += 1, // Default to warn for unknown severities
                    }

                    files_with_issues.insert(file_path.clone());

                    // For the base engine, we know the ruleset is "base", but for other engines
                    // we'd need to look up which ruleset the rule belongs to
                    let ruleset_info = if engine_id == "base" {
                        "base"
                    } else {
                        // For other engines, we'd need more sophisticated ruleset tracking
                        "unknown"
                    };

                    let docs_part = if let Some(ref docs_url) = diagnostic.docs_url {
                        format!(" ({})", docs_url)
                    } else {
                        String::new()
                    };

                    println!(
                        "{}:{}:{}: {} [{}@{}]{}",
                        file_path.display(),
                        diagnostic.range.start.line + 1,
                        diagnostic.range.start.character + 1,
                        diagnostic.message,
                        diagnostic.rule_id,
                        ruleset_info,
                        docs_part
                    );
                }
            }

            // Print summary
            if total_diagnostics > 0 {
                println!();
                println!("Summary:");
                println!("  Files checked: {}", file_results.len());
                println!("  Files with issues: {}", files_with_issues.len());
                println!("  Total issues: {}", total_diagnostics);
                if error_count > 0 {
                    println!("    Errors: {}", error_count);
                }
                if warn_count > 0 {
                    println!("    Warnings: {}", warn_count);
                }
                if info_count > 0 {
                    println!("    Info: {}", info_count);
                }
            } else {
                println!();
                println!("✓ No issues found in {} file(s)", file_results.len());
            }
        }
        OutputFormat::Json => {
            // Create a JSON output with file->diagnostics mapping
            let json_output: std::collections::HashMap<
                String,
                Vec<&forseti_sdk::core::Diagnostic>,
            > = file_results
                .iter()
                .map(|(path, diags, _)| (path.display().to_string(), diags.iter().collect()))
                .collect();
            let json = serde_json::to_string_pretty(&json_output)?;
            if let Some(output_file) = output_file {
                fs::write(output_file, json)?;
            } else {
                println!("{}", json);
            }
        }
        OutputFormat::Junit => {
            let junit_xml = generate_junit_xml(&file_results, total_diagnostics)?;
            if let Some(output_file) = output_file {
                fs::write(output_file, junit_xml)?;
            } else {
                println!("{}", junit_xml);
            }
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Output format {:?} not yet implemented",
                output
            ));
        }
    }

    // Clean shutdown
    engine_manager.shutdown_all()?;

    // Return error code if there were diagnostics
    if total_diagnostics > 0 && config.linter.fail_on_error {
        std::process::exit(1);
    }

    Ok(())
}

fn collect_files(path: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.clone());
    } else if path.is_dir() {
        if recursive {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    files.push(entry.into_path());
                }
            }
        } else {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    files.push(entry.path());
                }
            }
        }
    }

    Ok(files)
}

fn generate_junit_xml(
    file_results: &[(PathBuf, Vec<forseti_sdk::core::Diagnostic>, String)],
    total_diagnostics: usize,
) -> Result<String> {
    use std::fmt::Write;

    let mut xml = String::new();

    // XML header
    writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

    // Count statistics
    let total_files = file_results.len();
    let files_with_issues = file_results
        .iter()
        .filter(|(_, diags, _)| !diags.is_empty())
        .count();
    let failures = total_diagnostics;

    // Testsuite opening tag
    writeln!(
        xml,
        r#"<testsuite name="Forseti Linter" tests="{}" failures="{}" errors="0" skipped="{}">"#,
        total_files,
        failures,
        total_files - files_with_issues
    )?;

    // Generate test cases for each file
    for (file_path, diagnostics, engine_id) in file_results {
        let file_name = file_path.display().to_string();
        let has_issues = !diagnostics.is_empty();

        if has_issues {
            // File with issues - create failure test case
            writeln!(
                xml,
                r#"  <testcase classname="forseti.{}" name="{}" time="0">"#,
                engine_id,
                html_escape(&file_name)
            )?;

            // Add failures for each diagnostic
            for diagnostic in diagnostics {
                let ruleset_info = if engine_id == "base" {
                    "base"
                } else {
                    "unknown"
                };
                let failure_message = format!(
                    "{}:{}: {} [{}@{}]",
                    diagnostic.range.start.line + 1,
                    diagnostic.range.start.character + 1,
                    diagnostic.message,
                    diagnostic.rule_id,
                    ruleset_info
                );

                writeln!(
                    xml,
                    r#"    <failure message="{}" type="{}">{}</failure>"#,
                    html_escape(&failure_message),
                    html_escape(&diagnostic.rule_id),
                    html_escape(&diagnostic.message)
                )?;
            }

            writeln!(xml, r#"  </testcase>"#)?;
        } else {
            // File with no issues - create passing test case
            writeln!(
                xml,
                r#"  <testcase classname="forseti.{}" name="{}" time="0"/>"#,
                engine_id,
                html_escape(&file_name)
            )?;
        }
    }

    // Close testsuite
    writeln!(xml, r#"</testsuite>"#)?;

    Ok(xml)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
