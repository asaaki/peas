---
# peas-u8d7
title: Fix deprecated test API
status: completed
type: task
priority: normal
created_at: 2026-01-18T15:55:19Z
updated_at: 2026-01-18T16:00:52Z
parent: peas-4988
---

Replace assert_cmd::Command::cargo_bin with cargo::cargo_bin_cmd! macro in tests/cli_tests.rs:6