---
title: Deployment
description: Take your Ironic application from localhost to production — binary builds, Docker, environment config, reverse proxy, graceful shutdown, and health checks.
---

# Deployment

## What you'll learn

- Build a release binary with size optimizations
- Containerize with a multi-stage Dockerfile and Docker Compose
- Configure the app with environment variables
- Set up nginx as a reverse proxy with TLS termination
- Handle graceful shutdown with Ctrl‑C or orchestration signals
- Use the built-in `/health` endpoint and Docker HEALTHCHECK
- Verify your production readiness with a checklist

## Binary builds

Build the release binary from your project root:

```bash
cargo build --release
```

The binary lands at `target/release/{your-app-name}`. For a typical Ironic API, this is around 12–18 MB.

**Size optimization tips** — add these to `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Enable link-time optimization
codegen-units = 1    # Single codegen unit for better inlining
strip = "symbols"    # Strip debug symbols (or "debuginfo" on macOS)
```

These cut the binary to roughly 5–8 MB. The `strip` option requires Rust nightly or a post-build `strip` command.

## Docker

Ironic generates a multi-stage Dockerfile with every new project (`ironic new`). It compiles in a Rust builder image and copies the binary into a distroless runtime — no shell, no package manager, minimal attack surface:

```dockerfile
# Stage 1: Build
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Stage 2: Distroless runtime
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/{binary} /app/{binary}
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
EXPOSE 3000
CMD ["./{binary}"]
```

> `ENV SERVER_HOST=0.0.0.0` is **critical** for Docker. Without it, the app binds to `127.0.0.1` and is unreachable from outside the container.

Build and run:

```bash
docker build -t my-api .
docker run -p 3000:3000 --env-file .env my-api
```

## Docker Compose

Ironic also generates a standalone `docker-compose.yml` with app + PostgreSQL + Redis:

```yaml
services:
  app:
    build: .
    ports:
      - 3000:3000
    env_file: .env
    restart: unless-stopped
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: CHANGE_ME
      POSTGRES_DB: my_api
    ports:
      - 5432:5432
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d my_api"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - 6379:6379
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  pgdata:
```

Start everything with `docker compose up -d`.

## Environment configuration

Create a `.env` file (copy from `.env.example`). The generated `main.rs` reads these via `std::env::var` and `dotenvy`:

```bash
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Logging
RUST_LOG=info

# Security
CORS_ORIGINS=["https://app.com","https://admin.com"]
RATE_LIMIT_MAX=100

# Database
DATABASE_URL=postgres://user:CHANGE_ME@localhost:5432/mydb

# Redis
REDIS_URL=redis://localhost:6379
```

| Variable | Default | Purpose |
|----------|---------|---------|
| `SERVER_HOST` | `127.0.0.1` | Address to bind |
| `SERVER_PORT` | `3000` | Port to listen on |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `CORS_ORIGINS` | `[]` | JSON array of allowed origins |
| `RATE_LIMIT_MAX` | `100` | Max requests per IP per 60-second window |
| `DATABASE_URL` | (none) | PostgreSQL connection string |
| `REDIS_URL` | (none) | Redis connection string |

## Reverse proxy (nginx)

Place nginx in front of your app for TLS termination and static‑file serving:

```nginx
server {
    listen 443 ssl;
    server_name api.example.com;

    ssl_certificate     /etc/letsencrypt/live/api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name api.example.com;
    return 301 https://$host$request_uri;
}
```

## Graceful shutdown

Ironic handles Ctrl‑C automatically with `application.listen(&addr).await`.
Under the hood, it:

1. Catches `SIGINT` / `SIGTERM`
2. Stops accepting new connections (TCP backlog is closed)
3. Enters the **drain phase** — waits for in-flight requests to complete
4. Runs module lifecycle hooks in reverse initialization order
5. Exits the process

You can observe each phase in the logs:

```
INFO  Signal received, starting graceful shutdown...
INFO  Draining in-flight requests (max 30s)...
WARN  3 in-flight request(s) timed out and were dropped
INFO  Running shutdown hooks...
INFO  Application shutdown complete.
```

### Drain timeout

By default, the drain phase waits up to 30 seconds for in-flight requests to
complete.  Requests that exceed this timeout are dropped and a warning is
logged with the count of dropped requests.

```rust
use std::time::Duration;

let adapter = AxumAdapter::new()
    .drain_timeout(Duration::from_secs(60));  // 60 seconds
```

**Choosing a timeout:**

| Application type | Recommended drain timeout | Rationale |
|---|---|---|
| Real-time API (<100ms requests) | `10s` | Requests are fast; a short drain is sufficient |
| Standard web API | `30s` (default) | Balances safety with fast deploys |
| File upload / streaming | `60s`–`120s` | Long-running requests need more time |
| Websocket-heavy | `5s` | WebSocket connections are dropped — reconnecting clients is expected |

### Rolling deployment strategy

When deploying with an orchestrator (Kubernetes, Nomad, ECS):

1. Orchestrator sends `SIGTERM` to the old instance
2. Ironic stops accepting new connections
3. Load balancer detects the instance is draining and routes traffic elsewhere
4. In-flight requests complete (up to `drain_timeout`)
5. Process exits cleanly

The drain timeout should be shorter than your orchestrator's health check
failure threshold so the instance is terminated before being declared dead.

### Custom shutdown

For custom pre-shutdown logic (e.g., deregistering from a load balancer,
flushing buffered events, closing database connections):

```rust
use tokio::signal;
use ironic::ShutdownSignal;

let shutdown = async move {
    signal::ctrl_c().await.ok();

    // Custom pre-drain logic
    tracing::info!("Deregistering from load balancer...");
    deregister_from_lb().await;

    tracing::info!("Flushing pending events...");
    flush_event_buffer().await;

    ShutdownSignal::Interrupt
};

application.listen_with_shutdown(address, shutdown).await?;
```

Module lifecycle hooks (`on_application_shutdown`) fire automatically during
the drain phase — no extra wiring needed.

## Health checks

The built-in `HealthModule` serves `GET /health` and returns `{"status":"ok"}`. It's imported by every generated app:

```rust
#[derive(Module)]
#[module(imports = [HealthModule, /* ... */])]
pub struct AppModule;
```

For Docker, add a HEALTHCHECK instruction:

```dockerfile
HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
  CMD ["/app/{binary}", "health"] || exit 1
```

Or use curl in a non‑distroless image:

```dockerfile
HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1
```

## Production checklist

| Check | What to verify |
|-------|---------------|
| **SERVER_HOST** | Set to `0.0.0.0` in Docker/K8s; `127.0.0.1` with a reverse proxy |
| **SERVER_PORT** | Doesn't collide with other services |
| **RUST_LOG** | Set to `warn` or `error` to reduce noise; `info` for staging |
| **CORS_ORIGINS** | Restrict to your frontend origin(s) — never `*` in production |
| **RATE_LIMIT_MAX** | Tune based on expected traffic; default 100/min per IP |
| **RATE_LIMIT_BACKEND** | `RedisRateLimiter` for multi-replica; `InMemoryRateLimiter` for single |
| **SESSION_STORE** | `RedisSessionStore` for multi-replica; `InMemorySessionStore` for dev |
| **DATABASE_URL** | Uses strong password; never committed to source control |
| **REDIS_URL** | Uses strong password if exposed over network |
| **TLS** | Terminate at reverse proxy or load balancer; app listens plain HTTP |
| **DRAIN_TIMEOUT** | Set to match your max request duration (default 30s) |
| **HEALTHCHECK** | `/health` returns `200` and orchestration monitors it |
| **METRICS** | `/metrics` is scraped by Prometheus / VictoriaMetrics |
| **DISTROLESS** | No shell in runtime image; minimal attack surface |
| **ENV_FILE** | `.env` added to `.gitignore`; never committed to source |
| **BACKTRACES** | Set `RUST_BACKTRACE=1` for debugging, omit in production |

## Try it yourself

## Try it yourself

1. Run `ironic new demo` and `cd demo`
2. Build the image: `make docker-build`
3. Start the stack: `make docker-up`
4. Hit `http://localhost:3000/health` — you should see `{"status":"ok"}`
5. Run `docker compose logs app` to verify logs

## Common mistakes

| Mistake | Fix |
|---------|-----|
| App unreachable in Docker | Set `SERVER_HOST=0.0.0.0` — never `127.0.0.1` inside a container |
| `.env` committed to git | Add `.env` to `.gitignore`; only `.env.example` should be tracked |
| `CORS_ORIGINS` too permissive | Use exact origins (`["https://app.com"]`), not `["*"]` |
| No health check configured | Import `HealthModule` and add a HEALTHCHECK in Dockerfile |
| Distroless image can't debug | Ship a separate debug image with `alpine` and a shell if needed |
| Wrong binary name in COPY | The binary name is the kebab‑case project name with `-` replaced by `_` |

## What you learned

- [x] Build optimized release binaries with `opt-level="z"`, LTO, and stripping
- [x] Use a multi‑stage Dockerfile with distroless runtime
- [x] Orchestrate app + PostgreSQL + Redis with Docker Compose
- [x] Configure every aspect of the server via `.env` variables
- [x] Set up nginx for TLS termination and proxying
- [x] Understand graceful shutdown with `listen()` and `listen_with_shutdown()`
- [x] Configure drain timeout to control how long in-flight requests have to complete
- [x] Use the built‑in `/health` endpoint and Docker HEALTHCHECK
- [x] Run through a production readiness checklist

## Next steps

Learn how Ironic compares to raw frameworks:

→ [Benchmarks](./benchmarks)
