---
# peas-6qxq
title: Implement or remove unused CLI flags
status: completed
type: task
priority: normal
created_at: 2026-01-18T15:55:18Z
updated_at: 2026-01-18T15:59:40Z
parent: peas-4988
---

--config and --peas-path global args are defined in cli/commands.rs but never used in load_config(). Either implement them or remove.