+++
id = "peas-85bwj"
title = "Optimize URL regex compilation"
type = "chore"
status = "completed"
priority = "low"
parent = "peas-w51zp"
created = "2026-01-20T15:11:56.943481900Z"
updated = "2026-01-20T21:25:24.932688700Z"
+++

## Problem
URL regex compiled on every extraction in url_utils.rs:
```rust
let url_pattern = regex::Regex::new(r"https?://[^\s<>]+").unwrap();
```

Called repeatedly in render loop.

## Impact
- Unnecessary allocations
- Performance degradation in TUI
- ~100μs overhead per call

## Solution
Use lazy_static or once_cell to compile regex once:
```rust
static URL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s<>]+").unwrap()
});
```

## Files
- src/tui/url_utils.rs
