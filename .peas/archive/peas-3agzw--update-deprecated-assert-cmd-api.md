+++
id = "peas-3agzw"
title = "Update deprecated assert_cmd API"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-u5iym"
created = "2026-01-22T13:04:48.788783300Z"
updated = "2026-04-03T11:48:02.542883549Z"
+++

Replace assert_cmd::Command::cargo_bin with the newer cargo::cargo_bin_cmd! macro in all test files. This is a deprecated API that should be updated for future compatibility.
