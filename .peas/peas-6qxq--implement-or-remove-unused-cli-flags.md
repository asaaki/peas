+++
id = "peas-6qxq"
title = "Implement or remove unused CLI flags"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-4988"
created = "2026-01-18T15:55:18Z"
updated = "2026-01-18T15:59:40Z"
+++

--config and --peas-path global args are defined in cli/commands.rs but never used in load_config(). Either implement them or remove.
