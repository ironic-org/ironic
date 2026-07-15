# Architecture

## Directory structure

```
src/
├── main.rs                 # Entry point — wiring, init, server
├── app.rs                  # Root AppModule
├── welcome.rs              # WelcomeController + WelcomeModule
├── platform/
│   ├── mod.rs
│   ├── config.rs           # Environment variable helpers
│   ├── database.rs         # Database pool (OnceLock, migrations)
│   └── telemetry.rs        # Tracing/logging initialization
└── modules/
    └── todos/
        ├── mod.rs          # TodosModule definition
        ├── controller/
        │   ├── mod.rs
        │   └── todos_controller.rs   # HTTP handlers
        ├── repositories/
        │   ├── mod.rs
        │   └── todo_repository.rs   # Data access (SQL)
        ├── services/
        │   ├── mod.rs
        │   └── todo_service.rs       # Business logic
        ├── dto/
        │   ├── mod.rs
        │   ├── create_todo_dto.rs    # Create validation
        │   └── update_todo_dto.rs    # Update schema
        ├── entities/
        │   ├── mod.rs
        │   └── todo.rs              # Database model
        └── tests/
            ├── mod.rs
            ├── todo_tests.rs        # Unit tests (DTO validation)
            └── api_tests.rs         # Integration tests (need DB)
```

## Request lifecycle

```
HTTP Request
    │
    ▼
SecurityHeadersMiddleware
    │
    ▼
RateLimitMiddleware
    │
    ▼
CorsMiddleware
    │
    ▼
Axum Router
    │
    ├── MetricsLayer (Prometheus)
    │
    ▼
TodosController
    │
    ▼
TodoService (business logic)
    │
    ▼
TodoRepository (data access)
    │
    ▼
SQLx → PostgreSQL
    │
    ▼
JSON Response
```

## Module system

Modules declare providers (repositories + services) and controllers. The DI container auto-wires dependencies:

- `TodosModule` registers `TodoRepository`, `TodoService`, and `TodosController`
- `Arc<TodoRepository>` is injected into `TodoService`
- `Arc<TodoService>` is injected into `TodosController`
- The database pool is accessed globally via `OnceLock` (no DI registration needed)
