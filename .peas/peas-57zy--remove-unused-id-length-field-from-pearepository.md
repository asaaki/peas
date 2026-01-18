+++
id = "peas-57zy"
title = "Remove unused id_length field from PeaRepository"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-ep55"
created = "2026-01-18T14:50:57Z"
updated = "2026-01-18T14:55:08Z"
+++

Compiler warning: field id_length is never read in PeaRepository struct (src/storage/repository.rs:15).

Options:
1. Remove the field entirely if not needed
2. Implement variable-length ID generation using the field
3. Add #[allow(dead_code)] with comment explaining future use

Current behavior uses hardcoded 4-char IDs. Decide whether to support configurable lengths.
