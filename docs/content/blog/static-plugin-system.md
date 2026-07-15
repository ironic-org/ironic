---
title: "Static Plugin System — linked module composition with name collision prevention"
description: "A deep dive into Ironic's compile-time plugin architecture: how plugins extend module definitions, how duplicate names are caught before they crash, and why static linking beats runtime DynamicModule loading."
date: "2026-07-15"
author: "Ironic Team"
---

# Static Plugin System — linked module composition with name collision prevention

Most framework plugin systems are built on dynamic discovery — scan a directory for `.so` files, bounce through `dlopen`, and hope the contract holds at runtime. Ironic takes the opposite approach. Plugins are ordinary Rust structs compiled directly into your binary, registered at startup, and validated before any module graph is compiled. There is no reflection, no `unsafe` FFI, and no silent loading of unverified code. Let's walk through how it works.

---

## The `Plugin` trait: three methods, no magic

The entire contract lives in `crates/ironic-devtools/src/plugins.rs:25`:

```rust
pub trait Plugin: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn apply(&self, module: ModuleDefinitionBuilder) -> Result<ModuleDefinitionBuilder, PluginError>;
}
```

**`name()`** returns a stable, package-style identifier — `"redis-cache"`, `"open-telemetry"`, `"rate-limiter"`. This is the key Ironic uses for deduplication. Two plugins claiming the same name is a hard error before any module is built.

**`version()`** is purely diagnostic. It appears in the `inventory()` dump at startup alongside every registered plugin, giving operators an instant view of what's linked into the binary and at which version. No more guessing whether `v0.4.1` of the tracing plugin was actually pulled in.

**`apply()`** is where the composition happens. The plugin receives a `ModuleDefinitionBuilder` — the same fluent builder your `AppModule` uses — and returns a modified one. Inside `apply`, a plugin can call `.provider(...)`, `.controller(...)`, `.import::<T>()`, `.export::<T>()`, or `.import_if(enabled)`. The returned builder is threaded into the next plugin, forming a linear chain of transformations.

---

## `PluginRegistry`: a `Vec` and a `HashSet` for ordered, deduplicated registration

The registry (`plugins.rs:42`) is deceptively simple:

```rust
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
    names: HashSet<&'static str>,
}
```

The `Vec` preserves insertion order — plugins are applied in the exact sequence they were registered. The `HashSet` tracks every `name` seen so far. When `register()` is called, `HashSet::insert()` returns `true` on first insertion and `false` on a collision (`plugins.rs:59`). A `false` immediately short-circuits with `PluginError::Duplicate`. There is no fallback, no "last write wins" — the registry refuses to accept the second plugin outright.

This design choice matters because plugin order can be load-bearing. A `MetricsPlugin` that registers request-scoped middleware must run before a `TracePlugin` that wraps those same routes in spans. The `Vec` guarantees the sequence you declared, and the `HashSet` guarantees nobody accidentally registered the same plugin from two different initialization paths.

---

## Name collision prevention: fail fast, fail clearly

The collision check at `plugins.rs:59-61` is a single `if`:

```rust
if !self.names.insert(plugin.name()) {
    return Err(PluginError::Duplicate(plugin.name()));
}
```

`PluginError` (`plugins.rs:10`) has two variants:

- **`Duplicate(&'static str)`** — "plugin `{name}` is registered more than once." Caught at registration time, before any `apply()` calls fire. This means you can't silently shadow one plugin with another.
- **`Configuration { plugin: &'static str, message: String }`** — a plugin failed during `apply()`. The `message` field carries a safe, human-readable description. No backtraces leak into HTTP responses; the error is a structured value with the offending plugin's name attached.

Both variants are `#[non_exhaustive]`, so future releases can add new error kinds without breaking existing `match` arms.

Compare this to typical Node.js DI frameworks. If two packages register a provider called `"LOGGER"`, the container silently overwrites the first registration. You find out at 2 AM when the wrong logger is handling audit events. Ironic's collision detection catches this the moment `register()` is called — before `apply()`, before graph compilation, before any HTTP route is stood up.

---

## How `apply()` folds plugins over the builder

The `apply()` method on `PluginRegistry` (`plugins.rs:70`) is a linear fold:

```rust
pub fn apply(&self, mut module: ModuleDefinitionBuilder) -> Result<ModuleDefinitionBuilder, PluginError> {
    for plugin in &self.plugins {
        module = plugin.apply(module)?;
    }
    Ok(module)
}
```

Each plugin receives the builder output of the previous plugin. The first plugin gets the raw `ModuleDefinition::builder::<AppModule>()`. It adds providers. The second plugin gets that enriched builder. It adds controllers. The third adds imports. By the end of the loop, the builder carries contributions from every registered plugin, and the caller calls `.build()` to produce the final `ModuleDefinition`.

This is the same pattern NestJS uses with `DynamicModule.register()` — but applied at compile time. In NestJS, each `DynamicModule` is constructed imperatively at bootstrap via `forRoot()` or `registerAsync()`. In Ironic, plugins produce the same result through a folding chain, and the entire chain resolves before the module graph compiler inspects it.

---

## The `inventory()` diagnostic

For observability, the registry exposes `inventory()` (`plugins.rs:82`):

```rust
pub fn inventory(&self) -> Vec<(&'static str, &'static str)> {
    self.plugins.iter().map(|p| (p.name(), p.version())).collect()
}
```

This returns a `Vec` of `(name, version)` pairs in registration order. Applications typically call it once during startup and log the result:

```
INFO  ironic::bootstrap > plugin inventory: [("redis-cache", "0.1.0"), ("open-telemetry", "0.3.2")]
```

When a production incident involves version skew, this single log line answers the question immediately — no need to grep Cargo.lock diffs or inspect build artifacts.

---

## Static linking: no `dlopen`, no surprises

Plugins are compiled into the binary. There is no filesystem scan, no `unsafe` pointer casts, no `.so` hot-reload. This means three things:

1. **Type checking across plugin boundaries.** The compiler verifies that every `.provider()`, `.controller()`, and `.import()` call inside `apply()` is well-typed against the builder API. If a plugin references a type that doesn't implement `Injectable`, the build fails.

2. **Dead code elimination.** If a plugin is registered but its `apply()` only adds providers under `import_if(cfg!(feature = "redis"))`, the linker can strip the entire plugin when that feature flag is off.

3. **Auditable dependency surface.** Running `cargo tree` tells you exactly which plugins are linked. No runtime introspection needed.

---

## Plugins vs. modules: reusable extensions vs. application structure

Confusing plugins with modules is easy — both contribute providers, controllers, and imports. The distinction is scope.

A **module** (`MyAppModule`, `UsersModule`, `DatabaseModule`) defines the application's fixed architecture. It knows about domain types, business logic, and route trees. A module's `definition()` is hand-written for a specific application.

A **plugin** (`RedisCachePlugin`, `OpenTelemetryPlugin`) is generic. It doesn't know about your application's `UserRepository` or `OrderService`. It knows about Redis connections, tracing spans, and rate-limit counters. Its `apply()` method adds infrastructure-level providers to *any* `ModuleDefinitionBuilder` — the builder doesn't care which application it belongs to.

This separation means you can publish a `ironic-redis-plugin` crate on crates.io. Any Ironic application can add it with `plugins.register(RedisCachePlugin::default())` and instantly receive a scoped `RedisCache` provider without modifying its module tree.

---

## A concrete example: `RedisCachePlugin`

```rust
struct RedisCachePlugin {
    connection_url: String,
}

impl Plugin for RedisCachePlugin {
    fn name(&self) -> &'static str { "redis-cache" }
    fn version(&self) -> &'static str { "0.1.0" }

    fn apply(&self, builder: ModuleDefinitionBuilder) -> Result<ModuleDefinitionBuilder, PluginError> {
        let url = self.connection_url.clone();
        Ok(builder
            .provider(ProviderDefinition::factory(
                Scope::Singleton,
                vec![],
                move |_resolver| {
                    let url = url.clone();
                    Box::pin(async move {
                        let client = redis::Client::open(url.as_str())
                            .map_err(|e| ironic::DependencyError::Construction(e.to_string()))?;
                        Ok(RedisCache::new(client))
                    })
                },
            ))
            .export::<RedisCache>())
    }
}
```

The plugin adds a `RedisCache` singleton provider, exports it so any importing module can use it, and exposes no controllers or lifecycle hooks. A `RateLimiterPlugin` could then import `RedisCache` in its own `apply()` — the plugin chain composes infrastructure piece by piece.

---

## NestJS comparison: compile-time linking meets the builder pattern

NestJS plugins — `@nestjs/typeorm`, `@nestjs/jwt` — are `DynamicModule` factories. At bootstrap, the framework calls `TypeOrmModule.forRoot(config)` and receives a dynamically constructed module with providers, imports, and exports. The wiring happens in JavaScript, at runtime, through `Reflect.getMetadata()`.

Ironic's plugins produce the same result — a module definition with providers, controllers, imports, and exports — but the `apply()` chain runs before graph compilation and the resulting `ModuleDefinitionBuilder` is a concrete Rust struct, not a bag of runtime metadata. There's no `Map<string, unknown>`, no decorator reflection, no eager instantiation of providers. The plugin says "here's what I contribute," the builder accumulates it, and the graph compiler validates it.

The trade-off is clear: you cannot hot-reload an Ironic plugin without recompiling. In exchange, you get compile-time collision detection, type-safe builder calls, and a linker that strips unused plugin code. For server-side Rust applications where correctness matters more than runtime dynamism, that's exactly the right trade.

---

## Putting it together at bootstrap

```rust
fn main() {
    let mut plugins = PluginRegistry::new();
    plugins.register(RedisCachePlugin::new("redis://localhost:6379")).unwrap();
    plugins.register(OpenTelemetryPlugin::default()).unwrap();
    plugins.register(RateLimiterPlugin::new(100)).unwrap();

    ironic::logging::info!(
        plugins = ?plugins.inventory(),
        "Plugins registered"
    );

    let root_builder = ModuleDefinition::builder::<AppModule>();
    let enriched = plugins.apply(root_builder).expect("Plugin application failed");
    let graph = ironic::compile_module_graph(enriched.build()).expect("Graph compilation failed");

    ironic::bootstrap(graph).await;
}
```

In twenty lines, you've registered three plugins, logged their identities, folded their contributions into the root module, compiled the graph, and launched the server. If two plugins share a name, line 4 panics with a clear error. If one plugin's configuration is invalid, line 11 panics with `PluginError::Configuration`. No surprises. No silent failures. Just statically linked, validated composition.
