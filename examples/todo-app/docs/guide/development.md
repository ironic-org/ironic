# Development

## Commands

```bash
# Build
cargo build

# Run
cargo run

# Run with hot reload (requires hot-reload feature)
cargo run -- dev

# Tests (unit only — no database needed)
cargo test

# Integration tests (requires database)
DATABASE_URL=postgres://user:pass@localhost:5432/todo_test cargo test -- --ignored

# Lint
cargo clippy

# Format
cargo fmt
```

## Project conventions

- **Module per domain** — each feature gets its own module under `src/modules/`
- **Flat services** — business logic lives in services, not controllers
- **DTOs for boundaries** — request/response shapes are explicit DTOs
- **Instrumentation** — all service methods use `#[instrument(skip(self))]` for tracing

## Adding a new module

```bash
# 1. Create module structure
mkdir -p src/modules/items/{controller,services,dto,entities}

# 2. Create module file
# src/modules/items/mod.rs

# 3. Register in src/modules/mod.rs
pub mod items;

# 4. Register in src/app.rs
#[module(imports = [..., ItemsModule])]
```

## Testing strategy

| Test type | Location | Requires DB |
|---|---|---|
| DTO validation | `tests/todo_tests.rs` | No |
| Service logic | `tests/todo_tests.rs` | No |
| API integration | `tests/api_tests.rs` | Yes |

API integration tests use `TestApplication` from the Ironic testing module.
