+++
id = "peas-lmkga"
title = "JSON Schema for configuration files"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-02T19:19:03.285041300Z"
updated = "2026-02-02T19:20:44.621276900Z"
+++

Added JSON Schema for peas configuration files to enable LSP support in editors.

## Changes
- Created `schemas/peas.json` with JSON Schema draft-07
- Schema covers all config options: peas settings and tui settings
- Includes `x-taplo` extensions for enhanced Taplo support
- Includes `x-tombi-table-keys-order` for Tombi key ordering
- Added editor configuration examples to README

## Editor Support
- Zed with Tombi extension
- VS Code with Even Better TOML (Taplo)
- VS Code with YAML extension
- Neovim with taplo/yaml-language-server
- Any editor supporting JSON Schema

## In-file directive
Add `#:schema ./schemas/peas.json` at the top of .peas.toml (works with both Taplo and Tombi)

## Next Steps
- Submit to SchemaStore for automatic editor discovery
