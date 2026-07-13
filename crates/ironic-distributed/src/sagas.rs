//! Ordered saga execution with reverse compensation.

use std::{future::Future, pin::Pin, sync::Arc};

/// Boxed saga step operation.
pub type SagaFuture<'a> = Pin<Box<dyn Future<Output = Result<(), SagaError>> + Send + 'a>>;

/// A saga execution or compensation failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("IRONIC_SAGA_{stage}: step `{step}`: {message}")]
pub struct SagaError {
    stage: &'static str,
    step: &'static str,
    message: String,
}

impl SagaError {
    /// Creates an execution failure for `step`.
    #[must_use]
    pub fn execute(step: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage: "EXECUTE",
            step,
            message: message.into(),
        }
    }
    /// Creates a compensation failure for `step`.
    #[must_use]
    pub fn compensate(step: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage: "COMPENSATE",
            step,
            message: message.into(),
        }
    }
}

/// One forward operation and its compensating operation.
pub trait SagaStep<S>: Send + Sync + 'static {
    /// Stable step name used in diagnostics.
    fn name(&self) -> &'static str;
    /// Applies the forward operation.
    fn execute<'a>(&'a self, state: &'a mut S) -> SagaFuture<'a>;
    /// Reverses a previously completed operation.
    fn compensate<'a>(&'a self, state: &'a mut S) -> SagaFuture<'a>;
}

/// A deterministic ordered saga.
#[derive(Default)]
pub struct Saga<S> {
    steps: Vec<Arc<dyn SagaStep<S>>>,
}

impl<S: Send + 'static> Saga<S> {
    /// Creates an empty saga.
    #[must_use]
    pub const fn new() -> Self {
        Self { steps: Vec::new() }
    }
    /// Appends a step.
    #[must_use]
    pub fn step(mut self, step: impl SagaStep<S>) -> Self {
        self.steps.push(Arc::new(step));
        self
    }

    /// Executes all steps, compensating completed steps in reverse after a failure.
    ///
    /// # Errors
    /// Returns the execution failure unless compensation fails, in which case the compensation
    /// failure is returned because manual recovery is required.
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
}
