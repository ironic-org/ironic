---
title: "What #[derive(Injectable)] actually generates — a line-by-line breakdown"
description: "A deep dive into Ironic's Injectible proc macro: what it reads from your struct, how it maps attributes to ProviderDefinition fields, and the Rust it silently writes for you."
date: "2026-07-15"
author: "Ironic Team"
---

# What `#[derive(Injectable)]` actually generates — a line-by-line breakdown

Every Rust DI framework needs to answer one question: how do you go from *"here's a struct with some fields"* to *"here's the registration metadata and the factory that builds you one"*? In Ironic, the answer is a 180-line proc macro. No runtime reflection, no `HashMap<String, Box<dyn Any>>`, no decorators. Just a derive that reads your type and writes straight-line Rust back at you.

Let's walk through exactly what it writes.

---

## What you write

You type a normal Rust struct with `Arc<T>` fields and some attribute annotations:

```rust
#[derive(Injectable)]
#[injectable(scope = "request", optional = [CacheService])]
struct UserService {
    db: Arc<Pool>,
    cache: Arc<CacheService>,
}
```

That's it. Three things declared:
- **scope = "request"** — this service lives for one HTTP request, not the lifetime of the app
- **optional = [CacheService]** — `cache` depends on something that might not be registered; if it's missing, that's fine
- **two Arc<T> fields** — each is a dependency the container needs to supply

Now let's trace what the macro does with each piece.

---

## Step 1: reading the struct's attributes

The macro starts at `injectable.rs:24-58`. It scans `#[injectable(...)]` attributes using `parse_nested_meta`, a streaming parser that matches named arguments:

```
eager?              → sets a bool, adds `.eager()` later
scope = "request"?  → matches the string literal, emits Scope::Request
optional = [...]?   → parses the bracketed type list into a HashSet<String>
```

The scope string is compared literally (`injectable.rs:36-39`):

```rust
"singleton" → ::ironic::Scope::Singleton
"transient" → ::ironic::Scope::Transient
"request"   → ::ironic::Scope::Request
```

`OptionalTypes` at line 154-179 is a custom `syn::parse` implementation. It expects `[Type, Type, ...]`, parses each entry as a Rust type (`syn::Type`), then stores them as strings via `quote!(#ty).to_string()`. The string comparison against struct fields happens next.

---

## Step 2: walking struct fields

The macro matches `input.data` against `Data::Struct` with `Fields::Named` (`injectable.rs:62-101`). For each named field, it calls `arc_inner()`.

`arc_inner` (line 119-151) is a narrow type checker. It walks the type path, checks that the **last path segment** is literally `Arc`, and extracts the single generic argument from `Arc<T>`. Everything else — `Box<T>`, `Arc<T, U>`, bare `String`, `Option<Arc<T>>` — gets the same error: *"injectable fields must have type `Arc<T>`".* This is by design: if you want a dependency, you spell it `Arc<Something>`. No ambiguity.

For our example:

| Field | `arc_inner` result | Inner type string |
|-------|--------------------|-------------------|
| `db: Arc<Pool>` | `Pool` | `"Pool"` |
| `cache: Arc<CacheService>` | `CacheService` | `"CacheService"` |

`"Pool"` is not in `optional_types`, so it gets `Dependency::required::<Pool>()`. `"CacheService"` IS in `optional_types` (from the attribute), so it gets `Dependency::optional::<CacheService>()`.

---

## Step 3: collecting dependencies

For each field, the macro pushes a token stream into a `dependencies` vec:

```rust
// Required field:
quote!(::ironic::Dependency::required::<#inner_type>())

// Optional-marked field:
quote!(::ironic::Dependency::optional::<#inner_type>())
```

These expand to calls that construct the `Dependency` struct you saw in `lib.rs:74-77`:

```rust
pub struct Dependency {
    key: ProviderKey,  // TypeId + type name string
    optional: bool,    // false for required, true for optional
}
```

`Dependency::required::<T>()` sets `optional: false`. `Dependency::optional::<T>()` sets `optional: true`. Both store `ProviderKey::of::<T>()` — a `(TypeId, "fully::qualified::Type")` pair — as the key.

This metadata tells the container what it needs to know *before* calling any factory: "you'll need a `Pool` (non-negotiable) and a `CacheService` (nice to have)."

---

## Step 4: generating the factory body

The real work happens in the factory. For every field, the macro writes a field initializer that calls back into `resolver`:

```rust
// Required:
field_name: resolver.resolve::<T>().await?

// Optional:
field_name: ::std::option::Option::Some(
    resolver.resolve_optional::<T>().await?
)
```

These get spliced together into the struct literal via `quote!(Self { #(#initializers),* })`.

Here's the critical difference:

- `resolver.resolve::<T>()` returns `Result<Arc<T>, ResolveError>` — if `T` isn't registered, you get a `MissingProvider` error. The `?` propagates it.
- `resolver.resolve_optional::<T>()` returns `Result<Option<Arc<T>>, ResolveError>` — if `T` isn't registered, you get `Ok(None)` instead of an error (`lib.rs:584-591`). The `Some(...)` wraps it so the field gets `Some(Some(Arc<T>))` if present, `Some(None)` if missing.

The `Resolver` struct (line 555-560) is the clonable handle passed to every factory. It holds a reference to the full container plus the current resolution path for cycle detection. Your factory calls `resolver.resolve::<Dep>()`, and the container's `OnceCell` cache handles lazy initialization, scope enforcement, and deduplication.

---

## Step 5: assembling the full `ProviderDefinition`

Everything comes together in the output at `injectable.rs:104-116`:

```rust
Ok(quote! {
    impl #name {
        pub fn provider_definition() -> ::ironic::ProviderDefinition {
            ::ironic::ProviderDefinition::factory(
                #scope,
                ::std::vec![#(#dependencies),*],
                |resolver| async move { ::std::result::Result::Ok(#initializers) },
            )
            #eager_call
        }
    }
})
```

`ProviderDefinition::factory` (`lib.rs:136-158`) takes the scope, the dependencies vec, and a closure, then erases the factory into `Arc<dyn Fn(Resolver) -> Pin<Box<dyn Future<...>>> + Send + Sync>`. The closure captures nothing from outside — it only needs `resolver`, which is injected by the container at call time. If `eager` was set, `.eager()` is chained (`lib.rs:204-207`), flipping the `eager: true` flag so the container bootstraps this provider during `build()`.

There is no trait involved. The macro generates an inherent `pub fn provider_definition()` directly on your struct. No `impl Injectable for UserService`. No virtual dispatch. Just a plain function that returns a value.

---

## Complete before / after

Here's what the user writes and what lands in the compiled binary:

```rust
// —— YOU WRITE ——
#[derive(Injectable)]
#[injectable(scope = "request", optional = [CacheService])]
struct UserService {
    db: Arc<Pool>,
    cache: Arc<CacheService>,
}

// —— THE MACRO GENERATES ——
impl UserService {
    pub fn provider_definition() -> ::ironic::ProviderDefinition {
        ::ironic::ProviderDefinition::factory(
            ::ironic::Scope::Request,
            ::std::vec![
                ::ironic::Dependency::required::<Pool>(),
                ::ironic::Dependency::optional::<CacheService>(),
            ],
            |resolver| async move {
                ::std::result::Result::Ok(Self {
                    db: resolver.resolve::<Pool>().await?,
                    cache: ::std::option::Option::Some(
                        resolver.resolve_optional::<CacheService>().await?,
                    ),
                })
            },
        )
    }
}
```

With `eager` added:

```rust
// Additional attribute:
#[injectable(eager)]

// Adds this chain:
// ...factory(...)
// .eager()
```

---

## What this means

The macro doesn't do anything magical. It reads your struct's fields and attributes, maps them to `Dependency` metadata and `resolver` calls, and wraps the whole thing in `ProviderDefinition::factory()`. The output is a single function that returns a data structure. That data structure gets fed into `ContainerBuilder::register()`, which stores it in a `HashMap<ProviderKey, ProviderDefinition>`. The container's resolve logic (`lib.rs:594-655`) uses the scope, the dependencies, and the factory to construct your service on demand, enforcing cycle detection and scope isolation along the way.

No reflection. No global mutable state. No runtime type erasure beyond what's strictly necessary. Just a proc macro writing the boilerplate you'd otherwise write by hand.

That's the whole trick.
