+++
id = "peas-00pnq"
title = "reqwest default features pull in unnecessary TLS backends"
type = "chore"
status = "completed"
priority = "low"
created = "2026-03-31T16:25:27.798242Z"
updated = "2026-04-03T12:01:47.666758598Z"
+++

## Description

`Cargo.toml` declares:

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

reqwest 0.12 default features include both `native-tls` and `hyper`, pulling in `hyper-tls`, `hyper-rustls`, and related crates unnecessarily. For a CLI tool that only uses blocking HTTP to a single endpoint this inflates binary size and compile time.

## Fix

```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```
