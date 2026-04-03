# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Before You Do Anything

Run `peas prime` (or `just prime`) and follow its output. The `peas` binary may be installed globally; if not, use `cargo run -- <command>` from the project root. Use `cargo run --` when testing features under development.

## Task Runner

This project uses `just`. Always prefer `just` recipes over manual commands.

| Task | Command |
|------|---------|
| Full dev workflow (lint + test) | `just dev` |
| Build | `just build` / `just build-release` |
| Test | `just test` / `just test-verbose` |
| Run single test | `cargo test <test_name>` |
| Lint + format | `just lint` |
| Auto-fix lint issues | `just fix` |
| Full CI check | `just ci` |
| Run TUI | `just tui` |

Run `just --list` to see all available recipes.

## Architecture Overview

Peas is a flat-file issue tracker CLI. All data lives in the `.peas/` directory as markdown files with TOML frontmatter. There is no database.

### Data Flow

```
CLI args (clap) → CommandContext → Handler → PeaRepository → .peas/*.md files
```

- **`main.rs`**: Parses args, sets up logging, routes to handler
- **`cli/commands.rs`**: All clap command/arg definitions
- **`cli/handlers/`**: One module per subcommand; all receive `CommandContext`
- **`cli/mod.rs`**: `CommandContext` struct (holds config, repo, asset_manager, root path)
- **`storage/repository.rs`**: `PeaRepository` — CRUD, ID generation, in-memory cache
- **`storage/markdown.rs`**: Parses/serializes markdown + TOML/YAML frontmatter
- **`model/pea.rs`**: `Pea` struct; `model/types.rs` has enums (PeaType, PeaStatus, PeaPriority)

### Ticket File Format

```markdown
+++
id = "peas-abc12"
title = "Example"
type = "bug"
status = "todo"
priority = "normal"
tags = []
blocking = []
assets = []
created = "2024-01-01T00:00:00Z"
updated = "2024-01-01T00:00:00Z"
+++

Optional body text here.
```

Files live in `.peas/` (active) or `.peas/archive/` (archived).

### Key Modules

- **`graphql/`**: async-graphql schema; `peas serve` starts an HTTP server; `peas query/mutate` execute inline. Uses the same storage layer as the CLI.
- **`tui/`**: ratatui terminal UI. `app.rs` is a state machine with `InputMode` and `ViewMode` enums — see `docs/tui-state-machine.md` for the full diagram and invariants.
- **`search.rs`**: Field-specific and regex search query parser.
- **`undo.rs`**: `UndoManager` — 50-level stack stored in `.peas/.undo`.
- **`assets.rs`**: File attachment management.
- **`validation.rs`**: All input validation (title max 200 chars, body max 50k, path traversal prevention).
- **`error.rs`**: `PeasError` enum; handlers return `anyhow::Result<()>`.

## Commit Messages

Include relevant pea IDs in commit messages (e.g., `peas-abc12`).

## Testing Patterns

Integration tests in `tests/cli_tests.rs` use `assert_cmd` to invoke the binary as a subprocess and `tempfile::TempDir` for isolation. Pattern: init project in temp dir → run command → assert stdout/stderr.
