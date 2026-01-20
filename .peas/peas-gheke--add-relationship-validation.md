+++
id = "peas-gheke"
title = "Add relationship validation"
type = "feature"
status = "completed"
priority = "high"
parent = "peas-w51zp"
created = "2026-01-20T15:10:44.782056400Z"
updated = "2026-01-20T15:30:17.517469400Z"
+++

## Problem
No validation for:
- Orphaned peas (parent doesn't exist)
- Circular blocking relationships
- Self-blocking peas
- Circular parent-child hierarchies

## Impact
- Invalid data states
- Confusing tree views
- Deadlock situations (A blocks B blocks A)

## Solution
Add validation in repository.rs create/update:
1. Check parent exists before setting
2. Detect cycles in blocking graph
3. Prevent self-references
4. Provide clear error messages

## Files
- src/storage/repository.rs
- src/model/validation.rs
