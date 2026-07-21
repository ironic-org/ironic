---
title: Saga Orchestration
description: Ordered saga execution with automatic reverse compensation — implement distributed transactions safely.
---

# Saga Orchestration

## Enabling

```toml
ironic = { features = ["sagas"] }
```

## What is a saga?

A saga executes a sequence of steps. If any step fails, all previously completed steps are *compensated* in reverse order — rolling back the operation without requiring distributed transactions.

## Defining a saga step

Each step implements `SagaStep<S>` with a forward (`execute`) and backward (`compensate`) operation:

```rust
use ironic::distributed::saga::{Saga, SagaStep, SagaError};

struct CreateOrder;

impl SagaStep<OrderState> for CreateOrder {
    fn name(&self) -> &'static str {
        "create_order"
    }

    fn execute<'a>(&'a self, state: &'a mut OrderState) -> SagaFuture<'a> {
        Box::pin(async move {
            state.order_id = Some(insert_order(&state.cart).await?);
            Ok(())
        })
    }

    fn compensate<'a>(&'a self, state: &'a mut OrderState) -> SagaFuture<'a> {
        Box::pin(async move {
            if let Some(id) = state.order_id {
                cancel_order(id).await?;
            }
            Ok(())
        })
    }
}
```

## Running a saga

```rust
use ironic::distributed::saga::Saga;

struct OrderState {
    cart: Vec<Item>,
    order_id: Option<i64>,
    payment_id: Option<i64>,
    inventory_held: bool,
}

let saga = Saga::new()
    .step(CreateOrder)
    .step(ReserveInventory)
    .step(ProcessPayment);

let mut state = OrderState {
    cart: vec![item],
    order_id: None,
    payment_id: None,
    inventory_held: false,
};

match saga.execute(&mut state).await {
    Ok(()) => println!("Order complete"),
    Err(error) => println!("Saga failed at step {}: {error}", error.step()),
}
```

If `ProcessPayment` fails, the saga automatically calls `ReserveInventory.compensate()` then `CreateOrder.compensate()` in reverse order.

## Feature flags

| Flag | Enables |
|------|---------|
| `sagas` | `Saga<S>`, `SagaStep<S>` trait, `SagaError` |

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Compensation panics | Ensure `compensate()` is idempotent — it may be called multiple times |
| State shared across steps | Use a single mutable state struct; each step reads/writes the fields it owns |
