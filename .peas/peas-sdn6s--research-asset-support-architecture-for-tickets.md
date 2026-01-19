+++
id = "peas-sdn6s"
title = "Research asset support architecture for tickets"
type = "research"
status = "todo"
priority = "normal"
parent = "peas-1le6t"
created = "2026-01-19T16:11:56.593553400Z"
updated = "2026-01-19T16:12:05.860500900Z"
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

## Outcome
Create implementation work items based on research findings.
