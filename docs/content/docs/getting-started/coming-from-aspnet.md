---
title: Coming from ASP.NET
description: A guide for ASP.NET developers transitioning to Ironic. Concepts, patterns, and code comparisons.
---

# Coming from ASP.NET

If you're coming from ASP.NET Core, you'll find many familiar concepts in Ironic — dependency injection, middleware pipelines, controllers, and configuration are all first-class features.

## Concept mapping

| ASP.NET | Ironic | Notes |
|---------|--------|-------|
| `Startup.cs` / `Program.cs` | `Application::create()` | Application entry point with module registration |
| `IServiceCollection` | `ContainerBuilder` | DI container registration |
| `[ApiController]` | `#[controller]` | Controller attribute with route prefix |
| `[HttpGet]` / `[HttpPost]` | `#[get]` / `#[post]` | Route method attributes |
| `[FromBody]` / `[FromRoute]` | `JsonBody<T>` / `PathParameter` | Typed extractors |
| `IActionFilter` | `Interceptor` | Pre/post handler hooks |
| `IExceptionFilter` | `ExceptionFilter` | Centralized error handling |
| `IAuthorizationFilter` | `Guard` | Authorization before handler |
| `Middleware` | Tower middleware layers | Request pipeline |
| `IConfiguration` | `ConfigurationLoader` | Typed configuration |
| `IHostedService` | Background services | Cron scheduling, event bus |
| `app.UseCors()` / `app.UseAuth()` | Middleware layers | Security middleware |
| `ILogger<T>` | Tracing macros (`info!`, `warn!`, etc.) | Structured logging |
| `HttpClient` | `reqwest` via DI | HTTP client (bring your own) |
| `Entity Framework` | SQLx / SeaORM / Diesel | Database access |
| `IOptions<T>` | `ConfigurationLoader::load::<T>()` | Typed options pattern |

## Key differences

| Aspect | ASP.NET | Ironic |
|--------|---------|--------|
| Runtime | .NET (CLR, JIT) | Rust (native, compiled) |
| Memory management | GC (Garbage Collector) | Ownership + borrow checker |
| Async model | `Task<T>` | `Future` + tokio |
| DI resolution | Runtime (reflection) | Compile-time (generics) |
| Configuration | `appsettings.json` + env | Layered TOML/JSON + env |
| Package manager | NuGet | Cargo |
| Project format | `.csproj` / solution | `Cargo.toml` / workspace |
| ORM | EF Core (LINQ, migrations) | SQLx (compile-time checked SQL) |

## Controller comparison

ASP.NET:
```csharp
[ApiController]
[Route("api/users")]
public class UsersController : ControllerBase
{
    private readonly IUserService _service;

    public UsersController(IUserService service)
    {
        _service = service;
    }

    [HttpGet("{id}")]
    public async Task<ActionResult<User>> Get(int id)
    {
        var user = await _service.GetByIdAsync(id);
        if (user == null) return NotFound();
        return Ok(user);
    }
}
```

Ironic:
```rust
#[controller("/api/users")]
struct UsersController {
    service: Arc<UserService>,
}

#[routes]
impl UsersController {
    #[get("/{id}")]
    async fn get(&self, id: PathParameter<i32>) -> Result<Json<User>, HttpError> {
        let user = self.service.get_by_id(*id).await?;
        Ok(Json(user))
    }
}
```

## Service / DI pattern

ASP.NET:
```csharp
public interface IUserService
{
    Task<User?> GetByIdAsync(int id);
}

public class UserService : IUserService
{
    private readonly AppDbContext _db;

    public UserService(AppDbContext db)
    {
        _db = db;
    }
}

// In Program.cs
builder.Services.AddScoped<IUserService, UserService>();
```

Ironic:
```rust
#[injectable]
impl UserService {
    fn new(db: Arc<DatabaseProvider>) -> Self {
        Self { db }
    }
}

// In module definition
ProviderDefinition::constructor(Scope::Request, Vec::new(), |resolver| async {
    let db = resolver.resolve::<DatabaseProvider>().await?;
    Ok(UserService::new(db))
})
```

## Middleware pipeline

ASP.NET:
```csharp
app.UseCors()
   .UseAuthentication()
   .UseAuthorization();
```

Ironic:
```rust
AxumAdapter::new()
    .configure_router(|router| {
        router
            .layer(CorsLayer::new(config))
            .layer(AuthLayer::new())
    })
```

## What you'll need to learn

- **Rust ownership and borrowing** — The biggest shift from C#. Start with the [Rust Book](https://doc.rust-lang.org/book/)
- **`Result<T, E>` instead of exceptions** — No try/catch; errors are values returned from functions
- **Async with tokio** — Similar to `Task<T>` but with explicit runtime
- **Traits instead of interfaces** — Rust traits are more powerful (associated types, default impls)
- **No null** — `Option<T>` replaces nullable references
- **Cargo instead of NuGet** — Build tool, package manager, and test runner in one
