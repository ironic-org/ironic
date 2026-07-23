//! Type-safe command and query dispatch.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
};

type ErasedValue = Box<dyn Any + Send>;
type HandlerFuture = Pin<Box<dyn Future<Output = Result<ErasedValue, CqrsError>> + Send>>;
type Handler = dyn Fn(ErasedValue) -> HandlerFuture + Send + Sync;

/// A command with a declared result type.
pub trait Command: Send + 'static {
    /// Result produced by the command handler.
    type Output: Send + 'static;
}

/// A query with a declared result type.
pub trait Query: Send + 'static {
    /// Result produced by the query handler.
    type Output: Send + 'static;
}

/// A CQRS registration or dispatch failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum CqrsError {
    /// No handler is registered for the message type.
    #[error("IRONIC_CQRS_MISSING_HANDLER: {0}")]
    MissingHandler(&'static str),
    /// A handler was registered more than once.
    #[error("IRONIC_CQRS_DUPLICATE_HANDLER: {0}")]
    DuplicateHandler(&'static str),
    /// The erased handler contract was violated.
    #[error("IRONIC_CQRS_TYPE_MISMATCH: {0}")]
    TypeMismatch(&'static str),
    /// Application handler execution failed.
    #[error("IRONIC_CQRS_HANDLER_FAILED: {0}")]
    Handler(String),
}

/// Immutable command/query dispatcher built from typed handlers.
#[derive(Clone, Default)]
pub struct CqrsBus {
    commands: Arc<HashMap<TypeId, Arc<Handler>>>,
    queries: Arc<HashMap<TypeId, Arc<Handler>>>,
}

/// Builds a [`CqrsBus`].
#[derive(Default)]
pub struct CqrsBusBuilder {
    commands: HashMap<TypeId, Arc<Handler>>,
    queries: HashMap<TypeId, Arc<Handler>>,
}

impl CqrsBusBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one asynchronous command handler.
    ///
    /// # Errors
    /// Returns [`CqrsError::DuplicateHandler`] if `C` already has a handler.
    pub fn command<C, F, Fut>(&mut self, handler: F) -> Result<&mut Self, CqrsError>
    where
        C: Command,
        F: Fn(C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<C::Output, CqrsError>> + Send + 'static,
    {
        register::<C, C::Output, F, Fut>(&mut self.commands, handler)?;
        Ok(self)
    }

    /// Registers one asynchronous query handler.
    ///
    /// # Errors
    /// Returns [`CqrsError::DuplicateHandler`] if `Q` already has a handler.
    pub fn query<Q, F, Fut>(&mut self, handler: F) -> Result<&mut Self, CqrsError>
    where
        Q: Query,
        F: Fn(Q) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Q::Output, CqrsError>> + Send + 'static,
    {
        register::<Q, Q::Output, F, Fut>(&mut self.queries, handler)?;
        Ok(self)
    }

    /// Completes the immutable dispatcher.
    #[must_use]
    pub fn build(self) -> CqrsBus {
        CqrsBus {
            commands: Arc::new(self.commands),
            queries: Arc::new(self.queries),
        }
    }
}

impl CqrsBus {
    /// Executes a command.
    ///
    /// # Errors
    /// Returns an error for missing handlers, type contract violations, or handler failure.
    pub async fn execute<C: Command>(&self, command: C) -> Result<C::Output, CqrsError> {
        dispatch::<C, C::Output>(&self.commands, command).await
    }

    /// Executes a query.
    ///
    /// # Errors
    /// Returns an error for missing handlers, type contract violations, or handler failure.
    pub async fn ask<Q: Query>(&self, query: Q) -> Result<Q::Output, CqrsError> {
        dispatch::<Q, Q::Output>(&self.queries, query).await
    }
}

fn register<I, O, F, Fut>(
    handlers: &mut HashMap<TypeId, Arc<Handler>>,
    handler: F,
) -> Result<(), CqrsError>
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<O, CqrsError>> + Send + 'static,
{
    let id = TypeId::of::<I>();
    if handlers.contains_key(&id) {
        return Err(CqrsError::DuplicateHandler(std::any::type_name::<I>()));
    }
    handlers.insert(
        id,
        Arc::new(move |input| {
            let input = input
                .downcast::<I>()
                .map_err(|_| CqrsError::TypeMismatch(std::any::type_name::<I>()));
            match input {
                Ok(input) => {
                    let future = handler(*input);
                    Box::pin(
                        async move { future.await.map(|output| Box::new(output) as ErasedValue) },
                    )
                }
                Err(error) => Box::pin(async move { Err(error) }),
            }
        }),
    );
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    struct CreateUser {
        name: String,
    }
    impl Command for CreateUser {
        type Output = u64;
    }

    struct GetUser {
        id: u64,
    }
    impl Query for GetUser {
        type Output = String;
    }

    #[tokio::test]
    async fn command_handler_executes_successfully() {
        let mut builder = CqrsBusBuilder::new();
        builder
            .command(|cmd: CreateUser| async move {
                assert_eq!(cmd.name, "Alice");
                Ok(42u64)
            })
            .unwrap();
        let bus = builder.build();

        let result = bus
            .execute(CreateUser {
                name: "Alice".into(),
            })
            .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn query_handler_returns_result() {
        let mut builder = CqrsBusBuilder::new();
        builder
            .query(|query: GetUser| async move {
                assert_eq!(query.id, 1);
                Ok("Alice".into())
            })
            .unwrap();
        let bus = builder.build();

        let result = bus.ask(GetUser { id: 1 }).await;
        assert_eq!(result.unwrap(), "Alice");
    }

    #[tokio::test]
    async fn missing_command_handler_returns_error() {
        let bus = CqrsBusBuilder::new().build();
        let result = bus.execute(CreateUser { name: "Bob".into() }).await;
        assert!(matches!(result, Err(CqrsError::MissingHandler(_))));
    }

    #[tokio::test]
    async fn missing_query_handler_returns_error() {
        let bus = CqrsBusBuilder::new().build();
        let result = bus.ask(GetUser { id: 99 }).await;
        assert!(matches!(result, Err(CqrsError::MissingHandler(_))));
    }

    #[test]
    fn duplicate_handler_returns_error() {
        let mut builder = CqrsBusBuilder::new();
        builder
            .command(|_: CreateUser| async move { Ok(1u64) })
            .unwrap();
        let result = builder.command(|_: CreateUser| async move { Ok(2u64) });
        assert!(matches!(result, Err(CqrsError::DuplicateHandler(_))));
    }

    #[test]
    fn cqrs_error_display() {
        let err = CqrsError::MissingHandler("CreateUser");
        assert!(err.to_string().contains("IRONIC_CQRS_MISSING_HANDLER"));

        let err = CqrsError::DuplicateHandler("GetUser");
        assert!(err.to_string().contains("IRONIC_CQRS_DUPLICATE_HANDLER"));

        let err = CqrsError::TypeMismatch("u64");
        assert!(err.to_string().contains("IRONIC_CQRS_TYPE_MISMATCH"));

        let err = CqrsError::Handler("something went wrong".into());
        assert!(err.to_string().contains("IRONIC_CQRS_HANDLER_FAILED"));
    }

    #[test]
    fn cqrs_bus_builder_default_is_empty() {
        let builder = CqrsBusBuilder::new();
        let bus = builder.build();
        assert!(bus.commands.is_empty());
        assert!(bus.queries.is_empty());
    }
}
