+++
id = "peas-ljtmi"
title = "Implement rollback for bulk operations"
type = "bug"
status = "todo"
priority = "high"
parent = "peas-w51zp"
created = "2026-01-20T15:10:51.329595700Z"
updated = "2026-01-20T15:12:36.130819800Z"
+++

## Problem
Bulk operations can partially fail without rollback. If item 5/10 fails, items 1-4 are committed but 6-10 aren't attempted.

## Impact
- Inconsistent state after failures
- No way to recover
- Confusing error messages

## Solution
Implement transaction-like behavior:
1. Validate all items before any modifications
2. Apply changes in memory first
3. Write all to disk only if all succeed
4. Or provide clear 'continue on error' flag with summary

## Files
- src/cli/handlers/bulk.rs
