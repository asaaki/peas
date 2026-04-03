+++
id = "peas-sdn6s"
title = "Research asset support architecture for tickets"
type = "research"
status = "completed"
priority = "normal"
parent = "peas-1le6t"
created = "2026-01-19T16:11:56.593553400Z"
updated = "2026-01-19T19:40:00.000000000Z"
+++

Research and design asset support for tickets.

## Architecture
- Use ticket IDs as folder names for assets (e.g., `.peas/assets/peas-abc123/`)

## Phase 1: Basic Support
- List assets in TUI
- Provide local file paths in APIs for LLM consumption

## Media Support Guidelines
- Text files: Always supported
- Images: PNG, JPG, GIF should work (many LLMs support image consumption)
- WEBP/AVIF: Make educated guess on support (likely yes for modern LLMs)

## Phase 2: Modal Display
- Show assets in a modal view
- Text files: Always render
- Images: Render with appropriate terminal image support/dependencies

## Research Findings

### Storage Architecture
**Recommendation: `.peas/assets/{ticket-id}/` structure**
- Simple and predictable
- Easy to clean up when tickets are archived
- Works well with Git (can .gitignore binary assets selectively)
- Example: `.peas/assets/peas-abc12/screenshot.png`

### Image Display in Terminal
**Library: ratatui-image**
- Unified image rendering for Sixels, Kitty, iTerm2 protocols
- Auto-detects terminal capabilities
- Falls back to Unicode halfblocks for compatibility
- Uses `image` crate for loading PNG, JPG, GIF, WEBP, etc.

**Protocol Detection:**
1. Check env vars for terminal type
2. Query terminal with control sequences
3. Fallback to halfblock Unicode characters with colors

**Crate:** https://crates.io/crates/ratatui-image
**Repo:** https://github.com/benjajaja/ratatui-image

### Text File Display
- Can use existing markdown rendering (tui-markdown already integrated)
- For plain text: use ratatui Text widget with scrolling
- Syntax highlighting: Use `syntect` (already a dependency)

### Implementation Phases

#### Phase 1: Basic Asset Management (CLI)
Create tickets for:
1. Add `assets` subcommand to CLI
   - `peas assets list <ticket-id>` - list assets
   - `peas assets add <ticket-id> <file-path>` - copy file to assets folder
   - `peas assets remove <ticket-id> <file-name>` - remove asset
2. Create assets directory structure
3. Add asset paths to GraphQL API for LLM integration

#### Phase 2: TUI Asset Viewing
Create tickets for:
1. Add assets pane to detail view (4th pane)
2. Show asset list with file names and sizes
3. Add dependency: `ratatui-image = "1.0"`
4. Implement asset viewer modal:
   - Images: Use ratatui-image widget
   - Text files: Render with syntax highlighting
   - Other files: Show metadata only

#### Phase 3: Asset Upload/Attachment
Create tickets for:
1. TUI key binding to attach file (copy into assets folder)
2. Integration with system file picker (platform-specific)
3. Drag-and-drop support (if terminal supports it)

### File Size Considerations
- Warn if asset >10MB
- Consider adding size limits in config
- Assets should not be stored in markdown files (keep separate)

## Next Steps
Create Phase 1 implementation tickets under E7.6 epic.
