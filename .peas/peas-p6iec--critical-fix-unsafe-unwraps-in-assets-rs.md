+++
id = "peas-p6iec"
title = "Critical: Fix unsafe unwraps in assets.rs"
type = "bug"
status = "completed"
priority = "critical"
parent = "peas-w51zp"
assets = ["test-unwrap-fix.txt"]
created = "2026-01-20T15:10:30.178611100Z"
updated = "2026-01-20T20:49:52.551366300Z"
+++

## Problem
Multiple unsafe .unwrap() calls in assets.rs can panic if file paths are invalid:
- Line 60: dest_path.file_name().unwrap().to_str().unwrap()
- Line 78: path.file_name().unwrap().to_str().unwrap()
- Line 131: path.extension().unwrap_or()

## Impact
- Data loss on asset operations
- Poor user experience
- Silent panics

## Solution
Replace with proper error handling using .context() with anyhow

## Files
- src/assets.rs
