+++
id = "peas-4988"
title = "Code hardening and cleanup"
type = "epic"
status = "completed"
priority = "normal"
parent = "peas-ep55"
created = "2026-01-18T15:54:14Z"
updated = "2026-01-18T16:03:53Z"
+++

Assessment findings to address:

## High Priority
1. **Replace unwrap() calls with proper error handling** (19 instances)
   - src/config.rs:64 - config_path.parent().unwrap()
   - src/main.rs:68,176,189 - path.file_name().unwrap()
   - src/graphql/schema.rs:33 - ctx.data unwrap
   - src/storage/repository.rs:87,119,135 - file_name().unwrap()

2. **Implement or remove unused CLI flags**
   - --config and --peas-path global args are defined but never used

3. **Refactor large main.rs match block** (317 lines)
   - Extract command handlers into separate functions

## Medium Priority
4. **Fix deprecated test API**
   - Replace assert_cmd::Command::cargo_bin with cargo::cargo_bin_cmd! macro

5. **Consolidate duplicate filtering logic**
   - Filtering exists in main.rs, graphql/schema.rs, and tui/app.rs

6. **Add input validation**
   - Title/body length limits
   - Path traversal protection for IDs

## Documentation
7. **Add module-level documentation**
   - src/model/, src/storage/, src/graphql/, src/tui/
