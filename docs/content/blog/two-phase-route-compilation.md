---
title: "Two-phase route compilation — path normalization and cross-controller conflict detection"
description: "How Ironic compiles controller routes in two phases: normalizing paths, resolving prefixes, detecting intra-controller duplicates, then performing cross-controller conflict detection with version-aware deduplication."
date: "2026-07-15"
author: "Ironic Team"
---

# Two-phase route compilation — path normalization and cross-controller conflict detection

HTTP frameworks often discover route conflicts at runtime, if at all. Two controllers silently register `GET /users` and the second one wins. Ironic rejects this at startup through a two-phase compilation pipeline in `crates/ironic-http/src/route.rs`. Phase one compiles each controller's routes in isolation, joining prefixes, normalizing paths, and checking for internal duplicates. Phase two merges all compiled routes and detects cross-controller conflicts with version-aware deduplication.

## Phase one: per-controller compilation

The entry point is `ControllerDefinition::compile_routes()` at line 393. Each controller carries a base path (the controller prefix), a list of route definitions, and optional pipeline components. The compiler iterates through every route and produces `CompiledRoute` structs:

```rust
pub(crate) fn compile_routes(&self) -> Result<Vec<CompiledRoute>, RouteError> {
    let mut seen = HashSet::new();
    let mut compiled = Vec::with_capacity(self.routes.len());
    for route in &self.routes {
        let path = join_paths(&self.path, route.path());
        if !seen.insert((route.method.clone(), path.clone())) {
            return Err(RouteError::DuplicateRoute { method: route.method.clone(), path });
        }
        // ...merge pipelines, inject controller pipes, build CompiledRoute
    }
    Ok(compiled)
}
```

### Path joining: `join_paths()`

The function at line 731 is deliberately simple — it handles three cases:

```rust
fn join_paths(prefix: &str, path: &str) -> String {
    if prefix == "/" { return path.to_owned(); }
    if path == "/" { return prefix.to_owned(); }
    format!("{prefix}{path}")
}
```

No slash-doubling logic is needed here because paths are normalized before they enter compilation. When a `ControllerDefinition` is created (line 298) and when each `RouteDefinition` is constructed (line 116), both call `normalize_path()` on their inputs. By the time `join_paths` runs, the prefix always starts with `/` and the route path always starts with `/` — the `format!("{prefix}{path}")` concatenation naturally produces `/controller-prefix/route-path` without duplicates.

The edge case guard (`prefix == "/"` or `path == "/"`) prevents producing `//` for root-prefixed routes. If both are `"/"`, the prefix check fires first and returns `"/"`.

### Path normalization: `normalize_path()`

The normalizer at line 713 is the workhorse for cleaning up user-supplied path strings:

```rust
fn normalize_path(path: &str) -> Result<String, RouteError> {
    if !path.starts_with('/') {
        return Err(RouteError::InvalidPath { path: path.to_owned() });
    }
    let normalized = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if normalized.is_empty() { Ok("/".to_owned()) }
    else { Ok(format!("/{normalized}")) }
}
```

It enforces three invariants:

1. Every path must start with `/` — relative paths are rejected at compile time.
2. Empty segments from doubled slashes (`//users`) are stripped by `filter(|segment| !segment.is_empty())`.
3. A leading `/` is prepended after the `join("/")`, so the result always has exactly one leading slash.

Examples of the transformation:

| Input | Output |
|---|---|
| `"/users"` | `"/users"` |
| `"//users//:id/"` | `"/users/:id"` |
| `"/users/:id/"` | `"/users/:id"` |
| `"///"` | `"/"` |

### Intra-controller duplicate detection

Inside the per-controller loop, a `HashSet<(HttpMethod, String)>` tracks `(method, path)` tuples. If `seen.insert()` returns `false`, the compiler immediately returns `RouteError::DuplicateRoute` with the method and path. This catches two routes on the same controller that somehow ended up with identical method+path combinations after normalization and prefix joining.

When no duplicates are found, the compiler merges the controller's pipeline with each route's individual pipeline, injects controller-level parameter pipes, and attaches the controller's version metadata (if present) to every compiled route. The resulting vector of `CompiledRoute` values is returned.

## Phase two: cross-controller conflict detection

Once every controller has been individually compiled, `compile_controller_routes()` at line 688 merges them:

```rust
pub fn compile_controller_routes(
    controllers: impl IntoIterator<Item = ControllerDefinition>,
) -> Result<Vec<CompiledRoute>, RouteError> {
    let mut routes = Vec::new();
    let mut seen = HashSet::new();
    for controller in controllers {
        for route in controller.compile_routes()? {
            let version = route.metadata.get::<VersionMetadata>()
                .map(|v| (v.version.clone(), v.strategy.clone()));
            if !seen.insert((route.method.clone(), route.path.clone(), version)) {
                return Err(RouteError::DuplicateRoute {
                    method: route.method,
                    path: route.path,
                });
            }
            routes.push(route);
        }
    }
    Ok(routes)
}
```

The deduplication key is a 3-tuple: `(HttpMethod, String, Option<(String, VersioningStrategy)>)`. This means two routes are considered conflicting only if they share the same HTTP method, the same normalized path, *and* the same (or absent) version metadata.

### Why VersionMetadata is in the key

Ironic supports API versioning through `VersionMetadata` (defined in `crates/ironic-http/src/metadata.rs:17`), which carries a version string and a `VersioningStrategy` (URI-based, header-based, or media-type-based). Routes at different API versions must be allowed to share the same logical path because the version disambiguates them at request time.

For URI-based versioning, the version prefix (`/v1`, `/v2`) is *not* embedded in the compiled route's `path` field. Instead, it's stored in the metadata and applied dynamically via `CompiledRoute::versioned_path()` at line 506:

```rust
pub fn versioned_path(&self) -> String {
    self.version()
        .filter(|v| v.strategy == crate::VersioningStrategy::Uri)
        .map_or_else(|| self.path.clone(), |v| format!("{}{}", v.uri_prefix(), self.path))
}
```

This means two controllers at versions `v1` and `v2` both produce compiled routes with path `"/users"` and method `GET`, but with different version metadata. The dedup key `(GET, "/users", Some(("1", Uri)))` differs from `(GET, "/users", Some(("2", Uri)))`, so both are accepted. At dispatch time, `versioned_path()` prepends `/v1` or `/v2` as appropriate.

### How conflicts are reported

When a conflict is found, the error is `RouteError::DuplicateRoute { method, path }`. The `Debug` implementation on `DuplicateRoute` includes both the HTTP method and the normalized path. Combined with the controller name (available from the `CompiledRoute`'s `controller` field in debug contexts), the error message is immediately actionable: you know *which two controllers* defined `GET /users`, and you can remove or rename one.

## A concrete example

Consider two controllers:

```rust
#[controller("/admin")]
struct AdminController { /* GET /users */ }

#[controller("/")]
struct PublicController { /* GET /users */ }
```

**Phase one** compiles each individually. `AdminController` produces `GET /admin/users`. `PublicController` produces `GET /users`. No intra-controller duplicates exist.

**Phase two** merges them. The `seen` set receives `(GET, "/admin/users", None)` and `(GET, "/users", None)`. Distinct paths — no conflict.

Now consider:

```rust
#[controller("/api")]
struct UserControllerV1 { /* GET /users */ }

#[controller("/api")]
struct UserControllerV2 { /* GET /users */ }
```

Both compile to `GET /api/users` with no version metadata. `seen.insert()` returns `false` on the second insert, and the framework returns `RouteError::DuplicateRoute`. The application fails to start with a clear error.

If the same controllers are versioned — `UserControllerV1` carries `VersionMetadata::new("1", VersioningStrategy::Uri)` and `UserControllerV2` carries version `"2"` — the dedup keys differ and both routes are accepted. At dispatch time, they resolve to `/v1/api/users` and `/v2/api/users` respectively.

This two-phase design — normalize and deduplicate per-controller, then merge and conflict-detect across controllers — catches every route collision before a single request is served. The normalization pass guarantees that sloppy paths don't create silent duplicates, and the version-aware dedup key ensures that intentional API versioning isn't mistaken for a conflict.
