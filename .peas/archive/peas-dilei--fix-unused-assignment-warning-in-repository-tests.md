+++
id = "peas-dilei"
title = "Fix unused assignment warning in repository tests"
type = "task"
status = "completed"
priority = "normal"
parent = "peas-u5iym"
created = "2026-01-22T13:04:46.804825Z"
updated = "2026-04-03T11:48:02.152008238Z"
+++

Fix the unused assignment warning in src/storage/repository.rs:500 where pea2 is assigned but never read. Either use the variable or remove the assignment.
