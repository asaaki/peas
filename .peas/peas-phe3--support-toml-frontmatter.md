+++
id = "peas-phe3"
title = "Support TOML frontmatter"
type = "epic"
status = "completed"
priority = "normal"
created = "2026-01-18T16:08:25Z"
updated = "2026-01-18T16:08:25Z"
+++

Add support for TOML as an alternative frontmatter format alongside YAML.

Requirements:
- Auto-detect frontmatter format when reading (YAML uses ---, TOML uses +++)
- Add config option to set default format for new peas
- Preserve existing format when updating a pea
- Support both formats in the same project
