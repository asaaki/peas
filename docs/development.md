# Development Guide

## Prerequisites

- Rust (latest stable, edition 2024)
- [just](https://github.com/casey/just) task runner

## Quick Start

```bash
git clone https://github.com/asaaki/peas
cd peas
just dev          # Lint + test
```

## Task Runner (`just`)

Always prefer `just` recipes over manual commands.

### Essential Recipes

| Recipe | Command | Description |
|--------|---------|-------------|
| `just dev` | lint + test | Full development workflow |
| `just build` | `cargo build` | Debug build |
| `just build-release` | `cargo build --release` | Release build (LTO, stripped) |
| `just test` | `cargo test` | Run all tests |
| `just test-verbose` | `cargo test -- --nocapture` | Tests with output |
| `just lint` | fmt + clippy | Check formatting and lints |
| `just fix` | clippy-fix + fmt | Auto-fix lint issues |
| `just ci` | lint + test + release build | Full CI check |
| `just check` | `cargo check` | Type check without building |
| `just clean` | `cargo clean` | Clean build artifacts |

### Running

| Recipe | Description |
|--------|-------------|
| `just tui` | Launch TUI |
| `just prime` | Output agent instructions |
| `just list` | List all peas |
| `just serve` | Start GraphQL server |

### Quality & Analysis

| Recipe | Description |
|--------|-------------|
| `just coverage` | Code coverage with cargo-tarpaulin |
| `just audit` | Security audit of dependencies |
| `just outdated` | Check for outdated dependencies |
| `just doc` | Generate documentation |

### Release

| Recipe | Description |
|--------|-------------|
| `just version` | Show current version |
| `just version-patch` | Bump patch version |
| `just version-minor` | Bump minor version |
| `just version-major` | Bump major version |
| `just release` | Create a release |
| `just publish-check` | Dry-run publish check |
| `just install` | Install locally |

Run `just --list` for the complete list.

## Project Structure

```
peas/
├── src/                    Source code (see architecture.md)
├── tests/
│   └── cli_tests.rs        Integration tests
├── docs/                   Documentation
├── schemas/                JSON Schema for config validation
├── .peas/                  Project's own issue tracking
├── .github/                CI/CD workflows
├── Cargo.toml              Dependencies and build config
├── justfile                Task runner recipes
├── CLAUDE.md               AI agent instructions
├── README.md               Project README
├── LICENSE-MIT             MIT license
└── LICENSE-APACHE          Apache 2.0 license
```

## Testing

### Unit Tests

In-module tests using `#[cfg(test)]`:

```bash
# Run all tests
just test

# Run a specific test
cargo test test_name

# Run with output
just test-verbose
```

Key test modules:
- `model/types.rs` — enum parsing and serialization
- `search.rs` — query parsing and matching
- `validation.rs` — input validation rules
- `undo.rs` — undo stack operations
- `config.rs` — configuration loading

### Integration Tests

Located in `tests/cli_tests.rs`, using `assert_cmd` and `tempfile`:

```rust
// Pattern: init in temp dir → run command → assert output
#[test]
fn test_create_pea() {
    let dir = TempDir::new().unwrap();
    // Init project
    Command::cargo_bin("peas").unwrap()
        .arg("init")
        .current_dir(&dir)
        .assert()
        .success();
    // Create a pea
    Command::cargo_bin("peas").unwrap()
        .args(["create", "Test ticket", "-t", "bug"])
        .current_dir(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));
}
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` 4.6 | CLI argument parsing with derive macros |
| `ratatui` 0.30 | Terminal UI framework |
| `async-graphql` 7.2 | GraphQL schema and execution |
| `axum` 0.8 | HTTP server for GraphQL |
| `tokio` 1.50 | Async runtime |
| `serde` 1.0 | Serialization/deserialization |
| `chrono` 0.4 | Date/time handling |
| `notify` 8.2 | File system watcher (TUI) |
| `regex` 1.12 | Pattern matching (search) |
| `nanoid` 0.4 | Random ID generation |
| `thiserror` 2.0 | Error type derive |
| `tracing` 0.1 | Structured logging |
| `rat-text` 3.1 | Text area widget for TUI |
| `tachyonfx` 0.25 | TUI visual effects |
| `reqwest` 0.13 | HTTP client (update checker) |
| `colored` 3.1 | Terminal color output |

## Build Configuration

Release builds use:
- **LTO** (Link-Time Optimization) enabled
- **Stripped** binaries for smaller size

```toml
[profile.release]
lto = true
strip = true
```

## Error Handling

The project uses `thiserror` for error types and `anyhow` for handler-level error propagation:

```rust
// src/error.rs
pub enum PeasError {
    Config(String),
    NotFound(String),
    InvalidId(String),
    Storage(String),
    Parse(String),
    Validation(String),
    Io(std::io::Error),
    NotInitialized,
    AlreadyInitialized(String),
    // ...
}

// Handlers return anyhow::Result<()>
pub fn handle_create(ctx: &CommandContext, args: &CreateArgs) -> anyhow::Result<()> {
    // ...
}
```

## Windows Compatibility

The project runs on Windows (including WSL2). A notable fix: the main function spawns an 8MB stack thread to avoid stack overflow on Windows, where the default stack size is smaller than Linux.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run `just dev` to verify lint + tests pass
4. Include relevant pea IDs in commit messages (e.g., `peas-abc12`)
5. Submit a pull request
