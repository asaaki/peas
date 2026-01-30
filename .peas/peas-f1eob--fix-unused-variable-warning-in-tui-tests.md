+++
id = "peas-f1eob"
title = "Fix unused variable warning in TUI tests"
type = "task"
status = "todo"
priority = "normal"
parent = "peas-u5iym"
created = "2026-01-22T13:04:44.561987400Z"
updated = "2026-01-22T13:04:44.561987400Z"
+++

Fix the unused variable warning in tests/tui_tests.rs:134 where temp_dir is assigned but never used. Prefix with underscore to indicate intentional non-use.
