---
title: "Sagas — ordered steps with reverse compensation and error priority"
description: "How Ironic implements deterministic sagas with typed shared state, reverse-ordered compensation, and a controversial error priority rule that elevates compensation failures above original execution errors."
date: "2026-07-15"
author: "Ironic Team"
---

# Sagas — ordered steps with reverse compensation and error priority

Distributed sagas solve the atomicity problem that spans across services: when a workflow touches inventory, payments, and shipping, each step must be reversible. Ironic's saga implementation is deliberately simple — 83 lines of ordered-step traversal with reverse compensation — but encodes a design decision about error priority that is worth understanding before you adopt it.

## The saga pattern in miniature

A saga is a sequence of steps. Each step has a forward action and a compensating action. If all forward actions succeed, the saga completes. If step N fails, steps N-1 down to 0 execute their compensating actions in reverse order, undoing the partial work.

This is the standard saga pattern, first described in Garcia-Molina and Salem's 1987 paper. Ironic's implementation doesn't innovate on the pattern itself — it innovates on the interface, making it composable in Rust without distributed transaction coordinators or message brokers.

## The type-erased step collection

The `Saga<S>` struct holds a single vector:

```rust
pub struct Saga<S> {
    steps: Vec<Arc<dyn SagaStep<S>>>,
}
```

Every step is `Arc<dyn SagaStep<S>>` — a type-erased trait object parameterized only by the shared state type `S`. This means a saga for `OrderSagaState` can contain an `InventoryStep`, a `PaymentStep`, and a `ShipmentStep`, all stored in the same `Vec` despite being distinct concrete types. The `Arc` enables cheap cloning of the step list and ensures steps can be shared across multiple saga instances.

Steps are appended via a builder-style `step()` method that takes ownership and returns `Self`:

```rust
pub fn step(mut self, step: impl SagaStep<S>) -> Self {
    self.steps.push(Arc::new(step));
    self
}
```

Because the return type is `Self` (not `&mut Self`), the builder consumes itself and the step list is immutable after construction. There is no way to add or remove steps from a constructed `Saga<S>` — a guarantee that matters when sagas are shared across request handlers.

## The `SagaStep` trait

Each step implements three methods:

```rust
pub trait SagaStep<S>: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn execute<'a>(&'a self, state: &'a mut S) -> SagaFuture<'a>;
    fn compensate<'a>(&'a self, state: &'a mut S) -> SagaFuture<'a>;
}
```

The `name()` method provides a stable identifier for error reporting. The `execute()` and `compensate()` methods both receive `&'a mut S` — a mutable reference to the shared state — and return `SagaFuture<'a>`, an alias for `Pin<Box<dyn Future<Output = Result<(), SagaError>> + Send + 'a>>`. The lifetime `'a` ties the future's borrow of `state` to the duration of the async operation.

Note that both methods operate on the same shared state `S`. There is no per-step local state. If `InventoryStep::execute()` reserves a quantity and stashes the reservation ID in `state.reservation_id`, then `InventoryStep::compensate()` reads that same field to release the reservation. This pattern — mutable shared state threaded through all steps — is the saga's primary design constraint and its primary footgun. Steps must agree on which fields they read and write, and the state type must be constructed before the saga starts.

## Sequential execution with reverse rollback

The `execute()` method on `Saga<S>` is a single loop:

```rust
pub async fn execute(&self, state: &mut S) -> Result<(), SagaError> {
    for (index, step) in self.steps.iter().enumerate() {
        if let Err(error) = step.execute(state).await {
            for completed in self.steps[..index].iter().rev() {
                completed.compensate(state).await?;
            }
            return Err(error);
        }
    }
    Ok(())
}
```

The logic is straightforward but worth stepping through:

1. Iterate steps [0..N] in order, calling `step.execute(state)` for each.
2. If step K succeeds, advance to step K+1.
3. If step K fails, enter the compensation loop: steps [0..K-1] are iterated in reverse.
4. Each compensating call returns `Result<(), SagaError>`. If compensation succeeds, the loop continues to the next step backward. If any compensation fails, the entire saga returns that compensation error immediately — abandoning any remaining compensations.

Step 4 is the design decision that needs scrutiny. If step 3 (`ChargePayment`) fails, and then step 1's (`ReserveInventory`) compensation also fails, the saga returns the `ReserveInventory` compensation error. Step 0's compensation (if any) never runs. The rationale is stated in the doc comment: "because manual recovery is required."

## Error priority: why compensation wins

Consider the error types:

```rust
pub struct SagaError {
    stage: &'static str,
    step: &'static str,
    message: String,
}
```

A `SagaError` captures three things: the `stage` (either `"EXECUTE"` or `"COMPENSATE"`), the `step` name, and a freeform `message`. This is deliberately minimal — no backtrace, no chain of errors, no structured payload. The assumption is that saga failures are rare enough that an operator will examine logs by hand.

The priority rule is encoded in the `?` operator inside the compensation loop:

```rust
completed.compensate(state).await?;
```

If `compensate()` returns `Err(SagaError { stage: "COMPENSATE", ... })`, the `?` propagates it out of `execute()` immediately. The original execution error — the one that triggered the rollback — is discarded. The rationale: a failed compensation means the system is in an inconsistent state (e.g., inventory was reserved and not released), which is more dangerous than the original failure (e.g., payment was declined). The operator needs to know about the compensation failure to perform manual reconciliation.

This is a reasonable default but not universally correct. Some applications might prefer to return both errors, or to continue compensating remaining steps even after one compensation fails. Ironic's decision to abort on first compensation failure and report the compensation error is a deliberate simplicity choice — complex error aggregation is left to the application layer.

## A concrete example: e-commerce order saga

```rust
struct OrderSagaState {
    order_id: u64,
    user_id: u64,
    items: Vec<LineItem>,
    reservation_id: Option<String>,
    payment_id: Option<String>,
    shipment_id: Option<String>,
}

struct ReserveInventoryStep;
impl SagaStep<OrderSagaState> for ReserveInventoryStep {
    fn name(&self) -> &'static str { "ReserveInventory" }
    async fn execute(&self, state: &mut OrderSagaState) -> Result<(), SagaError> { /* ... */ }
    async fn compensate(&self, state: &mut OrderSagaState) -> Result<(), SagaError> { /* ... */ }
}

struct ChargePaymentStep;
impl SagaStep<OrderSagaState> for ChargePaymentStep { /* ... */ }

struct CreateShipmentStep;
impl SagaStep<OrderSagaState> for CreateShipmentStep { /* ... */ }

let saga = Saga::new()
    .step(ReserveInventoryStep)
    .step(ChargePaymentStep)
    .step(CreateShipmentStep);

let mut state = OrderSagaState { order_id: 1, user_id: 42, items: vec![...], ... };
saga.execute(&mut state).await?;
```

If `ChargePaymentStep::execute()` fails (e.g., insufficient funds), the saga compensates `ReserveInventoryStep` — releasing the held inventory. The `CreateShipmentStep` never runs, so it needs no compensation. If the inventory compensation also fails (e.g., the inventory service is down), the saga returns the compensation error with `stage: "COMPENSATE"` and `step: "ReserveInventory"`, and the operator knows manual reconciliation is required.

## Contrast with NestJS sagas

NestJS's `@nestjs/cqrs` package models sagas as `ISaga` classes with `@Saga()` decorated event handlers. A NestJS saga is a reactive orchestration: each method subscribes to an event, and the saga's state machine advances in response to event streams. This is fundamentally event-driven: publish `OrderPlaced`, wait for `PaymentProcessed`, then publish `ShipmentCreated`. Compensation, if implemented, must be wired manually through additional event handlers.

Ironic's sagas are deterministic and sequential. There is no event bus involved, no pub/sub, no async streams, and no state machine. You define an ordered list of steps with shared mutable state, and the framework walks through them forward or backward. This simplicity trades flexibility for predictability: you cannot express a saga where step C depends on the outcome of step A and step B running in parallel, but you also cannot express a saga with dangling compensation holes.

The choice between the two models depends on the domain. For a linear checkout flow — reserve inventory, charge payment, create shipment — Ironic's ordered approach is shorter, faster to write, and easier to reason about during incident response. For complex multi-day workflows with conditional branches and timeouts, the event-driven model is more appropriate. Ironic deliberately stays in the simpler lane.
