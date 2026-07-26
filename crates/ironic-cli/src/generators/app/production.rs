#[allow(clippy::too_many_lines)]
pub(crate) fn app_production_guide(name: &str, _port: u16) -> String {
    format!(
        r#"# Production Readiness Guide — {name}

This guide covers everything you need before deploying `{name}` to production.

---

## 1. OpenAPI / Swagger Documentation

Every API service should expose an OpenAPI 3.1 spec for consumers and tooling.

### Enable the Feature

```toml
ironic = {{ workspace = true, features = ["openapi"] }}
```

### Configure the Adapter

```rust
.platform(
    AxumAdapter::new()
        .with_openapi(OpenApiConfig::new("{name}", "0.1.0"))
        .swagger_ui("/docs"),   // Swagger UI at /docs
)
```

### Generate the Spec JSON

```bash
# Via CLI (recommended — auto build/start/fetch/save)
ironic openapi
ironic openapi -p {name} -o docs/openapi.json

# Via curl (CI/CD)
curl http://localhost:8080/openapi.json > spec.json
```

### Generate Client SDKs

```bash
npx openapi-typescript spec.json -o client.ts     # TypeScript
openapi-python-client generate --path spec.json    # Python
openapi-generator-cli generate -i spec.json -g go  # Go
```

### Validate the Spec

```bash
npx @redocly/cli lint spec.json
```

---

## 2. Middleware Stack

Add these middleware layers in order. Each addresses a specific production concern.

```rust
use std::time::Duration;
use ironic::security::{{CorsConfig, CorsMiddleware, RateLimitMiddleware, SecurityHeadersConfig, SecurityHeadersMiddleware}};
use ironic::metrics::{{MetricsLayer, MetricsConfig}};

Application::builder()
    // 1. Security headers (always first)
    .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))
    // 2. Rate limiting (per-IP, 100 req/min)
    .middleware(RateLimitMiddleware::new(100, 60))
    // 3. CORS (restrict to known origins)
    .middleware(CorsMiddleware::new(
        CorsConfig::new().allowed_origins(vec!["https://your-frontend.com"]),
    ))
```

| Middleware | What It Prevents | Feature |
|---|---|---|
| `SecurityHeadersMiddleware` | XSS, clickjacking, MIME sniffing, HSTS | `security` |
| `RateLimitMiddleware` | Brute force, DoS, API abuse | `security` |
| `CorsMiddleware` | Unauthorized cross-origin access | `security` |
| `MetricsLayer` | Prometheus metrics at `/metrics` | `metrics` |
| Body limit (5 MB) | Payload overflow | built-in |
| Timeout (30s) | Slow client DoS | built-in |

```toml
ironic = {{ workspace = true, features = ["security", "metrics", "logging", "openapi"] }}
```

---

## 3. Observability (Logging, Tracing, Metrics)

### Structured Logging (Development)

Included by default in generated apps — clean output without file paths:

```rust
tracing_subscriber::fmt()
    .with_env_filter("info")
    .with_target(false)
    .with_file(false)
    .with_line_number(false)
    .init();
```

### JSON Logging (Production)

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter("info")
    .init();
```

### Prometheus Metrics

Enable the `metrics` feature and add `MetricsLayer` to the router (see Middleware section above).
Metrics are exposed at `GET /metrics` in Prometheus format.

### Distributed Tracing (OTLP)

```toml
ironic = {{ features = ["telemetry"] }}
```

```rust
use ironic::telemetry::init_tracer;

init_tracer("{name}")?;
```

---

## 4. Error Handling

### Global Exception Filter

Catches unhandled errors and returns consistent JSON responses:

```rust
use ironic::prelude::*;

struct GlobalExceptionFilter;

impl ExceptionFilter for GlobalExceptionFilter {{
    fn catch(&self, error: &HttpError, _ctx: &FilterContext) -> Result<Response, HttpError> {{
        tracing::error!(%error, "unhandled exception");
        Err(HttpError::internal_server_error("INTERNAL_ERROR", "something went wrong"))
    }}
}}
```

### Register in Application

```rust
// Exception filters are registered per-route or globally via middleware.
// For a global handler, wrap your routes with a fallback middleware.
```

### Consistent Error Response Format

All errors should return JSON:

```json
{{"error": "ROUTE_NOT_FOUND", "message": "The requested route does not exist", "status": 404}}
```

---

## 5. Database

### Connection Pool

```rust
use std::sync::OnceLock;
use sqlx::postgres::PgPool;

static DB: OnceLock<PgPool> = OnceLock::new();

pub fn db() -> &'static PgPool {{
    DB.get().expect("database not initialized")
}}

pub async fn init_db(url: &str) -> PgPool {{
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .expect("failed to connect to database");
    sqlx::migrate!().run(&pool).await.expect("migrations failed");
    let _ = DB.set(pool.clone());
    pool
}}
```

### Environment Variables

```env
DATABASE_URL=postgres://user:password@host:5432/{name}
DB_POOL_SIZE=10
```

### Migrations

```bash
ironic migrate create add_users_table
# edit migrations/*.sql
ironic migrate up
```

---

## 6. Health Checks

Add the built-in health module to your root `AppModule`:

```rust
use ironic::health::HealthModule;

#[derive(Module)]
#[module(imports = [HealthModule])]
pub struct AppModule;
```

Exposes `GET /health` returning:

```json
{{"status": "ok"}}
```

Use this endpoint for:
- Load balancer health checks
- Kubernetes liveness/readiness probes
- Service discovery

---

## 7. Validation

### Request Body Validation

Use `#[validate]` with `garde` on DTOs:

```rust
use garde::Validate;

#[derive(Validate)]
pub struct CreateUserDto {{
    #[garde(length(min=1, max=100))]
    pub name: String,
    #[garde(email)]
    pub email: String,
}}
```

Enable the `validation` feature:

```toml
ironic = {{ features = ["validation"] }}
```

### Environment Validation

```rust
use ironic::prelude::*;

#[derive(ValidateConfiguration)]
pub struct AppConfig {{
    pub database_url: String,
    pub jwt_secret: String,
}}
```

---

## 8. CORS Configuration

```rust
CorsConfig::new()
    .allowed_origins(vec![
        "https://app.example.com",
        "https://admin.example.com",
    ])
    .allow_credentials(true)
```

For development, use `CorsConfig::permissive()` but **never** in production.

---

## 9. Rate Limiting

```rust
// 100 requests per minute per IP
RateLimitMiddleware::new(100, 60)
```

Adjust based on your API's usage patterns:
- Public endpoints: 30-60 req/min
- Authenticated endpoints: 100-300 req/min
- Webhooks/admin: 500+ req/min

---

## 10. Deployment

### Docker

The generated `Dockerfile` uses:
- **Alpine + musl** for fully static binaries (~10 MB)
- **scratch** final stage for minimal attack surface
- **Dependency caching** via `--mount=type=cache`

```bash
docker build -t {name} .
docker run -p 8080:8080 {name}
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
steps:
  - uses: dtolnay/rust-toolchain@stable
  - run: cargo fmt --check
  - run: cargo clippy -- -D warnings
  - run: cargo test
  - run: cargo build --release
```

### Release Build

```bash
ironic build -- --release
```

The workspace root `Cargo.toml` includes optimized release settings:

```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
panic = "abort"
strip = true
```

---

## 11. Security Checklist

- [ ] CORS origins restricted to known frontends (not `*`)
- [ ] Rate limiting enabled (start at 100 req/min per IP)
- [ ] Security headers enabled (XSS, CSP, HSTS, frame-guard)
- [ ] Request body size limited (5 MB default)
- [ ] Request timeout set (30s default)
- [ ] HTTPS enforced behind reverse proxy (nginx, Traefik, ALB)
- [ ] Database credentials use environment variables, not defaults
- [ ] Secrets managed via environment or vault — never in code
- [ ] OpenAPI docs disabled or behind auth in production
- [ ] `RUST_LOG` set to `info` or `warn` (not `debug`)
- [ ] Graceful shutdown confirmed — LB waits for health check to fail
- [ ] Dependencies audited: `cargo audit`

---

## 12. Performance Checklist

- [ ] Release profile with `lto`, `opt-level = "z"`, `panic = "abort"`
- [ ] Compression enabled via `.compression()` on the adapter
- [ ] Connection pooling tuned (start at 10, monitor and adjust)
- [ ] Prometheus metrics exported and scraped
- [ ] Structured JSON logging for log aggregation
- [ ] Database queries indexed — run `EXPLAIN ANALYZE` on slow queries
- [ ] Static assets served via CDN or reverse proxy
- [ ] Response pagination for list endpoints

---

## 13. Common Production Issues

| Issue | Cause | Solution |
|---|---|---|
| Slow startup | Compiling dependencies | Use `--mount=type=cache` in Docker |
| High memory | No release profile | Enable LTO + opt-level |
| Connection leaks | Pool not closed | Use `sqlx::PgPool` with max conns |
| OpenAPI spec missing | Feature not enabled | Add `openapi` feature |
| CORS errors | Wrong origin config | Check `CORS_ORIGINS` env var |
| Rate limiting too strict | Low limit | Adjust `RATE_LIMIT_MAX` |
"#,
    )
}
