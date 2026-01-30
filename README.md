# peas

A CLI-based, flat-file issue tracker for humans and robots.

**peas** stores issues as markdown files alongside your code, making them easy to version control and read. It provides both a CLI and GraphQL interface, perfect for AI coding agents.

Inspired by [beans](https://github.com/hmans/beans) and [beads](https://github.com/steveyegge/beads).

## Features

- **Flat-file storage**: Issues stored as markdown with YAML frontmatter in `.peas/`
- **GraphQL interface**: Query and mutate peas with GraphQL for AI agent integration
- **Interactive TUI**: Browse and manage peas in a terminal UI
- **Hierarchical structure**: Milestones, epics, features, bugs, and tasks
- **Agent-friendly**: `peas prime` outputs instructions for AI coding agents

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install peas
```

## Quick Start

```bash
# Initialize a peas project
peas init

# Create some peas
peas create "Set up authentication" -t feature
peas create "Fix login bug" -t bug -p high
peas create "Q1 Release" -t milestone

# List peas
peas list
peas list -t bug
peas list -s in-progress

# Update status
peas start <id>    # Mark as in-progress
peas done <id>     # Mark as completed

# Search
peas search "auth"

# Interactive TUI
peas tui
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `peas init` | Initialize a new peas project |
| `peas create <title>` | Create a new pea |
| `peas list` | List all peas |
| `peas show <id>` | Show pea details |
| `peas update <id>` | Update a pea |
| `peas start <id>` | Mark pea as in-progress |
| `peas done <id>` | Mark pea as completed |
| `peas archive <id>` | Archive a pea |
| `peas delete <id>` | Delete a pea |
| `peas search <query>` | Search peas |
| `peas roadmap` | Generate markdown roadmap |
| `peas prime` | Output agent instructions |
| `peas context` | Output project context as JSON |
| `peas graphql <query>` | Execute GraphQL query |
| `peas serve` | Start GraphQL HTTP server |
| `peas tui` | Open interactive TUI |

## Pea Types

- `milestone` - High-level project goals
- `epic` - Large features or initiatives
- `feature` - New functionality
- `bug` - Issues to fix
- `task` - General work items

## Pea Statuses

- `draft` - Not ready to work on
- `todo` - Ready to be worked on
- `in-progress` - Currently being worked on
- `completed` - Done
- `scrapped` - Cancelled

## GraphQL Interface

peas provides a full GraphQL API for programmatic access:

```bash
# Query stats
peas graphql '{ stats { total byStatus { todo inProgress completed } } }'

# List open peas
peas graphql '{ peas(filter: { isOpen: true }) { nodes { id title status } } }'

# Create a pea
peas graphql 'mutation { createPea(input: { title: "New Task", peaType: TASK }) { id } }'

# Update status
peas graphql 'mutation { setStatus(id: "peas-abc1", status: IN_PROGRESS) { id status } }'
```

Start the GraphQL playground:

```bash
peas serve --port 4000
# Open http://localhost:4000
```

## Agent Integration

### Claude Code

Add to your `.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      { "hooks": [{ "type": "command", "command": "peas prime" }] }
    ],
    "PreCompact": [
      { "hooks": [{ "type": "command", "command": "peas prime" }] }
    ]
  }
}
```

Or add to your `AGENTS.md`:

```markdown
**IMPORTANT**: Run `peas prime` before starting work to see project tasks.
```

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j`/`k` or arrows | Navigate up/down |
| `Tab` | Next filter |
| `Shift+Tab` | Previous filter |
| `/` | Search |
| `Space` | Toggle status |
| `s` | Start (set in-progress) |
| `d` | Done (set completed) |
| `r` | Refresh |
| `?` | Help |
| `q` | Quit |

## Configuration

peas uses `.peas.toml` for configuration (also supports `.peas.yml`, `.peas.yaml`, or `.peas.json`):

```toml
[peas]
path = ".peas"          # Data directory
prefix = "peas-"        # ID prefix
id_length = 5           # Random ID length
default_status = "todo"
default_type = "task"
frontmatter = "toml"    # Frontmatter format: toml, yaml

[tui]
use_type_emojis = false # Enable emoji icons for ticket types in TUI
```

## File Format

Peas are stored as markdown files with YAML frontmatter:

```markdown
---
id: peas-abc1
title: Implement feature X
type: feature
status: in-progress
priority: high
tags:
  - backend
  - api
parent: peas-xyz9
created: 2024-01-15T10:30:00Z
updated: 2024-01-15T14:22:00Z
---

Detailed description of the feature goes here.

## Acceptance Criteria
- [ ] API endpoint created
- [ ] Tests written
- [ ] Documentation updated
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
