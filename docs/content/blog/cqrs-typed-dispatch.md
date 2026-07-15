---
title: "CQRS Bus — TypeId-based typed command and query dispatch"
description: "Deep dive into Ironic's zero-reflection CQRS bus: how TypeId-indexed handler maps, type erasure, and double-downcast guardrails provide compile-time safe command/query dispatch without decorators."
date: "2026-07-15"
author: "Ironic Team"
---

# CQRS Bus — TypeId-based typed command and query dispatch

Rust CQRS libraries face an awkward tension: you want strongly typed commands with distinct `Output` types, but you also need to store them in a single dispatcher so controllers can fire arbitrary commands without knowing which handler runs them. NestJS solves this with decorators and runtime metadata; Ironic solves it with generics, type erasure, and Rust's `TypeId`. The result is a bus that rejects duplicates at registration time and catches type mismatches at dispatch time — all without a single heap-allocated `dyn Reflect`.

## The trait foundation: `Command` and `Query`

The entire system rests on two traits so minimal they fit on a single line each:

```rust
pub trait Command: Send + 'static {
    type Output: Send + 'static;
}

pub trait Query: Send + 'static {
    type Output: Send + 'static;
}
```

These are marker traits with an associated output type. A `CreateOrderCommand` declares its handler will produce `OrderCreated`. A `GetUserQuery` declares it returns `User`. The traits impose no behavior — they exist purely to couple a message type to its result type at compile time. Every subsequent layer of the bus reads this association through generic bounds.

## The type-erased handler map

The heart of the bus is a single `HashMap`:

```rust
type ErasedValue = Box<dyn Any + Send>;
type HandlerFuture = Pin<Box<dyn Future<Output = Result<ErasedValue, CqrsError>> + Send>>;
type Handler = dyn Fn(ErasedValue) -> HandlerFuture + Send + Sync;

commands: HashMap<TypeId, Arc<Handler>>
queries: HashMap<TypeId, Arc<Handler>>
```

Commands and queries each get their own map, because a type that is both a `Command` and a `Query` should be dispatchable through either channel without collision. The key is `TypeId` — Rust's built-in compile-time type fingerprint from `std::any::TypeId`. The value is an `Arc<dyn Fn(Box<dyn Any + Send>) -> Pin<Box<dyn Future<...>>>>`: a fully type-erased async closure that takes an opaque box and returns an opaque future over an opaque result. The concrete types have been absorbed into the closure body and are invisible to the map itself.

## Registration: one handler per type

Registration happens inside the private `register()` function, which is called by `CqrsBusBuilder::command()` and `CqrsBusBuilder::query()`. The critical enforcement:

```rust
let id = TypeId::of::<I>();
if handlers.contains_key(&id) {
    return Err(CqrsError::DuplicateHandler(std::any::type_name::<I>()));
}
handlers.insert(id, Arc::new(move |input| { /* closure body */ }));
```

The builder uses `HashMap::entry`-style checking (via `contains_key` + `insert`) to guarantee at most one handler per message type. If a developer accidentally calls `.command(handle_create_order)` twice for `CreateOrderCommand`, the second call returns a `CqrsError::DuplicateHandler` at registration time — before any business logic runs. This is strictly better than the decorator-based approach where duplicate handler detection is a runtime surprise during application bootstrap.

Inside the closure, the registered handler does its first downcast:

```rust
let input = input
    .downcast::<I>()
    .map_err(|_| CqrsError::TypeMismatch(std::any::type_name::<I>()))?;
let future = handler(*input);
Box::pin(async move { future.await.map(|output| Box::new(output) as ErasedValue) })
```

The `input` arrives as `Box<dyn Any + Send>`. The closure downcasts it back to the concrete type `I`. If the downcast succeeds, the handler runs and its return value gets packed back into `Box<dyn Any + Send>`. If it fails, the closure returns `TypeMismatch` — which, assuming correct registration, is structurally impossible. This downcast exists as a safety net, not a normal codepath.

## Dispatch: double downcast

When a controller calls `bus.execute(my_command)`, the dispatch path mirrors the registration path in reverse:

```rust
async fn dispatch<I: Send + 'static, O: Send + 'static>(
    handlers: &HashMap<TypeId, Arc<Handler>>,
    input: I,
) -> Result<O, CqrsError> {
    let handler = handlers
        .get(&TypeId::of::<I>())
        .ok_or(CqrsError::MissingHandler(std::any::type_name::<I>()))?;
    handler(Box::new(input))
        .await?
        .downcast::<O>()
        .map(|value| *value)
        .map_err(|_| CqrsError::TypeMismatch(std::any::type_name::<O>()))
}
```

Three things happen in sequence:

1. **Handler lookup**: `TypeId::of::<I>()` retrieves the registered handler. If none exists, dispatch fails immediately with `MissingHandler`.

2. **Type-erased invocation**: The concrete input `I` is packed into `Box::new(input)` and passed through the erased closure. The closure downcasts, calls the real handler, and returns `Result<Box<dyn Any + Send>, CqrsError>`.

3. **Output downcast**: The returned opaque box is downcast to `O` via `.downcast::<O>()`. This is the second downcast — and again, it is structurally guaranteed to succeed because the handler closure was constructed with `I -> O` as its contract. If this downcast ever fails, the type system was violated somewhere between registration and dispatch.

Each of the three failure modes — `MissingHandler`, `TypeMismatch`, and `Handler` (for business logic errors) — carries the type name of the offending message, making diagnostics trivially greppable in production logs.

## Builder-vs-bus: the read-only split

The API splits into two structs by design:

- **`CqrsBusBuilder`** owns the mutable `HashMap<TypeId, Arc<Handler>>` and exposes `command()` and `query()` (which return `Result<&mut Self, CqrsError>` for chaining). After registration, `build()` consumes the builder and moves the maps into `Arc`.

- **`CqrsBus`** wraps both maps in `Arc<HashMap<...>>`, derives `Clone`, and exposes only `execute::<C>()` and `ask::<Q>()`. It cannot be mutated after construction.

This is the classical builder pattern applied to dependency injection: your startup code builds the bus with all registered handlers, then hands out cheaply cloneable `CqrsBus` handles to every controller. No controller can accidentally register a handler mid-flight, and the `Arc` ensures the handler maps are shared without deep copies.

## A concrete example

Consider an e-commerce module:

```rust
struct CreateOrderCommand { user_id: u64, items: Vec<LineItem> }
struct OrderCreated { order_id: u64, total: f64 }

impl Command for CreateOrderCommand { type Output = OrderCreated; }

async fn handle_create_order(cmd: CreateOrderCommand) -> Result<OrderCreated, CqrsError> {
    // validate, persist, emit event
    Ok(OrderCreated { order_id: 42, total: 99.97 })
}

let bus = CqrsBusBuilder::new()
    .command::<CreateOrderCommand, _, _>(handle_create_order)?
    .build();

// In any controller:
let result: OrderCreated = bus.execute(CreateOrderCommand { user_id: 1, items: vec![] }).await?;
```

The `bus.execute()` call is generic over `C: Command`, so the compiler infers both the input type and the output type from the call site. The `?` operator propagates `CqrsError` variants neatly into Ironic's exception filter pipeline.

## Contrast with NestJS CQRS

NestJS's `@nestjs/cqrs` package uses `@CommandHandler(CreateOrderCommand)` decorators on classes that implement `ICommandHandler`. The NestJS runtime discovers these decorators via TypeScript's `emitDecoratorMetadata` and `reflect-metadata`, assembling a handler registry at bootstrap. There is no `TypeId` equivalent because JavaScript's type system is structural, not nominal.

Ironic's approach is fundamentally different: there are no decorators, no class-based handlers, no runtime metadata, and no scanning of the module tree. A handler is just an `async fn` whose type signature `Fn(C) -> Fut` is sufficient for the compiler to wire everything together. The tradeoff is that registration is explicit — you must call `.command()` once per handler — but in return you get duplicate detection at registration time, zero-cost dispatch (no reflection, no decorator stack), and Rust's type checker verifying every connection between every message and every handler.
