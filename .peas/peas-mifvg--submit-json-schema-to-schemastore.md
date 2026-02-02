+++
id = "peas-mifvg"
title = "Submit JSON Schema to SchemaStore"
type = "chore"
status = "todo"
priority = "normal"
created = "2026-02-02T19:19:10.880253300Z"
updated = "2026-02-02T19:19:10.880253300Z"
+++

Submit the peas configuration schema to SchemaStore for automatic editor discovery.

## Requirements
- Fork SchemaStore/schemastore repository
- Add schema to `src/schemas/json/peas.json`
- Add catalog entry with fileMatch patterns:
  - `.peas.toml`
  - `.peas.yml`
  - `.peas.yaml`
  - `.peas.json`
- Add test files in `src/test/peas/` and `src/negative_test/peas/`
- Submit PR

## References
- https://github.com/SchemaStore/schemastore/blob/master/CONTRIBUTING.md
- https://www.schemastore.org/
