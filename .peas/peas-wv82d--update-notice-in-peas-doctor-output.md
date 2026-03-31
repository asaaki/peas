+++
id = "peas-wv82d"
title = "Update notice in `peas doctor` output"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-qcb9j"
created = "2026-03-30T11:52:03.793079Z"
updated = "2026-03-31T09:37:34.685109Z"
+++

## Description

Add a new "Update Check" section to \`peas doctor\` using the existing \`DiagnosticResults\` pattern.

## Acceptance Criteria

- [ ] New check section "Update Check" added to \`handle_doctor\`
- [ ] Up to date: \`✓ peas is up to date (X.Y.Z)\`
- [ ] Update available: \`! Update available: X.Y.Z\` + \`→ https://github.com/asaaki/peas/releases/latest\`
- [ ] Check failed: \`! Could not check for updates (network error or GitHub unreachable)\`
- [ ] Opted out: \`! Update checks are disabled (set updates.enabled = true in {config_dir}/peas/config.toml to re-enable)\`
- [ ] Spawns update check thread and joins it (blocks until result)

## Notes

Depends on: peas-wxbki (GlobalPeasConfig), peas-3mvco (updater module)
