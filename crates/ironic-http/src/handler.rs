use std::{any::type_name, future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use ironic_di::ProviderValue;

use crate::{Response, HttpError, IntoResponse};

use super::extract::ExtractedValue;

/// The asynchronous result of erased handler invocation.
pub type HandlerFuture =
    Pin<Box<dyn Future<Output = Result<Response, HttpError>> + Send + 'static>>;

/// Extracted, type-erased arguments supplied to a controller handler adapter.
pub struct HandlerArguments {
    values: Vec<Option<ExtractedValue>>,
}

impl HandlerArguments {
    pub(crate) fn new(values: Vec<ExtractedValue>) -> Self {
        Self {
            values: values.into_iter().map(Some).collect(),
        }
    }

    /// Takes and downcasts the argument at `index`.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when the index is absent, already consumed, or has the wrong type.
    pub fn take<T: Send + 'static>(&mut self, index: usize) -> Result<T, HttpError> {
        let value = self
            .values
            .get_mut(index)
            .and_then(Option::take)
            .ok_or_else(|| {
                HttpError::internal(
                    "RF_HTTP_HANDLER_ARGUMENT_MISSING",
                    format!("Handler argument {index} is unavailable"),
                )
            })?;
        value.downcast::<T>().map(|value| *value).map_err(|_| {
            HttpError::internal(
                "RF_HTTP_HANDLER_TYPE_MISMATCH",
                format!("Handler argument {index} is not `{}`", type_name::<T>()),
            )
        })
    }
}

/// Invokes a controller method through type-erased runtime metadata.
pub trait ErasedHandler: Send + Sync + 'static {
    /// Invokes the handler with a resolved controller and extracted arguments.
    fn call(&self, controller: ProviderValue, arguments: HandlerArguments) -> HandlerFuture;
}

struct HandlerFn<C, F, Fut, R> {
    function: F,
    marker: PhantomData<fn(C) -> (Fut, R)>,
}

impl<C, F, Fut, R> ErasedHandler for HandlerFn<C, F, Fut, R>
where
    C: Send + Sync + 'static,
    F: Fn(Arc<C>, HandlerArguments) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, HttpError>> + Send + 'static,
    R: IntoResponse + 'static,
{
    fn call(&self, controller: ProviderValue, arguments: HandlerArguments) -> HandlerFuture {
        let Ok(controller) = controller.downcast::<C>() else {
            return Box::pin(async {
                Err(HttpError::internal(
                    "RF_HTTP_CONTROLLER_TYPE_MISMATCH",
                    "Controller metadata did not match its provider",
                ))
            });
        };
        let future = (self.function)(controller, arguments);
        Box::pin(async move { future.await?.into_framework_response() })
    }
}

/// Erases a typed asynchronous controller handler.
#[must_use]
pub fn handler_fn<C, F, Fut, R>(function: F) -> Arc<dyn ErasedHandler>
where
    C: Send + Sync + 'static,
    F: Fn(Arc<C>, HandlerArguments) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<R, HttpError>> + Send + 'static,
    R: IntoResponse + 'static,
{
    Arc::new(HandlerFn {
        function,
        marker: PhantomData,
    })
}
