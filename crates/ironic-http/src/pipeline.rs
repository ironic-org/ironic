use std::{any::type_name, fmt, future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use crate::{
    CompiledHttpApplication, CompiledRoute, ExceptionFilterSet, ExtractedValue, FrameworkResponse,
    HttpError, RequestContext,
};

/// The asynchronous result of middleware or interceptor execution.
pub type PipelineFuture<'a> =
    Pin<Box<dyn Future<Output = Result<FrameworkResponse, HttpError>> + Send + 'a>>;

/// The asynchronous result of guard evaluation.
pub type GuardFuture<'a> =
    Pin<Box<dyn Future<Output = Result<GuardDecision, HttpError>> + Send + 'a>>;

/// The asynchronous result of parameter transformation or validation.
pub type PipeFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ExtractedValue, HttpError>> + Send + 'a>>;

/// The result of an authorization guard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardDecision {
    /// Continue to the next guard or pipeline stage.
    Allow,
    /// Stop the request with the framework's standard forbidden response.
    Deny,
}

/// Wraps request execution before guards and handlers.
pub trait Middleware: Send + Sync + 'static {
    /// Executes this middleware.
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a>;
}

/// Determines whether a request may invoke its handler.
pub trait Guard: Send + Sync + 'static {
    /// Evaluates this guard.
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a>;
}

/// Wraps extraction and handler execution.
pub trait Interceptor: Send + Sync + 'static {
    /// Executes this interceptor around the next inner stage.
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a>;
}

/// Transforms or validates one extracted handler parameter.
pub trait ParameterPipe: Send + Sync + 'static {
    /// Transforms the erased value or returns a validation error.
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        context: &'a mut RequestContext,
    ) -> PipeFuture<'a>;

    /// Returns a diagnostic name.
    fn description(&self) -> &'static str;
}

struct SyncPipe<T, U, F> {
    transform: F,
    marker: PhantomData<fn(T) -> U>,
}

impl<T, U, F> ParameterPipe for SyncPipe<T, U, F>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    F: Fn(T) -> Result<U, HttpError> + Send + Sync + 'static,
{
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        _context: &'a mut RequestContext,
    ) -> PipeFuture<'a> {
        let result = value
            .downcast::<T>()
            .map_err(|_| {
                HttpError::internal(
                    "RF_HTTP_PIPE_TYPE_MISMATCH",
                    format!("Parameter pipe expected `{}`", type_name::<T>()),
                )
            })
            .and_then(|value| (self.transform)(*value))
            .map(|value| Box::new(value) as ExtractedValue);
        Box::pin(async move { result })
    }

    fn description(&self) -> &'static str {
        type_name::<F>()
    }
}

/// Creates a synchronous typed transformation or validation pipe.
#[must_use]
pub fn pipe_fn<T, U, F>(transform: F) -> Arc<dyn ParameterPipe>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    F: Fn(T) -> Result<U, HttpError> + Send + Sync + 'static,
{
    Arc::new(SyncPipe {
        transform,
        marker: PhantomData,
    })
}

/// Immutable pipeline component registrations at one scope.
#[derive(Clone, Default)]
pub struct PipelineComponents {
    pub(crate) middleware: Vec<Arc<dyn Middleware>>,
    pub(crate) guards: Vec<Arc<dyn Guard>>,
    pub(crate) interceptors: Vec<Arc<dyn Interceptor>>,
    pub(crate) exception_filters: ExceptionFilterSet,
}

impl fmt::Debug for PipelineComponents {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PipelineComponents")
            .field("middleware_count", &self.middleware.len())
            .field("guard_count", &self.guards.len())
            .field("interceptor_count", &self.interceptors.len())
            .field("exception_filter_count", &self.exception_filters.len())
            .finish()
    }
}

impl PipelineComponents {
    /// Creates an empty component set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            middleware: Vec::new(),
            guards: Vec::new(),
            interceptors: Vec::new(),
            exception_filters: ExceptionFilterSet::new(),
        }
    }

    /// Registers middleware after existing middleware at this scope.
    #[must_use]
    pub fn middleware(mut self, middleware: impl Middleware) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    /// Registers a guard after existing guards at this scope.
    #[must_use]
    pub fn guard(mut self, guard: impl Guard) -> Self {
        self.guards.push(Arc::new(guard));
        self
    }

    /// Registers an interceptor after existing interceptors at this scope.
    #[must_use]
    pub fn interceptor(mut self, interceptor: impl Interceptor) -> Self {
        self.interceptors.push(Arc::new(interceptor));
        self
    }

    /// Registers an exception filter at this scope.
    #[must_use]
    pub fn exception_filter(mut self, filter: Arc<dyn crate::ExceptionFilter>) -> Self {
        self.exception_filters.push(filter);
        self
    }

    pub(crate) fn append(&mut self, other: &Self) {
        self.middleware.extend(other.middleware.iter().cloned());
        self.guards.extend(other.guards.iter().cloned());
        self.interceptors.extend(other.interceptors.iter().cloned());
        self.exception_filters.append(&mut other.exception_filters.clone());
    }
}

struct ExecutionState<'a> {
    application: &'a CompiledHttpApplication,
    route: &'a CompiledRoute,
}

/// A consuming handle that invokes the next middleware exactly once.
pub struct MiddlewareNext<'a> {
    state: &'a ExecutionState<'a>,
    index: usize,
}

impl MiddlewareNext<'_> {
    /// Executes the next middleware or advances to guards.
    pub fn run<'a>(self, context: &'a mut RequestContext) -> PipelineFuture<'a>
    where
        Self: 'a,
    {
        run_middleware(self.state, self.index, context)
    }
}

/// A consuming handle that invokes the next interceptor exactly once.
pub struct InterceptorNext<'a> {
    state: &'a ExecutionState<'a>,
    index: usize,
}

impl InterceptorNext<'_> {
    /// Executes the next interceptor or advances to extraction and the handler.
    pub fn run<'a>(self, context: &'a mut RequestContext) -> PipelineFuture<'a>
    where
        Self: 'a,
    {
        run_interceptor(self.state, self.index, context)
    }
}

pub(crate) async fn execute(
    application: &CompiledHttpApplication,
    route: &CompiledRoute,
    context: &mut RequestContext,
) -> Result<FrameworkResponse, HttpError> {
    context.set_route_metadata(route.metadata().clone());
    let state = ExecutionState { application, route };
    match run_middleware(&state, 0, context).await {
        Ok(response) => Ok(response),
        Err(error) => {
            let filter_ctx = crate::FilterContext::new(route.metadata().clone());
            // Route-level filters (includes controller filters, most specific first)
            if let Some(result) = route.pipeline().exception_filters.catch(&error, &filter_ctx) {
                return result;
            }
            // Global-level filters
            if let Some(result) =
                application.pipeline().exception_filters.catch(&error, &filter_ctx)
            {
                return result;
            }
            Err(error)
        }
    }
}

fn run_middleware<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    if let Some(middleware) = middleware_at(state, index) {
        middleware.handle(
            context,
            MiddlewareNext {
                state,
                index: index + 1,
            },
        )
    } else {
        run_guards(state, context)
    }
}

fn run_guards<'a>(
    state: &'a ExecutionState<'a>,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    Box::pin(async move {
        let count = guard_count(state);
        for index in 0..count {
            match guard_at(state, index)
                .expect("guard index is in bounds")
                .can_activate(context)
                .await?
            {
                GuardDecision::Allow => {}
                GuardDecision::Deny => {
                    return Err(HttpError::forbidden(
                        "RF_HTTP_GUARD_DENIED",
                        "Access to this route was denied",
                    ));
                }
            }
        }
        run_interceptor(state, 0, context).await
    })
}

fn run_interceptor<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    if let Some(interceptor) = interceptor_at(state, index) {
        interceptor.intercept(
            context,
            InterceptorNext {
                state,
                index: index + 1,
            },
        )
    } else {
        Box::pin(async move {
            let controller = route_scope(context)?
                .resolve_key(state.route.controller())
                .await
                .map_err(|_| {
                    HttpError::internal(
                        "RF_HTTP_CONTROLLER_RESOLUTION_FAILED",
                        "Controller resolution failed",
                    )
                })?;
            state.route.invoke_handler(controller, context).await
        })
    }
}

fn route_scope(context: &RequestContext) -> Result<crate::RequestScope, HttpError> {
    context
        .extension::<crate::RequestScope>()
        .cloned()
        .ok_or_else(|| {
            HttpError::internal(
                "IRONIC_HTTP_REQUEST_SCOPE_MISSING",
                "Request dependency scope was not initialized",
            )
        })
}

fn middleware_at<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
) -> Option<&'a Arc<dyn Middleware>> {
    let global = &state.application.pipeline().middleware;
    global
        .get(index)
        .or_else(|| state.route.pipeline().middleware.get(index - global.len()))
}

fn guard_count(state: &ExecutionState<'_>) -> usize {
    state.application.pipeline().guards.len() + state.route.pipeline().guards.len()
}

fn guard_at<'a>(state: &'a ExecutionState<'a>, index: usize) -> Option<&'a Arc<dyn Guard>> {
    let global = &state.application.pipeline().guards;
    global
        .get(index)
        .or_else(|| state.route.pipeline().guards.get(index - global.len()))
}

fn interceptor_at<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
) -> Option<&'a Arc<dyn Interceptor>> {
    let global = &state.application.pipeline().interceptors;
    global.get(index).or_else(|| {
        state
            .route
            .pipeline()
            .interceptors
            .get(index - global.len())
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use ironic_di::{ContainerBuilder, ProviderDefinition, Scope};

    use super::*;
    use crate::{
        ControllerDefinition, ExtractFuture, FrameworkRequest, HeaderMap, HttpMethod, HttpStatus,
        IntoFrameworkResponse, ParameterExtractor, RouteDefinition, Uri, compile_controller_routes,
        handler_fn, parse_int,
    };

    type Events = Arc<Mutex<Vec<&'static str>>>;

    fn push(events: &Events, event: &'static str) {
        events.lock().unwrap().push(event);
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FailureStage {
        None,
        Middleware,
        GuardDenied,
        GuardError,
        Interceptor,
        Extraction,
        Pipe,
        Handler,
    }

    struct RecordingMiddleware {
        events: Events,
        before: &'static str,
        after: &'static str,
        fail: bool,
    }

    impl Middleware for RecordingMiddleware {
        fn handle<'a>(
            &'a self,
            context: &'a mut RequestContext,
            next: MiddlewareNext<'a>,
        ) -> PipelineFuture<'a> {
            Box::pin(async move {
                push(&self.events, self.before);
                if self.fail {
                    return Err(HttpError::bad_request(
                        "TEST_MIDDLEWARE_FAILED",
                        "middleware failed",
                    ));
                }
                let result = next.run(context).await;
                push(&self.events, self.after);
                result
            })
        }
    }

    struct RecordingGuard {
        events: Events,
        name: &'static str,
        stage: FailureStage,
    }

    impl Guard for RecordingGuard {
        fn can_activate<'a>(&'a self, _context: &'a mut RequestContext) -> GuardFuture<'a> {
            Box::pin(async move {
                push(&self.events, self.name);
                match self.stage {
                    FailureStage::GuardDenied => Ok(GuardDecision::Deny),
                    FailureStage::GuardError => {
                        Err(HttpError::bad_request("TEST_GUARD_FAILED", "guard failed"))
                    }
                    _ => Ok(GuardDecision::Allow),
                }
            })
        }
    }

    struct RecordingInterceptor {
        events: Events,
        before: &'static str,
        after: &'static str,
        fail: bool,
    }

    impl Interceptor for RecordingInterceptor {
        fn intercept<'a>(
            &'a self,
            context: &'a mut RequestContext,
            next: InterceptorNext<'a>,
        ) -> PipelineFuture<'a> {
            Box::pin(async move {
                push(&self.events, self.before);
                if self.fail {
                    return Err(HttpError::bad_request(
                        "TEST_INTERCEPTOR_FAILED",
                        "interceptor failed",
                    ));
                }
                let result = next.run(context).await;
                push(&self.events, self.after);
                result
            })
        }
    }

    struct RecordingExtractor {
        events: Events,
        fail: bool,
    }

    impl ParameterExtractor for RecordingExtractor {
        fn extract<'a>(&'a self, _context: &'a mut RequestContext) -> ExtractFuture<'a> {
            Box::pin(async move {
                push(&self.events, "extract");
                if self.fail {
                    Err(HttpError::bad_request(
                        "TEST_EXTRACTION_FAILED",
                        "extraction failed",
                    ))
                } else {
                    Ok(Box::new(7_u64) as ExtractedValue)
                }
            })
        }

        fn description(&self) -> &'static str {
            "recording extractor"
        }
    }

    struct RecordingPipe {
        events: Events,
        fail: bool,
    }

    impl ParameterPipe for RecordingPipe {
        fn transform<'a>(
            &'a self,
            value: ExtractedValue,
            _context: &'a mut RequestContext,
        ) -> PipeFuture<'a> {
            Box::pin(async move {
                push(&self.events, "pipe");
                if self.fail {
                    Err(HttpError::unprocessable_entity(
                        "TEST_VALIDATION_FAILED",
                        "validation failed",
                    ))
                } else {
                    Ok(value)
                }
            })
        }

        fn description(&self) -> &'static str {
            "recording pipe"
        }
    }

    struct Controller;

    fn component_middleware(
        events: &Events,
        before: &'static str,
        after: &'static str,
        fail: bool,
    ) -> RecordingMiddleware {
        RecordingMiddleware {
            events: Arc::clone(events),
            before,
            after,
            fail,
        }
    }

    fn component_guard(events: &Events, name: &'static str, stage: FailureStage) -> RecordingGuard {
        RecordingGuard {
            events: Arc::clone(events),
            name,
            stage,
        }
    }

    fn component_interceptor(
        events: &Events,
        before: &'static str,
        after: &'static str,
        fail: bool,
    ) -> RecordingInterceptor {
        RecordingInterceptor {
            events: Arc::clone(events),
            before,
            after,
            fail,
        }
    }

    fn fixture(stage: FailureStage) -> (CompiledHttpApplication, Events) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "pipeline",
            handler_fn(move |_controller: Arc<Controller>, mut arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let _value = arguments.take::<u64>(0)?;
                    if stage == FailureStage::Handler {
                        Err(HttpError::bad_request(
                            "TEST_HANDLER_FAILED",
                            "handler failed",
                        ))
                    } else {
                        Ok(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
                    }
                }
            }),
        )
        .unwrap()
        .parameter_with_pipe(
            RecordingExtractor {
                events: Arc::clone(&events),
                fail: stage == FailureStage::Extraction,
            },
            Arc::new(RecordingPipe {
                events: Arc::clone(&events),
                fail: stage == FailureStage::Pipe,
            }),
        )
        .middleware(component_middleware(
            &events,
            "route-middleware-before",
            "route-middleware-after",
            false,
        ))
        .guard(component_guard(&events, "route-guard", FailureStage::None))
        .interceptor(component_interceptor(
            &events,
            "route-interceptor-before",
            "route-interceptor-after",
            false,
        ));

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/pipeline", provider)
            .unwrap()
            .middleware(component_middleware(
                &events,
                "controller-middleware-before",
                "controller-middleware-after",
                false,
            ))
            .guard(component_guard(
                &events,
                "controller-guard",
                FailureStage::None,
            ))
            .interceptor(component_interceptor(
                &events,
                "controller-interceptor-before",
                "controller-interceptor-after",
                false,
            ))
            .route(route);
        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes)
            .middleware(component_middleware(
                &events,
                "global-middleware-before",
                "global-middleware-after",
                stage == FailureStage::Middleware,
            ))
            .guard(component_guard(&events, "global-guard", stage))
            .interceptor(component_interceptor(
                &events,
                "global-interceptor-before",
                "global-interceptor-after",
                stage == FailureStage::Interceptor,
            ));
        (application, events)
    }

    fn request_context() -> RequestContext {
        RequestContext::new(FrameworkRequest::new(
            HttpMethod::GET,
            "/pipeline".parse::<Uri>().unwrap(),
            HeaderMap::new(),
            Vec::new(),
        ))
    }

    async fn run(stage: FailureStage) -> (Result<FrameworkResponse, HttpError>, Vec<&'static str>) {
        let (application, events) = fixture(stage);
        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        let events = events.lock().unwrap().clone();
        (result, events)
    }

    #[tokio::test]
    async fn executes_all_scopes_in_documented_order() {
        let (result, events) = run(FailureStage::None).await;
        assert_eq!(result.unwrap().status(), HttpStatus::NO_CONTENT);
        assert_eq!(
            events,
            [
                "global-middleware-before",
                "controller-middleware-before",
                "route-middleware-before",
                "global-guard",
                "controller-guard",
                "route-guard",
                "global-interceptor-before",
                "controller-interceptor-before",
                "route-interceptor-before",
                "extract",
                "pipe",
                "handler",
                "route-interceptor-after",
                "controller-interceptor-after",
                "global-interceptor-after",
                "route-middleware-after",
                "controller-middleware-after",
                "global-middleware-after",
            ]
        );
    }

    #[tokio::test]
    async fn middleware_errors_stop_the_pipeline() {
        let (result, events) = run(FailureStage::Middleware).await;
        assert_eq!(result.unwrap_err().code(), "TEST_MIDDLEWARE_FAILED");
        assert_eq!(events, ["global-middleware-before"]);
    }

    #[tokio::test]
    async fn guard_denial_unwinds_middleware_without_entering_interceptors() {
        let (result, events) = run(FailureStage::GuardDenied).await;
        assert_eq!(result.unwrap_err().code(), "RF_HTTP_GUARD_DENIED");
        assert_eq!(
            events,
            [
                "global-middleware-before",
                "controller-middleware-before",
                "route-middleware-before",
                "global-guard",
                "route-middleware-after",
                "controller-middleware-after",
                "global-middleware-after",
            ]
        );
    }

    #[tokio::test]
    async fn guard_errors_propagate_without_becoming_denials() {
        let (result, _) = run(FailureStage::GuardError).await;
        assert_eq!(result.unwrap_err().code(), "TEST_GUARD_FAILED");
    }

    #[tokio::test]
    async fn interceptor_errors_stop_inner_execution() {
        let (result, events) = run(FailureStage::Interceptor).await;
        assert_eq!(result.unwrap_err().code(), "TEST_INTERCEPTOR_FAILED");
        assert!(!events.contains(&"extract"));
    }

    #[tokio::test]
    async fn extraction_errors_unwind_interceptors_and_middleware() {
        let (result, events) = run(FailureStage::Extraction).await;
        assert_eq!(result.unwrap_err().code(), "TEST_EXTRACTION_FAILED");
        assert!(events.contains(&"route-interceptor-after"));
        assert!(!events.contains(&"pipe"));
    }

    #[tokio::test]
    async fn validation_errors_stop_before_the_handler() {
        let (result, events) = run(FailureStage::Pipe).await;
        assert_eq!(result.unwrap_err().code(), "TEST_VALIDATION_FAILED");
        assert!(events.contains(&"pipe"));
        assert!(!events.contains(&"handler"));
    }

    #[tokio::test]
    async fn handler_errors_unwind_all_wrappers() {
        let (result, events) = run(FailureStage::Handler).await;
        assert_eq!(result.unwrap_err().code(), "TEST_HANDLER_FAILED");
        assert_eq!(events.last(), Some(&"global-middleware-after"));
    }

    #[tokio::test]
    async fn typed_pipe_helpers_transform_values() {
        let pipe = pipe_fn::<u64, String, _>(|value| Ok(value.to_string()));
        let mut context = request_context();
        let value = pipe
            .transform(Box::new(7_u64), &mut context)
            .await
            .unwrap()
            .downcast::<String>()
            .unwrap();
        assert_eq!(*value, "7");
    }

    #[test]
    fn http_errors_remain_convertible_after_pipeline_failures() {
        let response = HttpError::forbidden("DENIED", "denied")
            .into_framework_response()
            .unwrap();
        assert_eq!(response.status(), HttpStatus::FORBIDDEN);
    }

    // ------------------------------------------------------------------
    // Pipe registration tests (global, controller, and route-level)
    // ------------------------------------------------------------------

    struct LabeledRecordingPipe {
        events: Events,
        label: &'static str,
        fail: bool,
    }

    impl ParameterPipe for LabeledRecordingPipe {
        fn transform<'a>(
            &'a self,
            value: ExtractedValue,
            _context: &'a mut RequestContext,
        ) -> PipeFuture<'a> {
            Box::pin(async move {
                push(&self.events, self.label);
                if self.fail {
                    Err(HttpError::unprocessable_entity(
                        "TEST_VALIDATION_FAILED",
                        "validation failed",
                    ))
                } else {
                    Ok(value)
                }
            })
        }

        fn description(&self) -> &'static str {
            self.label
        }
    }

    fn labeled_pipe(events: &Events, label: &'static str) -> Arc<dyn ParameterPipe> {
        Arc::new(LabeledRecordingPipe {
            events: events.clone(),
            label,
            fail: false,
        })
    }

    fn pipe_fixture() -> (CompiledHttpApplication, Events) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/pipe-test",
            "pipe_test",
            handler_fn(move |_controller: Arc<Controller>, mut arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let _value = arguments.take::<u64>(0)?;
                    Ok(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
                }
            }),
        )
        .unwrap()
        .parameter_with_pipe(
            RecordingExtractor {
                events: Arc::clone(&events),
                fail: false,
            },
            labeled_pipe(&events, "route-pipe"),
        );

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .pipe(labeled_pipe(&events, "controller-pipe"))
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes)
            .pipe(&labeled_pipe(&events, "global-pipe"));

        (application, events)
    }

    #[tokio::test]
    async fn global_pipe_applies_to_all_parameters() {
        let (application, events) = pipe_fixture();
        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        let events = events.lock().unwrap().clone();
        assert!(
            events.contains(&"global-pipe"),
            "global-pipe should execute"
        );
    }

    #[tokio::test]
    async fn controller_pipe_applies_to_all_route_parameters() {
        let (application, events) = pipe_fixture();
        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        let events = events.lock().unwrap().clone();
        assert!(
            events.contains(&"controller-pipe"),
            "controller-pipe should execute"
        );
    }

    #[tokio::test]
    async fn all_pipe_levels_execute_in_order() {
        let (application, events) = pipe_fixture();
        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        let events = events.lock().unwrap().clone();

        let pipe_events: Vec<&str> = events
            .iter()
            .filter(|e| e.ends_with("-pipe"))
            .copied()
            .collect();
        assert_eq!(
            pipe_events,
            vec!["global-pipe", "controller-pipe", "route-pipe"],
            "pipes should execute in order: global -> controller -> route"
        );
    }

    #[tokio::test]
    async fn global_pipe_failure_stops_pipeline() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);
        let failing_pipe: Arc<dyn ParameterPipe> = Arc::new(LabeledRecordingPipe {
            events: events.clone(),
            label: "failing-global-pipe",
            fail: true,
        });

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/pipe-fail",
            "pipe_fail",
            handler_fn(move |_controller: Arc<Controller>, mut arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let _value = arguments.take::<u64>(0)?;
                    Ok(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
                }
            }),
        )
        .unwrap()
        .parameter_with_pipe(
            RecordingExtractor {
                events: Arc::clone(&events),
                fail: false,
            },
            labeled_pipe(&events, "route-pipe"),
        );

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application =
            CompiledHttpApplication::new(container.build(), routes).pipe(&failing_pipe);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert_eq!(result.unwrap_err().code(), "TEST_VALIDATION_FAILED");
    }

    // ------------------------------------------------------------------
    // Exception filter tests
    // ------------------------------------------------------------------

    struct RecordingExceptionFilter {
        events: Events,
        code: &'static str,
        handled: bool,
    }

    impl crate::ExceptionFilter for RecordingExceptionFilter {
        fn catch(
            &self,
            error: &HttpError,
            _context: &crate::FilterContext,
        ) -> Result<FrameworkResponse, HttpError> {
            push(&self.events, self.code);
            if self.handled && error.code() == self.code {
                Ok(FrameworkResponse::empty(HttpStatus::IM_A_TEAPOT))
            } else {
                Err(HttpError::bad_request("UNHANDLED", "not handled"))
            }
        }
    }

    fn exception_filter_fixture() -> (CompiledHttpApplication, Events) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);

        // Route that always fails with a specific error code
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/filter",
            "filter_test",
            handler_fn(move |_controller: Arc<Controller>, _arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    Err::<FrameworkResponse, HttpError>(HttpError::internal(
                        "SPECIFIC_ERROR",
                        "specific error",
                    ))
                }
            }),
        )
        .unwrap()
        .parameter(RecordingExtractor {
            events: Arc::clone(&events),
            fail: false,
        });

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes);
        (application, events)
    }

    #[tokio::test]
    async fn route_exception_filter_catches_error() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);

        let filter = Arc::new(RecordingExceptionFilter {
            events: events.clone(),
            code: "SPECIFIC_ERROR",
            handled: true,
        });

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/filter-route",
            "filter_route",
            handler_fn(move |_controller: Arc<Controller>, _arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let err: Result<FrameworkResponse, HttpError> =
                        Err(HttpError::internal("SPECIFIC_ERROR", "specific error"));
                    err
                }
            }),
        )
        .unwrap()
        .parameter(RecordingExtractor {
            events: Arc::clone(&events),
            fail: false,
        })
        .exception_filter(filter);

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), HttpStatus::IM_A_TEAPOT);
        let events = events.lock().unwrap().clone();
        assert!(events.contains(&"SPECIFIC_ERROR"));
    }

    #[tokio::test]
    async fn global_exception_filter_catches_error() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);

        let filter = Arc::new(RecordingExceptionFilter {
            events: events.clone(),
            code: "SPECIFIC_ERROR",
            handled: true,
        });

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/filter-global",
            "filter_global",
            handler_fn(move |_controller: Arc<Controller>, _arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let err: Result<FrameworkResponse, HttpError> =
                        Err(HttpError::internal("SPECIFIC_ERROR", "specific error"));
                    err
                }
            }),
        )
        .unwrap()
        .parameter(RecordingExtractor {
            events: Arc::clone(&events),
            fail: false,
        });

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes)
            .exception_filter(filter);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), HttpStatus::IM_A_TEAPOT);
    }

    #[tokio::test]
    async fn route_filter_takes_precedence_over_global() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);

        let global_filter = Arc::new(RecordingExceptionFilter {
            events: events.clone(),
            code: "SPECIFIC_ERROR",
            handled: true,
        });
        let route_filter = Arc::new(RecordingExceptionFilter {
            events: events.clone(),
            code: "ROUTE_FILTER",
            handled: true,
        });

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/filter-precedence",
            "filter_precedence",
            handler_fn(move |_controller: Arc<Controller>, _arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let err: Result<FrameworkResponse, HttpError> =
                        Err(HttpError::internal("ROUTE_FILTER", "route filter error"));
                    err
                }
            }),
        )
        .unwrap()
        .parameter(RecordingExtractor {
            events: Arc::clone(&events),
            fail: false,
        })
        .exception_filter(route_filter);

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes)
            .exception_filter(global_filter);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), HttpStatus::IM_A_TEAPOT);
    }

    #[tokio::test]
    async fn unhandled_error_propagates_to_caller() {
        let (application, _events) = exception_filter_fixture();
        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "SPECIFIC_ERROR");
    }

    #[tokio::test]
    async fn filter_has_access_to_route_metadata() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);

        struct MetadataInspector {
            events: Events,
        }

        impl crate::ExceptionFilter for MetadataInspector {
            fn catch(
                &self,
                _error: &HttpError,
                context: &crate::FilterContext,
            ) -> Result<FrameworkResponse, HttpError> {
                push(&self.events, "filter_executed");
                // Verify FilterContext provides route metadata
                let _metadata = context.route_metadata();
                Ok(FrameworkResponse::empty(HttpStatus::IM_A_TEAPOT))
            }
        }

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/filter-metadata",
            "filter_metadata",
            handler_fn(move |_controller: Arc<Controller>, _arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let err: Result<FrameworkResponse, HttpError> =
                        Err(HttpError::internal("METADATA_ERROR", "metadata error"));
                    err
                }
            }),
        )
        .unwrap()
        .parameter(RecordingExtractor {
            events: Arc::clone(&events),
            fail: false,
        })
        .exception_filter(Arc::new(MetadataInspector {
            events: events.clone(),
        }));

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        let events = events.lock().unwrap().clone();
        assert!(events.contains(&"filter_executed"));
    }

    // ------------------------------------------------------------------
    // Custom decorator tests
    // ------------------------------------------------------------------

    /// A custom extractor that provides a static string value.
    struct CustomStringExtractor {
        value: String,
    }

    impl CustomStringExtractor {
        fn new() -> Self {
            Self {
                value: "custom-decorator-value".to_string(),
            }
        }
    }

    impl ParameterExtractor for CustomStringExtractor {
        fn extract<'a>(&'a self, _context: &'a mut RequestContext) -> ExtractFuture<'a> {
            let v = self.value.clone();
            Box::pin(async move { Ok(Box::new(v) as ExtractedValue) })
        }

        fn description(&self) -> &'static str {
            "custom_string"
        }
    }

    #[tokio::test]
    async fn custom_decorator_extracts_parameter() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler_events = Arc::clone(&events);

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/custom",
            "custom_test",
            handler_fn(move |_controller: Arc<Controller>, mut arguments| {
                let events = Arc::clone(&handler_events);
                async move {
                    push(&events, "handler");
                    let value = arguments.take::<String>(0).unwrap();
                    assert_eq!(value, "custom-decorator-value");
                    Ok(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
                }
            }),
        )
        .unwrap()
        .parameter(CustomStringExtractor::new());

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
        let events = events.lock().unwrap().clone();
        assert!(events.contains(&"handler"));
    }

    #[tokio::test]
    async fn custom_decorator_with_pipe_chaining() {
        let events = Arc::new(Mutex::new(Vec::new()));

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/custom-pipe",
            "custom_pipe",
            handler_fn(move |_controller: Arc<Controller>, mut arguments| {
                let events = Arc::clone(&events);
                async move {
                    push(&events, "handler");
                    let value = arguments.take::<i64>(0).unwrap();
                    assert_eq!(value, 42);
                    Ok(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
                }
            }),
        )
        .unwrap()
        .parameter_with_pipe(
            CustomStringExtractor {
                value: "42".to_string(),
            },
            parse_int(),
        );

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn custom_decorator_with_pipe_failure() {
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/custom-pipe-fail",
            "custom_pipe_fail",
            handler_fn(|_controller: Arc<Controller>, _arguments| async move {
                Ok::<_, HttpError>(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
            }),
        )
        .unwrap()
        .parameter_with_pipe(
            CustomStringExtractor {
                value: "not-a-number".to_string(),
            },
            parse_int(),
        );

        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(Controller)
        });
        let controller = ControllerDefinition::new::<Controller>("/", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        let application = CompiledHttpApplication::new(container.build(), routes);

        let mut context = request_context();
        let result = application
            .execute(&application.routes()[0], &mut context)
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "RF_PARSE_INT_FAILED");
    }
}
