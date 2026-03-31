+++
id = "peas-3mvco"
title = "Updater module: cache, HTTP fetch, version comparison"
type = "feature"
status = "completed"
priority = "normal"
parent = "peas-qcb9j"
created = "2026-03-30T11:51:43.585182Z"
updated = "2026-03-31T09:37:31.742984Z"
+++

## Description

Add \`src/updater.rs\` — self-contained module owning all update-check logic: cache read/write, HTTP fetch, version comparison, and background thread spawning.

## Acceptance Criteria

- [ ] Cache file at \`{cache_dir}/peas/update-check.json\` with fields: \`last_checked\`, \`check_succeeded\`, \`latest_version\`, \`retry_interval_hours\`
- [ ] Cache valid (age < \`retry_interval_hours\`) → return \`Skipped\` immediately, no network call
- [ ] On success: update \`latest_version\`, reset \`retry_interval_hours\` to 24, save cache
- [ ] On failure: step down \`retry_interval_hours\` through \`24 → 12 → 6 → 3 → 1\`, hold at 1, save cache
- [ ] HTTP fetch via \`reqwest\` blocking client, 5s timeout, \`User-Agent: peas/{version}\` header
- [ ] Endpoint: \`https://api.github.com/repos/asaaki/peas/releases/latest\`, extract \`tag_name\` only
- [ ] Version comparison: strip \`v\`, split on \`."\`, compare each component as \`u32\` — no \`semver\` crate
- [ ] Public API: \`spawn_update_check(global_config: &GlobalPeasConfig) -> JoinHandle<UpdateCheckOutcome>\`
- [ ] \`UpdateCheckOutcome\` variants: \`UpdateAvailable(String)\`, \`UpToDate\`, \`CheckFailed\`, \`Skipped\`
- [ ] Returns \`Skipped\` immediately if \`global_config.updates.enabled == false\`

## Notes

Add \`reqwest\` with \`blocking\` and \`json\` features to \`Cargo.toml\`. \`serde_json\`, \`chrono\`, \`directories\` are already present.
