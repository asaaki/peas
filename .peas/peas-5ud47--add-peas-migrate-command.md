+++
id = "peas-5ud47"
title = "Add peas migrate command"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-02T19:42:29.409971400Z"
updated = "2026-02-02T19:42:29.409971400Z"
+++

Added `peas migrate` command to help users migrate from legacy config locations to the new `.peas/config.toml` location.

## Usage
```bash
peas migrate          # Perform migration
peas migrate --dry-run  # Preview what would be changed
```

## Features
- Detects legacy config files (`.peas.toml`, `.peas.yml`, etc.)
- Moves config to `.peas/config.toml`
- Removes deprecated `path` option from config
- Adds schema directive for LSP support
- Dry-run mode to preview changes
- Handles already-migrated projects gracefully
- Cleans up leftover legacy config files
