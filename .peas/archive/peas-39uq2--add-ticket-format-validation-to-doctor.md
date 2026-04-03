+++
id = "peas-39uq2"
title = "Add ticket format validation to doctor"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-02T23:46:57.775134400Z"
updated = "2026-02-02T23:48:02.331533600Z"
+++

Add validation checks for ticket files in peas doctor:

- Frontmatter parsing errors (already exists but could be more detailed)
- Malformed array fields (e.g., blocking field with comma-separated string instead of array)
- Invalid enum values (status, type, priority)
- ID format validation (matches prefix + expected length)
- Required fields present (id, title, type, status)
- Timestamp format validation
- Parent/blocking references format (should be valid ticket IDs)

This helps catch manual editing mistakes and data corruption.
