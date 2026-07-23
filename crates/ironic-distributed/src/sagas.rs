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
    /// Each step is executed in insertion order. If a step fails, all previously
    /// completed steps are compensated in reverse order.
    ///
    /// # Errors
    ///
    /// Returns the execution failure unless compensation fails, in which case the compensation
    /// failure is returned because manual recovery is required.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ironic::distributed::sagas::{Saga, SagaStep, SagaFuture, SagaError};
    ///
    /// struct ReserveFunds;
    /// impl SagaStep<String> for ReserveFunds {
    ///     fn name(&self) -> &'static str { "reserve_funds" }
    ///     fn execute<'a>(&'a self, _state: &'a mut String) -> SagaFuture<'a> {
    ///         Box::pin(async { Ok(()) })
    ///     }
    ///     fn compensate<'a>(&'a self, _state: &'a mut String) -> SagaFuture<'a> {
    ///         Box::pin(async { Ok(()) })
    ///     }
    /// }
    ///
    /// let saga = Saga::new().step(ReserveFunds);
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    struct CollectStep {
        name: &'static str,
        _counter: Arc<AtomicU64>,
    }

    impl SagaStep<Vec<&'static str>> for CollectStep {
        fn name(&self) -> &'static str {
            self.name
        }
        fn execute<'a>(&'a self, state: &'a mut Vec<&'static str>) -> SagaFuture<'a> {
            Box::pin(async move {
                state.push(self.name);
                Ok(())
            })
        }
        fn compensate<'a>(&'a self, state: &'a mut Vec<&'static str>) -> SagaFuture<'a> {
            Box::pin(async move {
                state.push(self.name);
                Ok(())
            })
        }
    }

    struct FailingStep;

    impl SagaStep<Vec<&'static str>> for FailingStep {
        fn name(&self) -> &'static str {
            "fail"
        }
        fn execute<'a>(&'a self, _state: &'a mut Vec<&'static str>) -> SagaFuture<'a> {
            Box::pin(async { Err(SagaError::execute("fail", "oops")) })
        }
        fn compensate<'a>(&'a self, _state: &'a mut Vec<&'static str>) -> SagaFuture<'a> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn saga_executes_all_steps_in_order() {
        let saga = Saga::new()
            .step(CollectStep {
                name: "step1",
                _counter: Arc::new(AtomicU64::new(0)),
            })
            .step(CollectStep {
                name: "step2",
                _counter: Arc::new(AtomicU64::new(0)),
            });
        let mut state = Vec::new();
        saga.execute(&mut state).await.unwrap();
        assert_eq!(state, vec!["step1", "step2"]);
    }

    #[tokio::test]
    async fn saga_compensates_on_failure() {
        let saga = Saga::new()
            .step(CollectStep {
                name: "a",
                _counter: Arc::new(AtomicU64::new(0)),
            })
            .step(FailingStep)
            .step(CollectStep {
                name: "c",
                _counter: Arc::new(AtomicU64::new(0)),
            });
        let mut state = Vec::new();
        let result = saga.execute(&mut state).await;
        assert!(result.is_err());
        // Step "a" should have been compensated
        assert_eq!(state, vec!["a", "a"]);
    }

    #[tokio::test]
    async fn empty_saga_succeeds() {
        let saga: Saga<()> = Saga::new();
        let mut state = ();
        saga.execute(&mut state).await.unwrap();
    }

    #[tokio::test]
    async fn saga_single_step_success() {
        let saga = Saga::new().step(CollectStep {
            name: "only",
            _counter: Arc::new(AtomicU64::new(0)),
        });
        let mut state = Vec::new();
        saga.execute(&mut state).await.unwrap();
        assert_eq!(state, vec!["only"]);
    }

    #[test]
    fn saga_error_execute_creation() {
        let err = SagaError::execute("step1", "something failed");
        assert!(err.to_string().contains("IRONIC_SAGA_EXECUTE"));
        assert!(err.to_string().contains("step1"));
    }

    #[test]
    fn saga_error_compensate_creation() {
        let err = SagaError::compensate("step1", "compensation failed");
        assert!(err.to_string().contains("IRONIC_SAGA_COMPENSATE"));
        assert!(err.to_string().contains("step1"));
    }

    #[test]
    fn saga_error_clone_and_eq() {
        let err = SagaError::execute("s", "msg");
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}
