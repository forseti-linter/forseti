# CLAUDE.md — Forseti CLI

This file provides guidance to Claude Code (claude.ai/code) when working with the Forseti main CLI.

## Workspace Context

**This is part of the Forseti workspace at:** `/home/digitalfiz/projects/forseti/`

- **Workspace root:** `../` (contains workspace `Cargo.toml` and main `CLAUDE.md`)
- **This CLI:** Current directory (`forseti/`)
- **SDK:** `../forseti-sdk/` (check `../forseti-sdk/CLAUDE.md` for detailed SDK info)
- **Base engine:** `../forseti-engine-base/`

## Purpose

The main Forseti CLI is the user-facing interface for the multi-language linter. It orchestrates engines, manages configurations, and provides commands for installation, linting, and system management.

## Architecture

- **Binary name:** `forseti`
- **CLI framework:** Custom command structure with `GlobalContext`
- **Engine management:** Uses SDK's `EngineManager` for engine lifecycle
- **Configuration:** Uses SDK's config system with TOML format

## Key Components

### Global Context System

The CLI uses a `GlobalContext` pattern for consistent flag handling:

```rust
pub struct GlobalContext {
    pub verbose: bool,
    pub config_path: Option<PathBuf>,
}
```

All commands receive `&GlobalContext` to access global flags like `--verbose` and `--config`.

### Commands

#### Install Command (`src/commands/install.rs`)
- **Purpose:** Install engines and rulesets from various sources
- **Sources:** crates.io, git repositories, local paths
- **Binary naming:** `forseti_<type>_<id>` convention
- **Features:** 
  - Uses `cargo-binstall` for precompiled binaries when available
  - Falls back to `cargo install` for source compilation
  - Supports git references (branches, tags, commits)

#### Lint Command (`src/commands/lint.rs`)
- **Purpose:** Run linting across files using installed engines
- **Flow:** Uses enhanced SDK architecture with capabilities and preprocessing
- **Engine discovery:** Automatically finds installed engines
- **Memory efficient:** Leverages SDK's on-demand loading

#### List Command (`src/commands/list.rs`)
- **Purpose:** Show installed engines and rulesets
- **Discovery:** Scans cache directories for engine binaries

#### Uninstall Command (`src/commands/uninstall.rs`)
- **Purpose:** Remove installed engines and rulesets
- **Safety:** Confirms before deletion

### Configuration

The CLI uses the SDK's configuration system:

```toml
[linter]
max_workers = 4
timeout_ms = 30000
output_format = "human"

[engine.base]
enabled = true
git = "https://github.com/user/forseti-engine-base.git"

[engine.base.rulesets.base]
"no-trailing-whitespace" = "warn"
"max-line-length" = ["warn", { "limit" = 120 }]
```

## Command Structure

### Main Entry Point
- `src/main.rs` - Sets up CLI with clap, handles global flags
- Uses `GlobalContext` to pass global state to commands

### Command Pattern
Each command follows this pattern:
1. Accepts `&GlobalContext` parameter
2. Handles command-specific arguments
3. Uses SDK components for core functionality
4. Returns `anyhow::Result<()>`

### Error Handling
- Uses `anyhow` for error chaining
- Verbose logging controlled by global `--verbose` flag
- Proper exit codes for CI/CD integration

## Development

### Build
```bash
cargo build -p forseti                  # Build CLI
cargo build -p forseti --release        # Release build
```

### Test
```bash
cargo test -p forseti                   # Run tests
```

### Run
```bash
cargo run -p forseti -- --help          # Show help
cargo run -p forseti -- install base    # Install base engine
cargo run -p forseti -- lint src/       # Lint files
```

## Enhanced Architecture Integration

The CLI integrates with the enhanced SDK architecture:

### Engine Discovery
- Automatically discovers installed engines
- Queries engine capabilities for file routing
- Manages engine lifecycle (start, initialize, shutdown)

### Memory-Efficient Processing
- Routes files to appropriate engines based on patterns
- Uses preprocessing for lightweight context gathering
- Executes rulesets with on-demand content loading

### Result Aggregation
- Collects diagnostics from multiple engines
- Provides unified output formatting
- Supports multiple output formats (human, json, etc.)

## Key Files

- `src/main.rs` - CLI entry point and global flag handling
- `src/context.rs` - GlobalContext definition
- `src/commands/` - Command implementations
  - `install.rs` - Engine/ruleset installation
  - `lint.rs` - File linting orchestration
  - `list.rs` - Show installed components
  - `uninstall.rs` - Remove components
- `Cargo.toml` - Dependencies and binary configuration

## Usage Examples

```bash
# Install from crates.io
forseti install engine base

# Install from git repository
forseti install engine custom --git https://github.com/user/custom-engine.git

# Lint files with verbose output
forseti --verbose lint src/

# List installed engines
forseti list engines

# Uninstall engine
forseti uninstall engine base
```

## Testing

The CLI includes integration tests that verify:
- Command parsing and execution
- Engine installation and discovery
- Configuration loading and merging
- Error handling and reporting

## Future Enhancements

- Plugin system for custom formatters
- IDE integration support
- Parallel engine execution
- Advanced filtering and ignore patterns
- Configuration validation and migration tools