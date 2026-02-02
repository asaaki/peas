# peas

A CLI-based, flat-file issue tracker for humans and robots.

**peas** stores issues as markdown files alongside your code, making them easy to version control and read. It provides both a CLI and GraphQL interface, perfect for AI coding agents.

Inspired by [beans](https://github.com/hmans/beans) and [beads](https://github.com/steveyegge/beads).

## Features

- **Flat-file storage**: Issues stored as markdown with TOML frontmatter in `.peas/`
- **GraphQL interface**: Query and mutate peas with GraphQL for AI agent integration
- **Interactive TUI**: Browse and manage peas in a terminal UI with multi-select and undo
- **Hierarchical structure**: Milestones, epics, stories, features, bugs, chores, research, and tasks
- **Memory system**: Store and retrieve project knowledge, decisions, and context
- **Asset management**: Attach files, images, and documents to tickets
- **Relationships**: Link tickets with parent/child and blocking dependencies
- **Agent-friendly**: `peas prime` outputs instructions for AI coding agents
- **Undo support**: Multi-level undo for accidental changes

## Installation

### With cargo-binstall (recommended)

The fastest way to install pre-built binaries:

```bash
cargo binstall peas
```

### From GitHub releases

Download pre-built binaries directly from [GitHub releases](https://github.com/asaaki/peas/releases).

### From crates.io

Build from source via crates.io:

```bash
cargo install peas --locked
```

### From source

Build from the repository:

```bash
git clone https://github.com/asaaki/peas
cd peas
cargo install --path .
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
| `peas list` | List all peas (filter by type, status, priority, tags) |
| `peas show <id>` | Show pea details |
| `peas update <id>` | Update a pea's properties |
| `peas start <id>` | Mark pea as in-progress |
| `peas done <id>` | Mark pea as completed |
| `peas archive <id>` | Archive a pea (move to archive folder) |
| `peas delete <id>` | Delete a pea permanently |
| `peas search <query>` | Search peas by text |
| `peas suggest` | Suggest the next ticket to work on |
| `peas roadmap` | Generate markdown roadmap from milestones and epics |
| `peas prime` | Output agent instructions |
| `peas context` | Output project context for LLMs |
| `peas query <query>` | Execute a GraphQL query |
| `peas mutate <mutation>` | Execute a GraphQL mutation |
| `peas serve` | Start GraphQL HTTP server |
| `peas tui` | Open interactive TUI |
| `peas import-beans` | Import from a beans project |
| `peas export-beans` | Export to beans format |
| `peas bulk <action>` | Bulk update multiple peas at once |
| `peas memory <action>` | Manage project memory and knowledge |
| `peas asset <action>` | Manage ticket assets (files, images, documents) |
| `peas undo` | Undo the last operation |

## Pea Types

- `milestone` - High-level project goals
- `epic` - Large features or initiatives
- `story` - User stories or scenarios
- `feature` - New functionality
- `bug` - Issues to fix
- `chore` - Maintenance tasks (refactoring, cleanup, etc.)
- `research` - Research tasks or spikes
- `task` - General work items (default)

## Pea Statuses

- `draft` - Not ready to work on
- `todo` - Ready to be worked on (default)
- `in-progress` - Currently being worked on
- `completed` - Done
- `scrapped` - Cancelled

## Pea Priorities

- `critical` - Must be done immediately (also: `p0`)
- `high` - Important, should be done soon (also: `p1`)
- `normal` - Standard priority (default, also: `p2`)
- `low` - Nice to have (also: `p3`)
- `deferred` - Postponed indefinitely (also: `p4`)

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
| `↑`/`↓` | Navigate up/down |
| `←`/`→` | Previous/next page |
| `Tab` | Switch between Tickets/Memory views |
| `/` | Search |
| `Enter` | Open detail view |
| `Space` | Multi-select toggle |
| `c` | Create new ticket |
| `s` | Change status |
| `t` | Change type |
| `P` | Change priority |
| `e` | Edit in $EDITOR |
| `r` | Refresh |
| `u` | Undo last operation |
| `?` | Help |
| `q` | Quit |

## Configuration

peas uses `.peas.toml` for configuration (also supports `.peas.yml`, `.peas.yaml`, or `.peas.json`, but TOML is preferred):

```toml
[peas]
path = ".peas"          # Data directory
prefix = "peas-"        # ID prefix
id_length = 5           # ID suffix length
id_mode = "random"      # ID mode: "random" (default) or "sequential"
default_status = "todo"
default_type = "task"
frontmatter = "toml"    # Frontmatter format: toml, yaml, json (TOML preferred)

[tui]
use_type_emojis = false # Enable emoji icons for ticket types in TUI
```

### ID Modes

- **random** (default): Generates IDs like `peas-a1b2c` using random alphanumeric characters
- **sequential**: Generates IDs like `peas-00001`, `peas-00002`, etc. using an incrementing counter stored in `.peas/.id`

### Editor Support (JSON Schema)

A JSON Schema is available at `schemas/peas.json` for editor autocompletion and validation.

**In-file directive (works with Taplo and Tombi):**

Add this comment at the top of your `.peas.toml`:
```toml
#:schema ./schemas/peas.json

[peas]
prefix = "peas-"
```

**Zed with Tombi extension:**

Add to your `tombi.toml` (or project settings):
```toml
[[schemas]]
path = "./schemas/peas.json"
include = [".peas.toml"]
```

**VS Code with Even Better TOML (Taplo):**

Add to your `.vscode/settings.json`:
```json
{
  "evenBetterToml.schema.associations": {
    ".peas.toml": "./schemas/peas.json"
  }
}
```

**VS Code with YAML extension:**
```json
{
  "yaml.schemas": {
    "./schemas/peas.json": [".peas.yml", ".peas.yaml"]
  }
}
```

**Neovim with taplo/yaml-language-server:**

Configure your LSP to associate the schema with `.peas.*` files.

## File Format

Peas are stored as markdown files with TOML frontmatter (YAML and JSON also supported):

```markdown
+++
id = "peas-abc1"
title = "Implement feature X"
type = "feature"
status = "in-progress"
priority = "high"
tags = ["backend", "api"]
parent = "peas-xyz9"
created = "2024-01-15T10:30:00Z"
updated = "2024-01-15T14:22:00Z"
+++

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
