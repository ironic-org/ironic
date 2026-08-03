---
title: Examples
description: A complete, runnable blog API built with Ironic — cross-module DI, JWT auth, CRUD with categories, caching, custom decorators and interceptors, cron tasks, OpenAPI, and 10 unit tests.
---

# Examples

Each example is a complete, runnable project. Read the code in the repo — it's the
fastest way to see how the pieces fit together.

| Example | What it demonstrates |
|---------|---------------------|
| [blog](https://github.com/ironic-org/ironic/tree/main/examples/blog) | Cross-module DI, JWT auth, CRUD with categories, in-memory repositories, stats module, filtering, slug management, custom decorators/interceptors, caching, cron tasks, OpenAPI |

---

## Running the example

```bash
git clone https://github.com/ironic-org/ironic
cd ironic/examples/blog
cargo run
```

The server listens on `http://0.0.0.0:3002` by default (`SERVER_HOST` / `SERVER_PORT`
env vars). Copy `.env.example` to `.env` to override settings such as the JWT secret,
CORS origins, and rate limit.

### Try it

Login with the demo credentials (`admin` / `ironic`):

```bash
curl -X POST http://localhost:3002/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"ironic"}'
```

Use the returned `access_token`:

```bash
TOKEN="<access_token from login>"
curl http://localhost:3002/api/blogs \
  -H "Authorization: Bearer $TOKEN"
```

Useful URLs while it's running:

| URL | What it is |
|-----|------------|
| `http://localhost:3002/` | API root with endpoint overview |
| `http://localhost:3002/openapi.json` | Generated OpenAPI JSON |
| `http://localhost:3002/docs` | Swagger UI |
| `http://localhost:3002/health` | Health check |
| `http://localhost:3002/metrics` | Prometheus metrics |

---

## What the example covers

The `blog-api` crate (package name — the folder is `examples/blog`) is a complete
blog platform. It wires together most of Ironic's headline features in one small,
readable codebase:

### Application bootstrap

`src/main.rs` builds the application with security middleware, a rate limit, CORS,
compression, metrics, and OpenAPI in one fluent chain:

```rust
let application = Application::builder()
    .module(AppModule::definition())
    .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))
    .middleware(RateLimitMiddleware::new(rate_limit_max, 60))
    .middleware(CorsMiddleware::new(CorsConfig::new().allowed_origins(cors_origins)))
    .platform(
        AxumAdapter::new()
            .compression()
            .request_body_limit(5 * 1024 * 1024)
            .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
            .with_openapi(openapi_config)
            .swagger_ui("/docs"),
    )
    .build()
    .await
    .expect("application must initialise");
```

### Module graph & cross-module DI

`src/app.rs` declares the root module. Each feature lives in its own module under
`src/modules/`:

```rust
#[derive(Module)]
#[module(imports = [HealthModule, TimeSeriesModule, MetricsModule, WelcomeModule,
                     AuthModule, BlogsModule, StatsModule, TasksModule])]
pub struct AppModule;
```

`StatsModule` and `TasksModule` both `imports = [BlogsModule]` and inject the
**exported** `BlogService` — the core demonstration of cross-module dependency
injection:

```rust
// modules/stats/services/stats_service.rs
#[derive(Injectable)]
pub struct StatsService {
    blog_service: Arc<BlogService>,  // injected from BlogsModule's exports
}
```

### Auth (JWT)

`AuthModule` issues short-lived access tokens (1 h) and refresh tokens (7 days) via
`jsonwebtoken`. `JwtGuard` protects every blog route and any route annotated with
`#[guard(JwtGuard)]`:

```rust
#[post("/login")]
#[api(summary = "Login", tag = "Auth")]
#[body(json = LoginDto)]
#[resp(200, "Login successful", json = TokenResponse)]
#[resp(401, "Invalid credentials")]
async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
    let tokens = self.auth.login(&dto.username, &dto.password)
        .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
    Ok(Json(tokens))
}
```

### Controllers, sub-resources & filtering

`BlogsController` shows full CRUD, slug lookup, publish/unpublish, per-post category
sub-resources (`/api/blogs/:id/categories`), and query-based filtering:

```rust
#[get("/:id")]
#[cache(ttl_secs = 60)]
#[api(summary = "Get blog post", tag = "Blogs", security = "bearerAuth")]
#[resp(200, "Blog post", json = BlogPost)]
#[resp(404, "Post not found")]
async fn get(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
    let post = self.service.find(id)?;
    Ok(Json(post))
}
```

### Caching

Read-heavy GET routes are cached with `#[cache(ttl_secs = ...)]` (30 s for lists,
60 s for single posts, 120 s for stats). The cache key includes the full URL path
and query string.

### Custom decorator (parameter extractor)

`Pagination` is a hand-rolled `ParameterExtractor` that reads `?page=` / `?size=`
from the query string, then a handler uses it with `#[decorator(Pagination)]`:

```rust
pub struct Pagination;

impl ParameterExtractor for Pagination {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let query = context.request().uri().query().unwrap_or_default();
            let page = get_param(query, "page").unwrap_or(1);
            let size = get_param(query, "size").unwrap_or(20).min(100);
            Ok(Box::new(PaginationParams { page, size }) as ExtractedValue)
        })
    }
}
```

### Custom interceptor

`TimingInterceptor` wraps write endpoints and logs method, path, status, and duration:

```rust
impl Interceptor for TimingInterceptor {
    fn intercept<'a>(&'a self, context: &'a mut RequestContext, next: InterceptorNext<'a>) -> PipelineFuture<'a> {
        Box::pin(async move {
            let start = Instant::now();
            let response = next.run(context).await?;
            let elapsed = start.elapsed();
            ironic::logging::log::info!(
                target: "ironic.http.timing",
                http_method = %context.request().method(),
                http_path = %context.request().uri().path(),
                http_status = response.status().as_u16(),
                duration_ms = elapsed.as_secs_f64() * 1000.0,
            );
            Ok(response)
        })
    }
}
```

### Lifecycle hooks

- `BlogsModule` declares `lifecycle_init = [BlogService]` — on `OnModuleInit`, the
  service seeds 2 blog posts and 3 categories.
- `TasksModule` declares `lifecycle_bootstrap = [StatsReporter]` — on
  `OnApplicationBootstrap`, it starts a cron task that logs blog stats every minute:

```rust
impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> ironic::LifecycleFuture<'_> {
        let svc = Arc::clone(&self.service);
        Box::pin(async move {
            let _task = ironic::services::scheduling::cron("0 * * * * *", move || {
                let svc = Arc::clone(&svc);
                async move { /* log svc.stats() */ }
            });
            Ok(())
        })
    }
}
```

### Validation

`CreateBlogDto` and friends derive `garde::Validate`; `#[body]` validates incoming
payloads automatically and returns a `422` on failure.

### Testing

`modules/blogs/tests/blog_tests.rs` contains **10 `#[ironic::test]` integration-style
unit tests** covering creation, slug generation and conflict detection, updates,
publish/unpublish, deletion, category CRUD, category assignment, stats, and a
route-level exception filter:

```rust
#[ironic::test]
async fn test_slug_conflict() {
    let svc = make_service();
    svc.create(CreateBlogDto { title: "Same Title".into(), ..Default::default() })
        .expect("first create should succeed");

    let result = svc.create(CreateBlogDto { title: "Same Title".into(), ..Default::default() });
    assert!(result.is_err());
}
```

Run them from the example directory:

```bash
cd examples/blog
cargo test
```

---

## Endpoint reference

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| `GET` | `/` | API root + endpoint overview | — |
| `POST` | `/api/auth/login` | Exchange credentials for tokens | — |
| `POST` | `/api/auth/refresh` | Refresh the access token | — |
| `GET` | `/api/auth/me` | Current user from Bearer token | JWT |
| `GET` | `/api/blogs` | List posts (filter + paginate) | JWT |
| `GET` | `/api/blogs/:id` | Get a post | JWT |
| `GET` | `/api/blogs/slug/:slug` | Get a post by slug | JWT |
| `POST` | `/api/blogs` | Create a post | JWT |
| `PUT` | `/api/blogs/:id` | Update a post | JWT |
| `DELETE` | `/api/blogs/:id` | Delete a post | JWT |
| `POST` | `/api/blogs/:id/publish` | Publish a draft | JWT |
| `POST` | `/api/blogs/:id/unpublish` | Unpublish a post | JWT |
| `GET` | `/api/blogs/stats` | Post statistics | JWT |
| `GET` | `/api/blogs/:id/categories` | Categories of a post | JWT |
| `POST` | `/api/blogs/:id/categories/:category_id` | Assign a category | JWT |
| `DELETE` | `/api/blogs/:id/categories/:category_id` | Remove a category | JWT |
| `GET` | `/api/categories` | List categories | — |
| `POST` | `/api/categories` | Create a category | JWT |
| `DELETE` | `/api/categories/:id` | Delete a category | JWT |
| `GET` | `/api/stats/blog` | Blog summary from `StatsService` | — |
| `GET` | `/api/stats/blog/tags` | Tag frequency breakdown | — |
| `GET` | `/health` | Health check | — |
| `GET` | `/metrics` | Prometheus metrics | — |

### Filtering

`GET /api/blogs` accepts these query parameters (parsed into `BlogFilterDto`):

```json
{
  "published": true,
  "author": "Alice",
  "tag": "rust",
  "category_id": "uuid",
  "search": "keyword"
}
```

---

## Project structure

```
examples/blog/
├── Cargo.toml
└── src/
    ├── main.rs               # Application entry point
    ├── app.rs                # Root module — imports all modules
    ├── welcome.rs            # Homepage endpoint
    ├── platform/config.rs    # Server configuration
    └── modules/
        ├── blogs/            # Primary module — blog posts + categories
        │   ├── entities/     # BlogPost, Category structs
        │   ├── dto/          # CreateBlogDto, UpdateBlogDto, BlogFilterDto
        │   ├── repositories/ # BlogRepository, CategoryRepository (in-memory)
        │   ├── services/     # BlogService — all business logic
        │   ├── controller/   # BlogsController, CategoriesController
        │   └── tests/        # 10 unit tests
        ├── stats/            # Cross-module DI demo — consumes BlogService
        │   ├── services/     # StatsService (injects Arc<BlogService>)
        │   └── controller/   # StatsController
        ├── auth/             # JWT login/refresh + JwtGuard
        ├── tasks/            # OnApplicationBootstrap cron reporter
        ├── decorators/       # Pagination parameter extractor
        └── interceptors/     # TimingInterceptor
```

## Key implementation details

### In-memory repository

Repositories use `LazyLock<Mutex<HashMap>>` for thread-safe in-memory storage. This
keeps the example self-contained without a database:

```rust
static BLOG_POSTS: LazyLock<Mutex<HashMap<Uuid, BlogPost>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
```

### Slug generation

Titles are converted to URL-friendly slugs with duplicate detection:

```rust
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}
```

### DTO validation

Create requests are validated at the boundary with `garde`:

```rust
#[derive(Validate)]
pub struct CreateBlogDto {
    #[garde(length(min = 1, max = 200))]
    pub title: String,

    #[garde(length(min = 1))]
    pub content: String,
}
```

---

## What you learned

- [x] `blog-api` = cross-module DI (module `imports` + `exports` + provider injection)
- [x] Full CRUD, sub-resource routing (`:id/categories/:category_id`), filtering, slug generation
- [x] JWT auth with guards, caching, custom decorators, interceptors, and middleware
- [x] Lifecycle hooks (`OnModuleInit` seeding, `OnApplicationBootstrap` cron)
- [x] OpenAPI + Swagger UI generation from route metadata
- [x] 10 unit tests covering the business logic
