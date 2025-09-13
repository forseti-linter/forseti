# Forseti

A fast, multi-language linter built with Rust that supports pluggable engines and rulesets.

## Installation

### Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs/))
- Git (for installing engines from repositories)

### Install Forseti

```bash
cargo install forseti
```

### Install Engines

Forseti uses pluggable engines for different languages and rule types. Start with the base engine:

```bash
# Install base engine (text linting rules)
forseti install engine base

# Install from git repository
forseti install engine custom --git https://github.com/user/custom-engine.git

# List available engines
forseti list engines
```

## Quick Start

1. **Initialize configuration in your project:**
   ```bash
   forseti init
   ```
   This creates a `.forseti.toml` configuration file.

2. **Run the linter:**
   ```bash
   forseti lint
   ```

## Configuration

Forseti uses a `.forseti.toml` file for configuration. Here's an example:

```toml
[linter]
max_workers = 4
timeout_ms = 30000
output_format = "human"

[engine.base]
enabled = true

[engine.base.rulesets.base]
"no-trailing-whitespace" = "warn"
"max-line-length" = ["warn", { "limit" = 120 }]
"no-mixed-line-endings" = "error"
```

### Rule Severity Levels

- `"off"` - Disable the rule
- `"warn"` - Show as warning
- `"error"` - Show as error (fails CI/CD)

### Rule Configuration

Some rules accept additional options:

```toml
# Simple rule with severity only
"no-trailing-whitespace" = "warn"

# Rule with options
"max-line-length" = ["warn", { "limit" = 120 }]
```

## Usage

### Basic Commands

```bash
# Lint current directory
forseti lint

# Lint specific files/directories
forseti lint src/
forseti lint main.rs lib.rs

# Lint recursively
forseti lint --recursive

# Verbose output
forseti --verbose lint
```

### Engine Management

```bash
# List installed engines
forseti list engines

# Install engine from crates.io
forseti install engine base

# Install engine from git
forseti install engine custom --git https://github.com/user/engine.git

# Install specific version/branch
forseti install engine custom --git https://github.com/user/engine.git --branch main

# Uninstall engine
forseti uninstall engine base
```

### Configuration Management

```bash
# Initialize config in current directory
forseti init

# Use custom config file
forseti --config path/to/config.toml lint
```

## Output Formats

Forseti supports multiple output formats:

```bash
# Human-readable (default)
forseti lint

# JSON output for CI/CD integration
forseti lint --format json

# Specify in config file
[linter]
output_format = "json"
```

## Common Workflows

### Local Development
```bash
# Quick check
forseti lint src/

# Comprehensive check with verbose output
forseti --verbose lint --recursive
```

### CI/CD Integration
```bash
# Install forseti and engines
cargo install forseti
forseti install engine base

# Run linting (fails on errors)
forseti lint --format json > lint-results.json
```

### Project Setup
```bash
# Initialize new project
forseti init

# Install recommended engines
forseti install engine base

# Run first lint
forseti lint
```

## Troubleshooting

### Engine Installation Issues
- Ensure Rust toolchain is up to date: `rustup update`
- Check network connectivity for git-based engines
- Use `--verbose` flag for detailed error messages

### Configuration Issues
- Validate TOML syntax in `.forseti.toml`
- Check that referenced engines are installed: `forseti list engines`
- Use `forseti --verbose lint` to see configuration loading details

### Performance
- Adjust `max_workers` in configuration for your system
- Use specific file paths instead of recursive scanning for large projects
- Consider `timeout_ms` setting for slow engines

## Getting Help

```bash
# Show help
forseti --help

# Show command-specific help
forseti lint --help
forseti install --help
```