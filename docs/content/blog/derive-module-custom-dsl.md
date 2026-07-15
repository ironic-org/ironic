---
title: "#[derive(Module)] — parsing a custom DSL in proc macro attributes"
description: "How Ironic's #[derive(Module)] macro parses a declarative attribute DSL, validates it at compile time, and generates boilerplate-free module definitions."
date: "2026-07-15"
author: "Ironic Team"
---

# #[derive(Module)] — parsing a custom DSL in proc macro attributes

Rust proc macros usually accept configuration through simple key-value pairs or raw token trees. Ironic's `#[derive(Module)]` takes a different approach: it defines a custom mini-language in proc macro attributes that reads like a configuration manifest. The entire implementation lives in 100 lines at `crates/ironic-macros/src/module.rs`.

## What the user writes

A typical Ironic module declaration looks like this:

```rust
#[global]
#[module(
    imports = [DbPool, RedisClient],
    providers = [UserService],
    controllers = [UserController],
    exports = [UserService]
)]
struct AppModule;
```

Four named lists, each containing Rust type identifiers. The `#[global]` annotation is a separate, orthogonal attribute. The syntax is deliberately concise: no quotes, no nested structures, just names, equals signs, and bracketed type lists. A module might declare any or all of the four keys — the macro handles every combination gracefully because `ModuleArgs` derives `Default`.

## The parser: `ModuleArgs`

The heart of the parser is the `impl Parse for ModuleArgs` block (`module.rs:15-46`). It operates as a hand-rolled recursive descent parser over `syn`'s token stream:

```rust
impl Parse for ModuleArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let content;
            bracketed!(content in input);
            let values = content
                .parse_terminated(Type::parse, Token![,])?
                .into_iter()
                .collect();
            match key.to_string().as_str() {
                "imports" => args.imports = values,
                "providers" => args.providers = values,
                "controllers" => args.controllers = values,
                "exports" => args.exports = values,
                _ => return Err(syn::Error::new_spanned(key, "expected ...")),
            }
            if input.is_empty() { break; }
            input.parse::<Token![,]>()?;
        }
        Ok(args)
    }
}
```

The parsing strategy is systematic:

1. **Lookahead on the identifier.** `input.parse::<Ident>()` reads the next token. If it's `imports`, `providers`, `controllers`, or `exports`, the parser proceeds. If it's anything else, a `syn::Error` with a helpful span points directly at the offending key.

2. **Expect the `=` token.** `input.parse::<Token![=]>()` consumes the assignment operator. If the user writes `imports : [...]` or `imports [...]`, this fails with a clear error at the `=` position.

3. **Parse a bracketed type list.** `bracketed!(content in input)` captures the content between `[` and `]` into a new parse stream. Then `content.parse_terminated(Type::parse, Token![,])` parses a `Punctuated<Type, Comma>` — each element is a Rust type, separated by commas. Trailing commas are allowed.

4. **Consume the comma separator between key-value pairs.** After each group, if more tokens remain, the parser consumes a comma before the next key. The `break` on `input.is_empty()` handles the final group without a trailing comma.

The parser delegates all type parsing to `syn::Type::parse`, which means users can write any valid Rust type path: `DbPool`, `my_crate::services::UserService`, or even generic types (though the module system itself rejects generics — checked separately at line 50-55).

## Detecting `#[global]`

The `#[global]` annotation is handled orthogonally to the `#[module(...)]` DSL. The `expand` function checks with a simple attribute scan (`module.rs:57-60`):

```rust
let has_global = input
    .attrs
    .iter()
    .any(|attr| attr.path().is_ident("global"));
```

It's a separate annotation mechanism — not a key inside the module DSL — because it modifies the module's *registration strategy* (eager loading, global visibility) rather than its composition. This separation keeps the DSL focused on wiring and avoids boolean flags inside the structured argument list.

A validation step ensures exactly one `#[module(...)]` attribute is present (`module.rs:62-72`). Zero attributes or multiple attributes both produce compile errors with span information pointing at the struct name.

## Code generation: chaining builder calls

The code generation block (`module.rs:88-99`) emits a trait implementation:

```rust
impl ::ironic::Module for AppModule {
    fn definition() -> ::ironic::ModuleDefinition {
        ::ironic::ModuleDefinition::builder::<Self>()
            .import::<DbPool>()
            .import::<RedisClient>()
            .provider(<UserService>::provider_definition())
            .controller(<UserController>::controller_definition())
            .export::<UserService>()
            .global()
            .build()
    }
}
```

Each parsed type list becomes a chain of method calls. `imports` and `exports` use turbofish syntax (`.import::<Type>()`), while `providers` and `controllers` invoke static methods (`.provider(<Type>::provider_definition())`) that return the corresponding definition objects. The `#[global]` flag emits `.global()` conditionally via `has_global.then(|| quote!(.global()))`.

The generated code calls `.build()` at the end, which performs module-level validation — checking for duplicate keys, verifying import-to-export consistency, and building the internal graph representation.

## Why a custom DSL and not separate attributes?

The alternative design would be separate `#[import]`, `#[provider]`, `#[controller]`, and `#[export]` attributes on the module struct:

```rust
#[import(DbPool)]
#[import(RedisClient)]
#[provider(UserService)]
// ...
struct AppModule;
```

Ironic chose the grouped DSL for three reasons:

1. **Conciseness.** A single `#[module(...)]` attribute groups related configuration. Module definitions typically involve 4-8 types; four attribute groups with one line each is more readable than 4-8 separate `#[import]` annotations.

2. **Discoverability.** The parser rejects unknown keys with a message listing the valid options (`"expected imports, providers, controllers, or exports"`). A user who writes `services = [...]` gets an immediate, precise compile error.

3. **Compile-time validation.** The custom `Parse` implementation validates structure before code generation. Malformed brackets, missing equals signs, and invalid type syntax fail during macro expansion with span-level errors — not as cryptic code generation failures downstream.

## Before and after

Without the derive macro, a module definition requires a manual trait implementation:

```rust
struct AppModule;
impl Module for AppModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DbPool>()
            .import::<RedisClient>()
            .provider(UserService::provider_definition())
            .controller(UserController::controller_definition())
            .export::<UserService>()
            .build()
    }
}
```

With the derive macro, the same semantics compress to a declarative 5-line block. The macro is transparent — it produces the same trait implementation, with the same types and the same builder API. Tools like `cargo expand` can show the generated code verbatim. The DSL is a compile-time convenience that desugars to the exact builder chain a user would write by hand.
