+++
id = "peas-wxbki"
title = "GlobalPeasConfig: global config file infrastructure"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-qcb9j"
created = "2026-03-30T11:51:21.768502Z"
updated = "2026-03-31T09:37:30.215586Z"
+++

## Description

Add \`src/global_config.rs\` with \`GlobalPeasConfig\` loaded from \`{config_dir}/peas/config.toml\` via the \`directories\` crate.

## Acceptance Criteria

- [ ] \`GlobalPeasConfig\` struct with \`[updates]\` section (\`enabled: bool\`, default \`true\`)
- [ ] Loaded from OS-appropriate config dir (Linux: \`~/.config/peas/\`, macOS: \`~/Library/Application Support/peas/\`, Windows: \`%APPDATA%\peas\\\`)
- [ ] File absent → silently use defaults
- [ ] File malformed → warn to stderr, use defaults, never fatal
- [ ] No \`save()\` method — file is user-managed only

## Notes

Path resolution uses the \`directories\` crate (already in \`Cargo.toml\`). Independent of \`PeasConfig\` (project-level config).
