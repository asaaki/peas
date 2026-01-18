+++
id = "peas-5wco"
title = "Replace unwrap() calls with proper error handling"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-4988"
created = "2026-01-18T15:55:18Z"
updated = "2026-01-18T19:09:55.933917500Z"
+++

19 instances of unwrap() that could panic:
- src/config.rs:64 - config_path.parent().unwrap()
- src/main.rs:68,176,189 - path.file_name().unwrap()
- src/graphql/schema.rs:33 - ctx.data unwrap
- src/storage/repository.rs:87,119,135 - file_name().unwrap()

Replace with proper error handling using ? operator or .ok_or() chains.
