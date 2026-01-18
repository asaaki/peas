---
# peas-h9gw
title: Fix clippy warnings
status: completed
type: task
priority: normal
created_at: 2026-01-18T14:51:03Z
updated_at: 2026-01-18T14:52:09Z
parent: peas-ep55
---

Address clippy warnings:

1. derivable_impls (src/config.rs:62): Replace manual Default impl for PeasConfig with #[derive(Default)]

2. ptr_arg (src/tui/app.rs:86): Change &PathBuf to &Path in App::new() signature

3. deprecated (tests/cli_tests.rs:6): Update assert_cmd::Command::cargo_bin usage