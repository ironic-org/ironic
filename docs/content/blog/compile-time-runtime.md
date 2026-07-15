---
title: "How Ironic resolves dependencies at runtime — no decorator magic needed"
description: "A deep dive into conditional providers, scoped instances, and how Ironic's compile-time wiring leaves runtime flexibility intact."
date: "2026-07-15"
author: "Ironic Team"
---

# How Ironic resolves dependencies at runtime — no decorator magic needed

You know that moment when you look at a Rust framework and think, *"Wait, if all the wiring happens at compile time, how does it handle runtime decisions?"*

Fair question. Let's walk through it.

---

## The concern, restated

In NestJS, you write this:

```typescript
@Injectable({ scope: Scope.REQUEST })
class UserContext {
  constructor(@Optional() private cache: CacheService?) {}
}
```

The `@Injectable`, `scope`, and `@Optional` decorators all trigger **runtime reflection**. NestJS reads metadata at boot time, dynamically builds a dependency graph, and resolves things when requests come in.

Ironic does none of that. There's no reflection. No `Reflect.getMetadata()`. No `Map<string, unknown>`. Everything is a Rust struct in a Rust `HashMap`.

So the obvious question:

> If the proc macros run before the binary exists, how do you handle things that can only be decided at runtime? Like environment variables, conditional providers, or per-request scoped instances?

The answer is that Ironic doesn't push everything to compile time. It pushes **type safety** to compile time and leaves **value construction** for runtime. Let me show you.

---

## The three layers

Think of it like building a house.

| Layer | When it runs | What it does |
|-------|-------------|--------------|
| **Proc macros** | `cargo build` | Draws the blueprint — generates trait impls that return metadata |
| **Module graph** | `.build().await` | Inspects the blueprint — checks for missing rooms, circular hallways |
| **Container** | Request time | Actually builds the furniture — resolves providers on demand |

The blueprint is fixed at compile time. But what you put IN the rooms — that's decided at runtime.

---

## Layer 1: The macro writes a blueprint, not the actual object

When you write this:

```rust
#[derive(Injectable)]
#[injectable(scope = "request")]
struct UserContext {
    pub user_id: Option<u64>,
}
```

The macro generates:

```rust
impl Injectable for UserContext {
    fn provider_definition() -> ProviderDefinition {
        ProviderDefinition::factory(
            Scope::Request,          // ← "This will be per-request"
            vec![],                  // ← "It has no dependencies"
            |resolver| async move {  // ← "Here's how to build it"
                Ok(UserContext { user_id: None })
            },
        )
    }
}
```

Notice what the factory does: it says **"here's how to build one."** Not "here IS one." The factory is a function pointer — a recipe, not a meal.

The `ProviderDefinition` is just metadata:
```rust
pub struct ProviderDefinition {
    key: ProviderKey,            // (TypeId, "UserContext")
    scope: Scope,                // Singleton | Transient | Request
    eager: bool,                 // Create at boot?
    dependencies: Arc<[Dependency]>,  // What do I need?
    factory: Arc<ErasedFactory>, // How do I build it?
}
```

No value exists yet. Just a description of what KIND of value, what LIFETIME it has, what it DEPENDS on, and HOW to make one.

---

## Layer 2: The graph validates dependencies, but factories capture the outside world

The module graph compilation (`compile_module_graph()`) checks structural things at boot:

- "Is every dependency registered somewhere?" → Missing provider error
- "Does A depend on B which depends on A?" → Circular dependency error  
- "Does a singleton depend on a request-scoped provider?" → Scope violation error

But it does NOT call any factories. It just reads the metadata. The factories are stored in the `Container`, waiting to be invoked.

This is where the runtime flexibility lives. Factories can do anything:

```rust
// Factory that reads an environment variable at RUNTIME:
ProviderDefinition::constructor(
    Scope::Singleton,
    vec![],
    |_| {
        let db_url = std::env::var("DATABASE_URL")?;  // ← runtime!
        Ok(PgPool::connect_lazy(&db_url)?)
    },
)
```

```rust
// Factory that uses conditional logic:
ProviderDefinition::factory(
    Scope::Singleton,
    vec![],
    |resolver| async move {
        let is_dev = std::env::var("ENV").unwrap_or_default() == "development";
        if is_dev {
            Ok(Logger::dev())
        } else {
            Ok(Logger::production(resolver.resolve::<Config>().await?))
        }
    },
)
```

The graph doesn't know or care what the factory does internally. It just knows the factory exists, its dependencies (if any), and its scope.

---

## Layer 3: How scopes actually work at runtime

The `Container` is just a fancy `HashMap`:

```rust
// Simplified — the real thing:
struct Container {
    providers: HashMap<ProviderKey, RegisteredProvider>,
}

struct RegisteredProvider {
    definition: ProviderDefinition,
    singleton_cache: OnceCell<ProviderValue>,  // ← for singletons
}
```

When you call `container.resolve::<T>()`, it:

1. Looks up the `ProviderKey` for `T` in the HashMap
2. Checks the scope:
   - **Singleton** → `singleton_cache.get_or_init(|| factory.resolve())` — builds once, caches forever
   - **Transient** → calls the factory directly — no cache, builds fresh every time
   - **Request** → returns an error — "you need a RequestScope for this"

For singletons, `OnceCell` is the hero. It's a thread-safe, lazy-initialized cell. First call builds it. All subsequent calls return the same value. If the factory fails, the error is returned, and the next call can retry.

For request-scoped providers, you use `RequestScope`:

```rust
// Inside an HTTP request handler — this is auto-created for you:
let scope = container.request_scope();
let ctx = scope.resolve::<UserContext>().await?;
```

`RequestScope` wraps the container with a fresh per-request cache:

```rust
struct RequestScope {
    container: Arc<Container>,
    cache: Mutex<HashMap<ProviderKey, Arc<OnceCell<ProviderValue>>>>,
}
```

The per-request HashMap starts empty. When you resolve a request-scoped provider, it:
1. Checks the request cache first
2. If not found, calls the factory and stores the result
3. Next resolve of the same type in the same request → returns the cached value
4. Next HTTP request → new `RequestScope` → new cache → new instance

Singletons never touch this cache. They use the container-level `OnceCell`. That's how the scope violation is enforced — if a singleton factory tries to resolve a request-scoped provider, the `Resolver` sees the missing `request_cache` and returns an error.

---

## Layer 4: Conditional providers — no runtime branching needed

In NestJS, `@Optional()` means: "at runtime, check if this exists in the container, and set it to undefined if not."

In Ironic, the same thing happens at the **type level**:

```rust
#[derive(Injectable)]
struct AnalyticsService {
    cache: Option<Arc<CacheService>>,  // ← Option IS the conditional
}
```

When the proc macro sees `Option<Arc<CacheService>>`, it generates:

```rust
fn provider_definition() -> ProviderDefinition {
    ProviderDefinition::factory(
        Scope::Singleton,
        vec![Dependency::optional::<CacheService>()],  // ← "optional = true"
        |resolver| async move {
            let cache = resolver.resolve_optional::<CacheService>().await?;
            //         ^^^^^^^^^^^^^^^^^ — returns Option<Arc<T>>, not Result
            //         Returns None if CacheService isn't registered
            Ok(AnalyticsService { cache })
        },
    )
}
```

At resolve time:

```rust
async fn resolve_optional<T>(&self) -> Result<Option<Arc<T>>, ResolveError> {
    match self.resolve_erased(key) {
        Ok(value) => Ok(Some(value.downcast()?)),
        Err(ResolveError::MissingProvider { .. }) => Ok(None),  // ← not an error!
        Err(e) => Err(e),
    }
}
```

No decorator, no metadata lookup. Just `Option<T>` at the type level and a `match` on the result. The compiler guarantees correctness because `Option<Arc<T>>` and `Arc<T>` are different types.

---

## Putting it all together: a real request

Here's what happens when a request hits your API, from the container's perspective:

```
[HTTP Request arrives]
        │
        ▼
RequestScope is created (fresh cache)
        │
        ▼
Handler calls scope.resolve::<RequestContext>()
        │
        ├─► Check request cache → miss
        ├─► Call factory → constructs RequestContext { user_id: None }
        ├─► Store in request cache
        └─► Return Arc<RequestContext>
        │
        ▼
Handler calls scope.resolve::<ClaimService>()
        │
        ├─► Check request cache → miss
        ├─► Scope is Singleton
        ├─► Check container-level OnceCell → first time?
        │     ├─► YES → call factory
        │     │     └─► Factory resolves ConfigService (also singleton, OnceCell hit)
        │     │         └─► Constructs ClaimService
        │     └─► NO → return cached value
        └─► Return Arc<ClaimService>
        │
        ▼
Handler calls scope.resolve::<RequestContext>() AGAIN
        │
        ├─► Check request cache → HIT!
        └─► Return same Arc<RequestContext> as before
        │
        ▼
[Response sent]
[RequestScope dropped → per-request cache freed]
[Next request → new RequestScope → new instances]
```

The key insight: the `ProviderDefinition` was generated at compile time by a proc macro. But the factories inside it are called at **runtime**, by the **container**, using a **HashMap** and **OnceCell** that were set up at **boot time**.

The compile-time part is the blueprint. The runtime part is the construction.

---

## The trade, honestly

| What NestJS gives you | What Ironic gives you instead |
|---|---|
| `@Injectable()` decorator with runtime metadata | `#[derive(Injectable)]` proc macro → trait impl |
| `@Optional()` runtime check | `Option<Arc<T>>` at the type level |
| `@Inject('TOKEN')` string-based resolution | `ProviderKey::of::<T>()` — TypeId-based, zero collision risk |
| `scope: Scope.REQUEST` on decorator | `#[injectable(scope = "request")]` → generated enum variant |
| Dynamic module compilation at boot | `compile_module_graph()` validates the graph, factories are lazy |
| Reflect-based parameter type detection | Proc macro reads struct fields directly from the AST |

You lose the ability to dynamically register new providers after the container is built. That's the real trade: Ironic's container is frozen after `.build().await`.

In practice, this rarely matters. Everything that needs runtime flexibility — reading env vars, loading config, connecting to databases — happens **inside the factories**, which are called lazily at runtime.

The decorator magic was always a workaround for JavaScript's lack of compile-time types. Rust doesn't have that gap to fill.

---

## The bottom line

- Proc macros draw the blueprint — they say *what exists* and *how to build it*
- The container is the construction crew — it builds things *on demand*, at *runtime*
- Factories capture the outside world — env vars, files, network connections
- `OnceCell` + `HashMap` replace reflection — no global mutable state, no locks on the hot path
- Scopes are just where the cache lives — container-level for singletons, request-level for per-request

The compile time / runtime split isn't a binary choice. It's a boundary. Ironic puts types on the compile-time side and values on the runtime side. That's the whole trick.
