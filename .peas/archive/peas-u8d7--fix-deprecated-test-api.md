+++
id = "peas-u8d7"
title = "Fix deprecated test API"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-4988a"
created = "2026-01-18T15:55:19Z"
updated = "2026-01-18T19:09:54.534570500Z"
+++

Replace assert_cmd::Command::cargo_bin with cargo::cargo_bin_cmd! macro in tests/cli_tests.rs:6
