# Agent Rules

**IMPORTANT**: before you do anything else, run the `peas prime` command (or `just prime`) and heed its output.

Note: The `peas` executable might be installed globally already. If not, you can run it via `cargo run -- <command>` from the project root. Also use the latter if you need to test a feature in development.

**IMPORTANT**: This project uses `just` for task automation. Always prefer `just` recipes over manual commands when available. Run `just` to see all available tasks.

## Common Tasks with Just

Use these `just` recipes for common development tasks:

- **Development workflow**: `just dev` - lints (includes formatting), and tests
- **Build**: `just build` or `just build-release`
- **Test**: `just test` or `just test-verbose`
- **Lint**: `just lint` (formats and runs clippy) or `just fix` (auto-fixes)
- **Full CI check**: `just ci` - runs all checks that CI would run
- **Version bump**: `just version-patch`, `just version-minor`, or `just version-major`
- **Release**: `just release "Release message"` - commits, tags, and pushes
- **Documentation**: `just doc` - generates and opens docs
- **Peas operations**: `just prime`, `just list`, `just tui`, etc.

Run `just --list` to see all available tasks.

## Commit Messages

When making a commit, include the relevant pea IDs in the commit message (e.g., peas-abc12).
