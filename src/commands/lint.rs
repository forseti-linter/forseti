use crate::commands::OutputFormat;
use crate::context::GlobalContext;
use anyhow::{Context, Result};
use forseti_sdk::config::Config;
use forseti_sdk::core::Diagnostic;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};

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

    // Get cache directory for rulesets
    let cache_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Unable to determine home directory"))?
        .join(".forseti")
        .join("cache");

    ctx.log_verbose("Discovering rulesets...");

    // Discover available rulesets
    let rulesets = discover_rulesets(&cache_dir, &config)?;
    ctx.log_verbose(&format!("Found {} ruleset(s)", rulesets.len()));

    // Collect files to lint
    let files = collect_files(path, recursive)?;
    ctx.log_verbose(&format!("Found {} file(s) to lint", files.len()));

    let mut file_results = Vec::new();

    // Process files with rulesets
    for file_path in files {
        ctx.log_verbose(&format!("Processing: {}", file_path.display()));

        // Read file content
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let file_uri = format!("file://{}", file_path.display());

        // Try each enabled ruleset
        for ruleset in &rulesets {
            if let Some(ruleset_cfg) = config.ruleset.get(&ruleset.id) {
                if ruleset_cfg.enabled {
                    ctx.log_verbose(&format!(
                        "Trying ruleset {} for file {}",
                        ruleset.id,
                        file_path.display()
                    ));

                    match analyze_file_with_ruleset(ctx, ruleset, &file_uri, &content, &ruleset_cfg.config) {
                        Ok(diagnostics) => {
                            ctx.log_verbose(&format!(
                                "Ruleset {} processed {} and found {} diagnostic(s)",
                                ruleset.id,
                                file_path.display(),
                                diagnostics.len()
                            ));
                            for diagnostic in &diagnostics {
                                ctx.log_verbose(&format!(
                                    "  Diagnostic: {} at {}:{} - {}",
                                    diagnostic.rule_id,
                                    diagnostic.range.start.line + 1,
                                    diagnostic.range.start.character + 1,
                                    diagnostic.message
                                ));
                            }
                            if !diagnostics.is_empty() {
                                file_results.push((
                                    file_path.clone(),
                                    diagnostics,
                                    ruleset.id.clone(),
                                ));
                            }
                        }
                        Err(e) => {
                            ctx.log_verbose(&format!(
                                "Ruleset {} failed for file {}: {}",
                                ruleset.id,
                                file_path.display(),
                                e
                            ));
                        }
                    }
                } else {
                    ctx.log_verbose(&format!("Ruleset {} is disabled", ruleset.id));
                }
            } else {
                ctx.log_verbose(&format!("No configuration found for ruleset {}", ruleset.id));
            }
        }
    }

    // Count total diagnostics
    let total_diagnostics = file_results
        .iter()
        .map(|(_, diags, _)| diags.len())
        .sum::<usize>();

    // Output results
    output_results(ctx, &file_results, total_diagnostics, output, output_file)?;

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

#[derive(Debug, Clone)]
struct RulesetInfo {
    id: String,
    binary_path: PathBuf,
}

fn discover_rulesets(cache_dir: &PathBuf, config: &Config) -> Result<Vec<RulesetInfo>> {
    let mut rulesets = Vec::new();

    // First, check for rulesets configured with local paths
    for (ruleset_id, ruleset_cfg) in &config.ruleset {
        if let Some(local_path) = &ruleset_cfg.path {
            let path = PathBuf::from(local_path);
            if path.exists() && path.is_file() {
                rulesets.push(RulesetInfo {
                    id: ruleset_id.clone(),
                    binary_path: path,
                });
            }
        }
    }

    // Then, look for rulesets in cache directory
    if cache_dir.exists() {
        let entries = fs::read_dir(cache_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let bin_dir = path.join("bin");
                if bin_dir.exists() {
                    let bin_entries = fs::read_dir(bin_dir)?;
                    for bin_entry in bin_entries {
                        let bin_entry = bin_entry?;
                        let bin_path = bin_entry.path();

                        if bin_path.is_file() {
                            let file_name = bin_path.file_name().unwrap().to_string_lossy();
                            if file_name.starts_with("forseti_ruleset_") {
                                let ruleset_id = file_name
                                    .strip_prefix("forseti_ruleset_")
                                    .unwrap()
                                    .to_string();

                                // Only add if not already found via local path
                                if !rulesets.iter().any(|r| r.id == ruleset_id) {
                                    rulesets.push(RulesetInfo {
                                        id: ruleset_id,
                                        binary_path: bin_path,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(rulesets)
}

fn analyze_file_with_ruleset(
    _ctx: &GlobalContext,
    ruleset: &RulesetInfo,
    file_uri: &str,
    content: &str,
    config: &toml::value::Table,
) -> Result<Vec<Diagnostic>> {
    // Start the ruleset process
    let mut child = Command::new(&ruleset.binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start ruleset: {}", ruleset.id))?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    // Create channels for communication
    let (tx, rx) = std::sync::mpsc::channel();

    // Start thread to read responses
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if tx_clone.send(line).is_err() {
                    break;
                }
            }
        }
    });

    // Send initialization request
    let mut writer = stdin;
    let init_request = json!({
        "v": 1,
        "kind": "req",
        "type": "initialize",
        "id": "init",
        "payload": {
            "rulesetId": ruleset.id,
            "workspaceRoot": ".",
            "rulesetConfig": config
        }
    });

    writeln!(writer, "{}", serde_json::to_string(&init_request)?)?;

    // Wait for initialization response
    let init_response = rx.recv_timeout(std::time::Duration::from_secs(5))
        .context("Timeout waiting for initialization response")?;
    let _init_res: Value = serde_json::from_str(&init_response)?;

    // Send analyze file request
    let analyze_request = json!({
        "v": 1,
        "kind": "req",
        "type": "analyzeFile",
        "id": "analyze",
        "payload": {
            "uri": file_uri,
            "content": content
        }
    });

    writeln!(writer, "{}", serde_json::to_string(&analyze_request)?)?;

    // Collect diagnostics
    let mut diagnostics = Vec::new();
    let mut analyze_complete = false;

    while !analyze_complete {
        let response = rx.recv_timeout(std::time::Duration::from_secs(10))
            .context("Timeout waiting for analysis response")?;
        let msg: Value = serde_json::from_str(&response)?;

        if let Some(kind) = msg.get("kind").and_then(|k| k.as_str()) {
            match kind {
                "event" => {
                    if let Some(msg_type) = msg.get("type").and_then(|t| t.as_str()) {
                        if msg_type == "diagnostics" {
                            if let Some(payload) = msg.get("payload") {
                                if let Some(diags) = payload.get("diagnostics").and_then(|d| d.as_array()) {
                                    for diag in diags {
                                        if let Ok(diagnostic) = serde_json::from_value::<Diagnostic>(diag.clone()) {
                                            diagnostics.push(diagnostic);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "res" => {
                    if let Some(id) = msg.get("id").and_then(|i| i.as_str()) {
                        if id == "analyze" {
                            analyze_complete = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Send shutdown request
    let shutdown_request = json!({
        "v": 1,
        "kind": "req",
        "type": "shutdown",
        "id": "shutdown"
    });

    let _ = writeln!(writer, "{}", serde_json::to_string(&shutdown_request)?);

    // Wait for process to finish
    let _ = child.wait();

    Ok(diagnostics)
}

fn output_results(
    _ctx: &GlobalContext,
    file_results: &[(PathBuf, Vec<Diagnostic>, String)],
    total_diagnostics: usize,
    output: OutputFormat,
    output_file: Option<PathBuf>,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            let mut error_count = 0;
            let mut warn_count = 0;
            let mut info_count = 0;
            let mut files_with_issues = std::collections::HashSet::new();

            for (file_path, diagnostics, ruleset_id) in file_results {
                for diagnostic in diagnostics {
                    // Count diagnostics by severity
                    match diagnostic.severity.as_str() {
                        "error" => error_count += 1,
                        "warn" => warn_count += 1,
                        "info" => info_count += 1,
                        _ => warn_count += 1, // Default to warn for unknown severities
                    }

                    files_with_issues.insert(file_path.clone());

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
                        ruleset_id,
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
                Vec<&Diagnostic>,
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
            let junit_xml = generate_junit_xml(file_results, total_diagnostics)?;
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
    Ok(())
}

fn generate_junit_xml(
    file_results: &[(PathBuf, Vec<Diagnostic>, String)],
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
    for (file_path, diagnostics, ruleset_id) in file_results {
        let file_name = file_path.display().to_string();
        let has_issues = !diagnostics.is_empty();

        if has_issues {
            // File with issues - create failure test case
            writeln!(
                xml,
                r#"  <testcase classname="forseti.{}" name="{}" time="0">"#,
                ruleset_id,
                html_escape(&file_name)
            )?;

            // Add failures for each diagnostic
            for diagnostic in diagnostics {
                let failure_message = format!(
                    "{}:{}: {} [{}@{}]",
                    diagnostic.range.start.line + 1,
                    diagnostic.range.start.character + 1,
                    diagnostic.message,
                    diagnostic.rule_id,
                    ruleset_id
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
                ruleset_id,
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
