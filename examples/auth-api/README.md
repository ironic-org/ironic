# auth-api

Built with [Ironic](https://github.com/ironic-org/ironic) v0.3.5.

## Quick start

```bash
# Install Ironic CLI
cargo install ironic

# Run with hot reload
ironic dev

# Or run directly
cargo run
```

Open http://localhost:3000 in your browser.

## Commands

| Task | Command |
|------|--------|
| Start dev server | `make dev` |
| Run tests | `make test` |
| Build | `make build` |
| Format | `make fmt` |
| Lint | `make clippy` |

## Docker

```bash
make docker-up    # Start app + postgres + redis
make docker-down  # Stop everything
make docker-build # Build image only
```

## Endpoints

| Path | Description |
|------|-------------|
| `GET /` | Welcome JSON |
| `GET /health` | Health check |
| `GET /example` | Example CRUD |

## Environment

Copy `.env.example` to `.env` and adjust values.
