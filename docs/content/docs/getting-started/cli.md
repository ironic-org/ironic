---
title: CLI Reference
description: Master the Ironic command-line tools — create, generate, run, test, and inspect your application.
---

# CLI Reference

## What you'll learn

- Every CLI command and what it does
- Generator commands for scaffolding code
- Project inspection tools
- Doctor command for debugging

---

## Project commands

| Command | What it does |
|---------|-------------|
| `ironic new <name>` | Create a new project |
| `ironic new .` | Create project in the current directory |
| `ironic start` | Run the server (`cargo run`) |
| `ironic dev` | Run with hot reload — auto-restarts on file changes |
| `ironic build` | Build the project (`cargo build`) |
| `ironic test` | Run tests (`cargo test`) |

## Generator commands

| Command | Alias | Creates |
|---------|-------|---------|
| `ironic generate resource <name>` | `g res` | Full module with controller, service, DTOs, entity, and tests |
| `ironic generate ready-resource auth` | `g rr auth` | Production-ready auth module (JWT, OAuth, RBAC) |
| `ironic generate ready-resource file-upload` | `g rr file-upload` | File upload module (local, S3, R2 backends) |
| `ironic generate ready-resource email` | `g rr email` | Email module (SMTP, SES, SendGrid, Mailgun) |
| `ironic generate module <name>` | `g mo` | Module shell only |
| `ironic generate controller <name>` | `g co` | Controller inside a module |
| `ironic generate service <name>` | `g s` | Service inside a module |
| `ironic generate decorator <name>` | `g d` | Custom parameter decorator |
| `ironic generate filter <name>` | `g f` | Exception filter |
| `ironic generate guard <name>` | `g gu` | Auth guard |
| `ironic generate middleware <name>` | `g mi` | Middleware |
| `ironic generate pipe <name>` | `g pi` | Parameter pipe |
| `ironic generate provider <name>` | `g pr` | Injectable provider |

## Inspection commands

| Command | What it does |
|---------|-------------|
| `ironic routes` | List all routes in the project |
| `ironic graph` | Print a Graphviz dependency graph |

## Doctor command

```bash
ironic doctor
```

Checks your environment:

```
Rust                   OK rustc 0.4.4
Cargo                  OK cargo 0.4.4
Project manifest       OK /path/to/Cargo.toml
Ironic dependency      OK found
CLI version            OK 0.2.0 (latest)
```

## Update command

```bash
ironic update
# or: ironic upgrade
```

Checks crates.io for a newer version and shows update instructions.

## What you learned

- [x] `ironic new` creates projects
- [x] `ironic start/build/test` wraps Cargo commands
- [x] `ironic generate resource` creates full vertical slices
- [x] `ironic doctor` diagnoses environment issues
- [x] `ironic routes` and `ironic graph` inspect projects
