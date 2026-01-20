+++
id = "peas-1le6t"
title = "E7.6: Asset Support"
type = "epic"
status = "in-progress"
priority = "normal"
parent = "peas-3h9f1"
created = "2026-01-19T16:10:53.228835100Z"
updated = "2026-01-20T14:22:13.309171900Z"
+++

# Asset Support Epic

## Vision

Enable tickets to reference and manage external assets (files, images, diagrams, documents) that provide additional context or documentation. This allows tickets to be more comprehensive and self-contained.

## Use Cases

1. **Design Mockups**: Attach wireframes, UI designs, or screenshots to feature tickets
2. **Diagrams**: Include architecture diagrams, flowcharts, or sequence diagrams
3. **Documents**: Link to specifications, RFCs, or research documents
4. **Screenshots**: Attach bug reproduction screenshots or before/after comparisons
5. **Test Data**: Include sample data files or test fixtures
6. **Reference Materials**: Attach PDFs, images, or other reference documents

## Scope

### In Scope
- Store assets in `.peas/assets/` directory organized by ticket ID
- Link assets to tickets via asset references in ticket body
- Display asset list in TUI detail view
- Open assets with system default application from TUI
- CLI commands to manage assets (add, list, remove)
- Support common file types (images, PDFs, text files, etc.)

### Out of Scope (for now)
- Asset versioning/history
- Inline image preview in TUI (would require image rendering)
- Asset compression or optimization
- Asset search/indexing
- Shared assets across multiple tickets

## Architecture

```
.peas/
├── assets/
│   ├── peas-abc123/
│   │   ├── screenshot-bug.png
│   │   ├── architecture.svg
│   │   └── spec.pdf
│   └── peas-xyz789/
│       └── mockup.png
└── [existing structure]
```

Assets are organized by ticket ID in subdirectories for easy management.

## Breakdown

This epic should be broken down into these stories/tickets:

1. **Asset Storage Foundation** (Story)
   - Create `.peas/assets/` directory structure
   - Implement asset path resolution
   - Add asset metadata tracking

2. **CLI Asset Commands** (Story)
   - `peas asset add <ticket-id> <file-path>` - Add asset to ticket
   - `peas asset list <ticket-id>` - List ticket assets
   - `peas asset remove <ticket-id> <asset-name>` - Remove asset
   - `peas asset open <ticket-id> <asset-name>` - Open asset

3. **Ticket Asset References** (Story)
   - Extend Pea model with assets field
   - Store asset references in frontmatter
   - Update serialization/deserialization

4. **TUI Asset Integration** (Story)
   - Add "Assets" tab in detail view
   - Display asset list with file types/sizes
   - Keyboard shortcut to open selected asset
   - Visual indicator when ticket has assets

5. **Asset Utilities** (Chore)
   - File type detection
   - File size formatting
   - Cross-platform file opening
   - Asset validation

## Success Criteria

- ✅ Users can attach files to tickets via CLI
- ✅ Assets are stored in organized directory structure
- ✅ TUI shows asset list in detail view
- ✅ Users can open assets from TUI with default app
- ✅ CLI provides full asset management capabilities
- ✅ Asset references persist in ticket frontmatter

## Future Enhancements

- Drag-and-drop file attachment in TUI
- Image thumbnail preview
- Asset search across all tickets
- Asset cleanup for deleted tickets
- Support for remote/URL assets
- Asset templates (e.g., screenshot template, diagram template)
