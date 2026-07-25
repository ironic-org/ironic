---
title: Project Structure Overview
description: Choose between single-service and monorepo workspace layouts
---

# Project Structure Overview

Ironic supports two project layouts. Choose based on your team size and service count:

| Aspect | Single Service | Monorepo Workspace |
|--------|---------------|-------------------|
| Team size | 1–5 developers | 5+ developers |
| Services | 1 | 2–20 |
| Code sharing | Copy/paste | Shared libraries |
| Dependencies | Single `Cargo.toml` | Workspace-level `Cargo.toml` |
| Build | `cargo build` | `cargo build --workspace` |
| Test | `cargo test` | `cargo test --workspace` |
| CI/CD | Simple pipeline | Matrix build per service |

## Quick Start

```bash
# Single service
ironic new my-api
cd my-api
cargo run

# Monorepo workspace
ironic new my-platform
cd my-platform
ironic generate app api-gateway
ironic generate app auth-service
ironic generate library shared-config
```

## Decision Guide

**Choose single service when:**
- You're building a monolithic API
- Your team is small
- You want the simplest possible setup

**Choose monorepo when:**
- You're building multiple microservices
- You need shared types/proto definitions
- You want unified dependency management
- You need atomic cross-service changes

## Scaffold Commands Reference

| Command | Creates | Location |
|---------|---------|----------|
| `ironic new <name>` | Full project | `./<name>/` |
| `ironic generate app <name>` | Binary crate | `apps/<name>/` |
| `ironic generate library <name>` | Library crate | `libs/<name>/` or `./<name>/` |
| `ironic generate module <name>` | Feature module | `src/modules/<name>/` |
| `ironic generate controller <name>` | Controller | `src/modules/<name>/controller/` |
| `ironic generate service <name>` | Service | `src/modules/<name>/services/` |
| `ironic generate repository <name>` | Repository | `src/modules/<name>/repositories/` |
| `ironic generate resource <name>` | Full CRUD scaffold | `src/modules/<name>/` |
