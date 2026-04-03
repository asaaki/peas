+++
id = "peas-8k8vw"
title = "Fix wl-clipboard-rs future incompatibility warning"
type = "bug"
status = "completed"
priority = "high"
created = "2026-04-03T12:25:24.317085515Z"
updated = "2026-04-03T12:50:15.908125584Z"
+++

\`wl-clipboard-rs v0.7.0\` will be **rejected by a future Rust compiler** due to a soundness fix. This comes transitively through \`cli-clipboard\`.

## Current state
- \`cargo clippy\` and \`cargo build --release\` both emit:
  \`warning: the following packages contain code that will be rejected by a future version of Rust: wl-clipboard-rs v0.7.0\`
- Newer versions available: 0.8.x, 0.9.x

## Action
- Check if \`cli-clipboard\` has a newer release that pulls in \`wl-clipboard-rs >= 0.8\`
- If not, evaluate alternatives (\`arboard\`, \`copypasta\`, or direct \`wl-clipboard-rs\` usage)
- Update dependency and verify build on Linux/Wayland
