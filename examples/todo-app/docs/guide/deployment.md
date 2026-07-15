# Deployment

## Docker

```bash
# Build image
docker build -t todo-example .

# Run with PostgreSQL
docker compose up -d
```

## Docker Compose

The included `docker-compose.yml` runs the app + Postgres 16 + Redis 7:

```yaml
services:
  app:     # Your todo API on port 3000
  postgres:# PostgreSQL 16 with health check
  redis:   # Redis 7 with health check
```

## Production checklist

- [ ] Set `SERVER_HOST=0.0.0.0`
- [ ] Set `RUST_LOG=info` (or `warn` for production)
- [ ] Set `CORS_ORIGINS` to your frontend domain
- [ ] Set `RATE_LIMIT_MAX` appropriate for your traffic
- [ ] Use a managed PostgreSQL (RDS, Cloud SQL, etc.)
- [ ] Use a secrets manager for `DATABASE_URL`
- [ ] Enable health checks (`GET /health`)

## Health endpoint

```http
GET /health
```

Response:

```json
{
    "status": "ok"
}
```
