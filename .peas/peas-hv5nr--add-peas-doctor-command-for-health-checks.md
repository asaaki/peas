+++
id = "peas-hv5nr"
title = "Add peas doctor command for health checks"
type = "feature"
status = "completed"
priority = "normal"
created = "2026-02-02T19:44:41.466851600Z"
updated = "2026-02-02T19:48:24.768703800Z"
+++

Add a `peas doctor` command that assesses the health of the peas setup and suggests fixes.

## Proposed checks
- [ ] Config location: suggest `peas migrate` if using legacy location
- [ ] Config validity: check for parse errors or invalid values
- [ ] Deprecated options: warn about `path` or other deprecated settings
- [ ] Schema directive: suggest adding if missing
- [ ] Data directory: check if `.peas/` exists and is readable/writable
- [ ] Orphaned files: detect tickets with invalid references (parent, blocking)
- [ ] Duplicate IDs: check for ID collisions
- [ ] Sequential ID counter: verify `.peas/.id` is in sync with existing tickets
- [ ] Archive integrity: check archived tickets

## Usage
```bash
peas doctor          # Run all health checks
peas doctor --fix    # Automatically fix issues where possible
```

## Output format
- Show ✓ for passing checks
- Show ! for warnings with suggestions
- Show ✗ for errors that need attention
- Provide actionable fix commands where applicable
