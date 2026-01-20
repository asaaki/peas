+++
id = "peas-n6yjy"
title = "Implement structured logging system"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-w51zp"
created = "2026-01-20T15:11:35.911923600Z"
updated = "2026-01-20T15:12:38.321859200Z"
+++

## Problem
All errors go to eprintln! making it hard to:
- Collect logs for debugging
- Audit operations
- Monitor performance
- Filter by severity

## Impact
- Hard to diagnose production issues
- No audit trail
- No performance monitoring

## Solution
Replace eprintln! with structured logging:
1. Use tracing crate with spans
2. Configure log levels (DEBUG, INFO, WARN, ERROR)
3. Support file output and rotation
4. Add --verbose/-v flag for CLI

## Files
- src/main.rs (tracing setup)
- All error handlers (replace eprintln!)
