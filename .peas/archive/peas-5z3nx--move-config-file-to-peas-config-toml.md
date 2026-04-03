+++
id = "peas-5z3nx"
title = "Move config file to .peas/config.toml"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-02T19:38:56.817948700Z"
updated = "2026-02-02T19:38:56.817948700Z"
+++

Moved the canonical config file location from project root to inside the .peas directory.

## Changes
- New config location: `.peas/config.toml` (also supports .yml, .yaml, .json)
- Legacy locations (`.peas.toml`, `.peas.yml`, etc.) still supported with deprecation warning
- Deprecated `peas.path` config option - data is always stored in `.peas/`
- Updated `peas init` to create config in new location
- Added deprecation warnings for legacy config and `--peas-path` CLI option

## Migration
Users with existing projects will see a deprecation warning. To migrate:
1. Move `.peas.toml` to `.peas/config.toml`
2. Remove the `path` option from config (it's now ignored)

## Benefits
- Cleaner project root (only `.peas/` directory)
- All peas-related files in one place
- Config file is now inside the data directory it configures
