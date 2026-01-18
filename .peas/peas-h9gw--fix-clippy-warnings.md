+++
id = "peas-h9gw"
title = "Fix clippy warnings"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-ep55"
created = "2026-01-18T14:51:03Z"
updated = "2026-01-18T14:52:09Z"
+++

Address clippy warnings:

1. derivable_impls (src/config.rs:62): Replace manual Default impl for PeasConfig with #[derive(Default)]

2. ptr_arg (src/tui/app.rs:86): Change &PathBuf to &Path in App::new() signature

3. deprecated (tests/cli_tests.rs:6): Update assert_cmd::Command::cargo_bin usage
