# Setup guide

## Prerequisites

- Rust 1.97+
- Docker (for PostgreSQL)
- Ironic CLI (`cargo install ironic`)

## Quick start

```bash
# 1. Clone and enter the project
cd examples/todo-app

# 2. Configure environment
cp .env.example .env
# Edit .env — set DATABASE_URL with your credentials

# 3. Start PostgreSQL
docker compose up -d postgres

# 4. Run the app
cargo run
```

Open http://localhost:3000.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `SERVER_HOST` | `127.0.0.1` | Bind address |
| `SERVER_PORT` | `3000` | Bind port |
| `DATABASE_URL` | — | PostgreSQL connection string |
| `DB_POOL_SIZE` | `10` | Max database connections |
| `CORS_ORIGINS` | `[]` | JSON array of allowed origins |
| `RATE_LIMIT_MAX` | `100` | Max requests per 60s window |
| `RUST_LOG` | `info,todo-example=debug` | Logging filter |

## Database

The project includes Docker Compose with PostgreSQL 16:

```bash
# Start only the database
docker compose up -d postgres

# Start everything (app + db)
docker compose up -d

# View logs
docker compose logs -f
```

Migrations run automatically on startup via `sqlx::migrate::Migrator`.
