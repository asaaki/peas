# Update Checker Design

**Date:** 2026-03-30
**Status:** Approved

## Overview

peas gains an automatic update checker that queries the GitHub Releases API once per day, caches the result locally, and surfaces update notices in the TUI footer, `peas doctor`, `peas --version`, and `--help` output. All network I/O happens in a background thread so no command is blocked at startup.

---

## Architecture

### New modules

**`src/global_config.rs`** — `GlobalPeasConfig`, loaded from `{config_dir}/peas/config.toml` (via `directories` crate). Handles user-level, cross-project settings. Independent of `PeasConfig` (project-level). For now contains only one section: `[updates]`.

**`src/updater.rs`** — self-contained update check module. Owns cache read/write, HTTP fetch, version comparison, and background thread spawning. Uses `directories` crate directly for cache path resolution. No dependency on `PeasConfig` or storage modules.

### Path resolution (via `directories` crate)

| Platform | Config | Cache |
|----------|--------|-------|
| Linux | `~/.config/peas/config.toml` | `~/.cache/peas/update-check.json` |
| macOS | `~/Library/Application Support/peas/config.toml` | `~/Library/Caches/peas/update-check.json` |
| Windows | `%APPDATA%\peas\config.toml` | `%LOCALAPPDATA%\peas\update-check.json` |

---

## GlobalPeasConfig

**File:** `{config_dir}/peas/config.toml`

```toml
[updates]
enabled = true  # set to false to opt out of update checks
```

**Struct:**

```rust
pub struct GlobalPeasConfig {
    pub updates: UpdatesConfig,
}

pub struct UpdatesConfig {
    pub enabled: bool,  // default: true
}
```

**Load behavior:**
- File absent → `GlobalPeasConfig::default()` (updates enabled), silent.
- File malformed → warn to stderr, fall back to default. Never fatal.
- No `save()` method — file is user-managed only.

---

## Updater Module

### Public API

```rust
pub fn spawn_update_check(global_config: &GlobalPeasConfig) -> JoinHandle<UpdateCheckOutcome>

pub enum UpdateCheckOutcome {
    UpdateAvailable(String),  // newer version tag, e.g. "0.2.2"
    UpToDate,
    CheckFailed,
    Skipped,                  // updates.enabled == false (opted out), or cache still valid and no new version
}
```

### Cache format

**File:** `{cache_dir}/peas/update-check.json`

```json
{
  "last_checked": "2026-03-30T12:00:00Z",
  "check_succeeded": true,
  "latest_version": "0.2.1",
  "retry_interval_hours": 24
}
```

### Cache / retry logic

- Cache valid (age < `retry_interval_hours`) → return `Skipped` immediately, no network call.
- Cache stale → spawn HTTP fetch.
- On **success**: update `latest_version`, reset `retry_interval_hours` to 24, save cache.
- On **failure**: step down `retry_interval_hours` through `24 → 12 → 6 → 3 → 1`, then hold at 1. Save cache with `check_succeeded: false`.

### HTTP fetch

- **Endpoint:** `https://api.github.com/repos/asaaki/peas/releases/latest`
- **Client:** `reqwest` blocking, 5s timeout, `User-Agent: peas/{version}` header.
- **Deserialization:** minimal — only `tag_name` field extracted.
- **Version comparison:** strip leading `v`, split on `.`, compare each component as `u32`. Uses `env!("CARGO_PKG_VERSION")` as current version. No `semver` crate needed.

---

## Update Notifications

### `peas --version`

Custom version output (override clap default), printed after joining the background thread:

```
peas 0.2.1
A new version is available: 0.2.2 — https://github.com/asaaki/peas/releases/latest
```

On failure:
```
peas 0.2.1
No update information available (check failed)
```

On up-to-date / skipped: just the version line, no second line.

### `peas doctor`

New "Update Check" section using the existing `DiagnosticResults` pattern:

- `✓ peas is up to date (0.2.1)`
- `! Update available: 0.2.2` + `→ https://github.com/asaaki/peas/releases/latest`
- `! Could not check for updates (network error or GitHub unreachable)`
- `! Update checks are disabled (set updates.enabled = true in {config_dir}/peas/config.toml to re-enable)`

### `peas help` / subcommand `--help`

Print the update notice as a trailing line **after** clap's help output. No modification to clap rendering.

```
... (clap help output) ...

A new version is available: 0.2.2 — https://github.com/asaaki/peas/releases/latest
```

No trailing line shown when up-to-date or check skipped.

### TUI footer

When `UpdateCheckOutcome::UpdateAvailable(v)` is resolved, append to footer spans:

```
  ● update available: v0.2.2
```

Shown in amber/yellow. Silently absent on `UpToDate`, `CheckFailed`, or `Skipped` — no error state shown in TUI.

---

## TUI Integration

The `App` struct gains:
```rust
pub update_check_handle: Option<JoinHandle<UpdateCheckOutcome>>,
pub available_update: Option<String>,  // set once handle resolves
```

Each render tick checks `handle.is_finished()`. Once finished, joins the handle, stores result in `available_update`, and sets handle to `None`. The footer rendering checks `available_update` to append the update notice.

---

## Dependencies to Add

- `reqwest` with `blocking` and `json` features (HTTP client)

`serde_json` is already in `Cargo.toml`. `directories`, `chrono`, `serde`, `tokio` are all already present.

---

## Out of Scope

- Global config settings beyond `updates.enabled` (future work)
- Automatic download or installation of updates
- Changelog display
- Notification on every command (only the listed surfaces)
