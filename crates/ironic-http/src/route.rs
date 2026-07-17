use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use ironic_di::{Container, Dependency, ProviderDefinition, ProviderKey, ProviderValue};

use crate::{
    ErasedHandler, FrameworkResponse, Guard, HandlerArguments, HttpError, HttpMethod, Interceptor,
    Middleware, ParameterExtractor, ParameterPipe, PipelineComponents, RequestContext,
    VersionMetadata,
};

#[derive(Clone)]
struct ParameterDefinition {
    extractor: Arc<dyn ParameterExtractor>,
    pipes: Vec<Arc<dyn ParameterPipe>>,
}

/// Cloneable, type-indexed metadata attached to a route definition.
#[derive(Clone, Default)]
pub struct RouteMetadata {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl RouteMetadata {
    /// Creates an empty metadata map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts typed metadata and returns the previous value, when present.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<Arc<T>> {
        self.values
            .insert(TypeId::of::<T>(), Arc::new(value))
            .and_then(|previous| Arc::downcast(previous).ok())
    }

    /// Returns typed metadata attached to the route.
    #[must_use]
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())?.downcast_ref()
    }

    /// Returns whether the map contains no metadata.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl fmt::Debug for RouteMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RouteMetadata")
            .field("entry_count", &self.values.len())
            .finish()
    }
}

/// A route or controller definition failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum RouteError {
    /// A route path is not absolute.
    #[error("RF_ROUTE_INVALID_PATH: route path `{path}` must begin with `/`")]
    InvalidPath {
        /// The invalid path.
        path: String,
    },
    /// A controller definition uses a provider of another concrete type.
    #[error("RF_ROUTE_CONTROLLER_PROVIDER_MISMATCH: `{controller}` does not match `{provider}`")]
    ControllerProviderMismatch {
        /// The declared controller key.
        controller: ProviderKey,
        /// The provider definition key.
        provider: ProviderKey,
    },
    /// Two controller routes have the same method and normalized path.
    #[error("RF_ROUTE_DUPLICATE: duplicate route `{method} {path}`")]
    DuplicateRoute {
        /// The duplicated method.
        method: HttpMethod,
        /// The normalized route path.
        path: String,
    },
}

/// An executable route owned by a controller definition.
#[derive(Clone)]
pub struct RouteDefinition {
    method: HttpMethod,
    path: String,
    handler_name: &'static str,
    parameters: Vec<ParameterDefinition>,
    handler: Arc<dyn ErasedHandler>,
    pipeline: PipelineComponents,
    metadata: RouteMetadata,
    /// Per-route request timeout. Overrides the global timeout for this route.
    pub(crate) timeout: Option<std::time::Duration>,
}

impl RouteDefinition {
    /// Creates and validates a route definition.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::InvalidPath`] when `path` is not absolute.
    pub fn new(
        method: HttpMethod,
        path: impl Into<String>,
        handler_name: &'static str,
        handler: Arc<dyn ErasedHandler>,
    ) -> Result<Self, RouteError> {
        let path = normalize_path(&path.into())?;
        Ok(Self {
            method,
            path,
            handler_name,
            parameters: Vec::new(),
            handler,
            pipeline: PipelineComponents::new(),
            metadata: RouteMetadata::new(),
            timeout: None,
        })
    }

    /// Sets a per-route request timeout. Overrides the global adapter timeout.
    #[must_use]
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Adds a parameter extractor in handler declaration order.
    #[must_use]
    pub fn parameter(mut self, extractor: impl ParameterExtractor) -> Self {
        self.parameters.push(ParameterDefinition {
            extractor: Arc::new(extractor),
            pipes: Vec::new(),
        });
        self
    }

    /// Adds a parameter extractor followed by a typed transformation or validation pipe.
    #[must_use]
    pub fn parameter_with_pipe(
        mut self,
        extractor: impl ParameterExtractor,
        pipe: Arc<dyn ParameterPipe>,
    ) -> Self {
        self.parameters.push(ParameterDefinition {
            extractor: Arc::new(extractor),
            pipes: vec![pipe],
        });
        self
    }

    /// Adds a parameter extractor followed by multiple ordered pipes.
    #[must_use]
    pub fn parameter_with_pipes(
        mut self,
        extractor: impl ParameterExtractor,
        pipes: impl IntoIterator<Item = Arc<dyn ParameterPipe>>,
    ) -> Self {
        self.parameters.push(ParameterDefinition {
            extractor: Arc::new(extractor),
            pipes: pipes.into_iter().collect(),
        });
        self
    }

    /// Registers route-level middleware.
    #[must_use]
    pub fn middleware(mut self, middleware: impl Middleware) -> Self {
        self.pipeline = self.pipeline.middleware(middleware);
        self
    }

    /// Registers a route-level guard.
    #[must_use]
    pub fn guard(mut self, guard: impl Guard) -> Self {
        self.pipeline = self.pipeline.guard(guard);
        self
    }

    /// Registers a route-level interceptor.
    #[must_use]
    pub fn interceptor(mut self, interceptor: impl Interceptor) -> Self {
        self.pipeline = self.pipeline.interceptor(interceptor);
        self
    }

    /// Attaches typed metadata for tooling such as `OpenAPI` generators.
    #[must_use]
    pub fn metadata<T: Send + Sync + 'static>(mut self, metadata: T) -> Self {
        self.metadata.insert(metadata);
        self
    }

    /// Attaches cache metadata to this route.
    #[must_use]
    pub fn cache(mut self, metadata: crate::CacheMetadata) -> Self {
        self.metadata.insert(metadata);
        self
    }

    /// Registers a route-level exception filter.
    #[must_use]
    pub fn exception_filter(mut self, filter: Arc<dyn crate::ExceptionFilter>) -> Self {
        self.pipeline = self.pipeline.exception_filter(filter);
        self
    }

    /// Registers a global-level pipe for all parameters of this route.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn pipe(mut self, pipe: Arc<dyn ParameterPipe>) -> Self {
        for parameter in &mut self.parameters {
            parameter.pipes.push(Arc::clone(&pipe));
        }
        self
    }

    /// Returns all typed metadata attached to this route.
    #[must_use]
    pub const fn route_metadata(&self) -> &RouteMetadata {
        &self.metadata
    }

    /// Returns the route method.
    #[must_use]
    pub const fn method(&self) -> &HttpMethod {
        &self.method
    }

    /// Returns the controller-relative normalized path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the handler name used in diagnostics.
    #[must_use]
    pub const fn handler_name(&self) -> &'static str {
        self.handler_name
    }
}

impl fmt::Debug for RouteDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RouteDefinition")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("handler_name", &self.handler_name)
            .field("parameter_count", &self.parameters.len())
            .finish_non_exhaustive()
    }
}

/// Static metadata, provider construction, and routes for a controller.
#[derive(Clone)]
pub struct ControllerDefinition {
    key: ProviderKey,
    path: String,
    provider: ProviderDefinition,
    routes: Vec<RouteDefinition>,
    pipeline: PipelineComponents,
    pipes: Vec<Arc<dyn ParameterPipe>>,
    version: Option<crate::VersionMetadata>,
}

impl fmt::Debug for ControllerDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ControllerDefinition")
            .field("key", &self.key)
            .field("path", &self.path)
            .field("route_count", &self.routes.len())
            .field("pipe_count", &self.pipes.len())
            .finish_non_exhaustive()
    }
}

impl ControllerDefinition {
    /// Creates controller metadata from its provider definition.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError`] when the path is invalid or the provider has another type.
    pub fn new<T: Send + Sync + 'static>(
        path: impl Into<String>,
        provider: ProviderDefinition,
    ) -> Result<Self, RouteError> {
        let key = ProviderKey::of::<T>();
        if provider.key() != key {
            return Err(RouteError::ControllerProviderMismatch {
                controller: key,
                provider: provider.key(),
            });
        }
        Ok(Self {
            key,
            path: normalize_path(&path.into())?,
            provider,
            routes: Vec::new(),
            pipeline: PipelineComponents::new(),
            pipes: Vec::new(),
            version: None,
        })
    }

    /// Adds one controller route.
    #[must_use]
    pub fn route(mut self, route: RouteDefinition) -> Self {
        self.routes.push(route);
        self
    }

    /// Adds controller routes in declaration order.
    #[must_use]
    pub fn with_routes(mut self, routes: impl IntoIterator<Item = RouteDefinition>) -> Self {
        self.routes.extend(routes);
        self
    }

    /// Registers controller-level middleware.
    #[must_use]
    pub fn middleware(mut self, middleware: impl Middleware) -> Self {
        self.pipeline = self.pipeline.middleware(middleware);
        self
    }

    /// Registers a controller-level guard.
    #[must_use]
    pub fn guard(mut self, guard: impl Guard) -> Self {
        self.pipeline = self.pipeline.guard(guard);
        self
    }

    /// Registers a controller-level interceptor.
    #[must_use]
    pub fn interceptor(mut self, interceptor: impl Interceptor) -> Self {
        self.pipeline = self.pipeline.interceptor(interceptor);
        self
    }

    /// Attaches version metadata to this controller for API versioning.
    #[must_use]
    pub fn version(mut self, metadata: crate::VersionMetadata) -> Self {
        self.version = Some(metadata);
        self
    }

    /// Registers a controller-level exception filter.
    #[must_use]
    pub fn exception_filter(mut self, filter: Arc<dyn crate::ExceptionFilter>) -> Self {
        self.pipeline = self.pipeline.exception_filter(filter);
        self
    }

    /// Registers a controller-level pipe applied to all route parameters.
    #[must_use]
    pub fn pipe(mut self, pipe: Arc<dyn ParameterPipe>) -> Self {
        self.pipes.push(pipe);
        self
    }

    /// Returns the version metadata, if any.
    #[must_use]
    pub const fn version_metadata(&self) -> Option<&crate::VersionMetadata> {
        self.version.as_ref()
    }

    /// Returns the controller's concrete type key.
    #[must_use]
    pub const fn key(&self) -> ProviderKey {
        self.key
    }

    /// Returns the controller's provider definition.
    #[must_use]
    pub const fn provider(&self) -> &ProviderDefinition {
        &self.provider
    }

    /// Returns the controller's declared dependencies.
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        self.provider.dependencies()
    }

    /// Returns controller-relative route definitions.
    #[must_use]
    pub fn routes(&self) -> &[RouteDefinition] {
        &self.routes
    }

    pub(crate) fn compile_routes(&self) -> Result<Vec<CompiledRoute>, RouteError> {
        let mut seen = HashSet::new();
        let mut compiled = Vec::with_capacity(self.routes.len());
        for route in &self.routes {
            let path = join_paths(&self.path, route.path());
            if !seen.insert((route.method.clone(), path.clone())) {
                return Err(RouteError::DuplicateRoute {
                    method: route.method.clone(),
                    path,
                });
            }
            let mut pipeline = self.pipeline.clone();
            pipeline.append(&route.pipeline);
            let mut metadata = route.metadata.clone();
            if let Some(version) = &self.version {
                metadata.insert(version.clone());
            }
            let mut cloneable_params = route.parameters.clone();
            if !self.pipes.is_empty() {
                for param in &mut cloneable_params {
                    let mut all_pipes = self.pipes.clone();
                    all_pipes.append(&mut param.pipes);
                    param.pipes = all_pipes;
                }
            }
            compiled.push(CompiledRoute {
                controller: self.key,
                method: route.method.clone(),
                path,
                handler_name: route.handler_name,
                parameters: cloneable_params,
                handler: Arc::clone(&route.handler),
                pipeline,
                metadata,
            });
        }
        Ok(compiled)
    }
}

/// An executable WebSocket gateway registered with a compiled HTTP application.
#[derive(Clone)]
pub struct WsGatewayDefinition {
    /// The WebSocket upgrade path.
    pub path: String,
    /// The owning controller/provider key resolved via DI.
    pub controller: ProviderKey,
    /// The handler name for diagnostics.
    pub handler_name: &'static str,
}

impl fmt::Debug for WsGatewayDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WsGatewayDefinition")
            .field("path", &self.path)
            .field("controller", &self.controller)
            .field("handler_name", &self.handler_name)
            .finish()
    }
}

/// A normalized executable route in a compiled HTTP application.
#[derive(Clone)]
pub struct CompiledRoute {
    controller: ProviderKey,
    method: HttpMethod,
    path: String,
    handler_name: &'static str,
    parameters: Vec<ParameterDefinition>,
    handler: Arc<dyn ErasedHandler>,
    pipeline: PipelineComponents,
    metadata: RouteMetadata,
}

impl CompiledRoute {
    /// Returns the owning controller key.
    #[must_use]
    pub const fn controller(&self) -> ProviderKey {
        self.controller
    }

    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(&self) -> &HttpMethod {
        &self.method
    }

    /// Returns the normalized application path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the handler name.
    #[must_use]
    pub const fn handler_name(&self) -> &'static str {
        self.handler_name
    }

    /// Returns all typed metadata attached before route compilation.
    #[must_use]
    pub const fn metadata(&self) -> &RouteMetadata {
        &self.metadata
    }

    /// Returns the version metadata if the route is versioned.
    #[must_use]
    pub fn version(&self) -> Option<crate::VersionMetadata> {
        self.metadata.get::<crate::VersionMetadata>().cloned()
    }

    /// Returns the route path, with the version prefix prepended when URI versioning is active.
    #[must_use]
    pub fn versioned_path(&self) -> String {
        self.version()
            .filter(|v| v.strategy == crate::VersioningStrategy::Uri)
            .map_or_else(
                || self.path.clone(),
                |v| format!("{}{}", v.uri_prefix(), self.path),
            )
    }

    /// Extracts parameters and invokes the erased handler.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when extraction or handler invocation fails.
    pub(crate) async fn invoke_handler(
        &self,
        controller: ProviderValue,
        context: &mut RequestContext,
    ) -> Result<FrameworkResponse, HttpError> {
        let mut arguments = Vec::with_capacity(self.parameters.len());
        for parameter in &self.parameters {
            let mut value = parameter.extractor.extract(context).await?;
            for pipe in &parameter.pipes {
                value = pipe.transform(value, context).await?;
            }
            arguments.push(value);
        }
        self.handler
            .call(controller, HandlerArguments::new(arguments))
            .await
    }

    pub(crate) const fn pipeline(&self) -> &PipelineComponents {
        &self.pipeline
    }
}

impl fmt::Debug for CompiledRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledRoute")
            .field("controller", &self.controller)
            .field("method", &self.method)
            .field("path", &self.path)
            .field("handler_name", &self.handler_name)
            .finish_non_exhaustive()
    }
}

/// Immutable runtime state consumed by an HTTP platform adapter.
#[derive(Clone)]
pub struct CompiledHttpApplication {
    container: Container,
    routes: Arc<[CompiledRoute]>,
    ws_gateways: Arc<[WsGatewayDefinition]>,
    pipeline: PipelineComponents,
}

impl CompiledHttpApplication {
    /// Creates runtime state from a container and compiled routes.
    #[must_use]
    pub fn new(container: Container, routes: Vec<CompiledRoute>) -> Self {
        Self {
            container,
            routes: routes.into(),
            ws_gateways: Arc::new([]),
            pipeline: PipelineComponents::new(),
        }
    }

    /// Returns the application container.
    #[must_use]
    pub const fn container(&self) -> &Container {
        &self.container
    }

    /// Returns all executable routes.
    #[must_use]
    pub fn routes(&self) -> &[CompiledRoute] {
        &self.routes
    }

    /// Returns all registered WebSocket gateways.
    #[must_use]
    pub fn ws_gateways(&self) -> &[WsGatewayDefinition] {
        &self.ws_gateways
    }

    /// Registers a WebSocket gateway.
    #[must_use]
    pub fn ws_gateway(mut self, gateway: WsGatewayDefinition) -> Self {
        let mut gateways = self.ws_gateways.to_vec();
        gateways.push(gateway);
        self.ws_gateways = gateways.into();
        self
    }

    /// Registers global middleware before controller and route middleware.
    #[must_use]
    pub fn middleware(mut self, middleware: impl Middleware) -> Self {
        self.pipeline = self.pipeline.middleware(middleware);
        self
    }

    /// Appends pre-erased global middleware from the application builder.
    #[must_use]
    pub fn extend_middleware(mut self, middlewares: Vec<Arc<dyn Middleware>>) -> Self {
        self.pipeline.middleware.extend(middlewares);
        self
    }

    /// Registers a global guard before controller and route guards.
    #[must_use]
    pub fn guard(mut self, guard: impl Guard) -> Self {
        self.pipeline = self.pipeline.guard(guard);
        self
    }

    /// Registers a global interceptor outside controller and route interceptors.
    #[must_use]
    pub fn interceptor(mut self, interceptor: impl Interceptor) -> Self {
        self.pipeline = self.pipeline.interceptor(interceptor);
        self
    }

    /// Registers a global exception filter that runs if route filters do not handle the error.
    #[must_use]
    pub fn exception_filter(mut self, filter: Arc<dyn crate::ExceptionFilter>) -> Self {
        self.pipeline = self.pipeline.exception_filter(filter);
        self
    }

    /// Registers a global pipe applied to every parameter of every route.
    /// Global pipes execute before controller and route-level pipes.
    #[must_use]
    pub fn pipe(mut self, pipe: &Arc<dyn ParameterPipe>) -> Self {
        let mut routes = self.routes.to_vec();
        for route in &mut routes {
            for param in &mut route.parameters {
                param.pipes.insert(0, Arc::clone(pipe));
            }
        }
        self.routes = routes.into();
        self
    }

    /// Executes one compiled route through the complete framework pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] from middleware, guards, extraction, pipes, interceptors, controller
    /// resolution, or the handler.
    pub async fn execute(
        &self,
        route: &CompiledRoute,
        context: &mut RequestContext,
    ) -> Result<FrameworkResponse, HttpError> {
        if context.extension::<crate::RequestScope>().is_none() {
            context.insert_extension(self.container.request_scope());
        }
        super::pipeline::execute(self, route, context).await
    }

    pub(crate) const fn pipeline(&self) -> &PipelineComponents {
        &self.pipeline
    }
}

impl fmt::Debug for CompiledHttpApplication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledHttpApplication")
            .field("routes", &self.routes)
            .finish_non_exhaustive()
    }
}

/// Compiles routes from all controller definitions.
///
/// # Errors
///
/// Returns [`RouteError`] when any controller contains conflicting routes.
pub fn compile_controller_routes(
    controllers: impl IntoIterator<Item = ControllerDefinition>,
) -> Result<Vec<CompiledRoute>, RouteError> {
    let mut routes = Vec::new();
    let mut seen = HashSet::new();
    for controller in controllers {
        for route in controller.compile_routes()? {
            // Include version metadata in the dedup key so routes at
            // different API versions are allowed to share the same path.
            let version = route
                .metadata
                .get::<VersionMetadata>()
                .map(|v| (v.version.clone(), v.strategy.clone()));
            if !seen.insert((route.method.clone(), route.path.clone(), version)) {
                return Err(RouteError::DuplicateRoute {
                    method: route.method,
                    path: route.path,
                });
            }
            routes.push(route);
        }
    }
    Ok(routes)
}

fn normalize_path(path: &str) -> Result<String, RouteError> {
    if !path.starts_with('/') {
        return Err(RouteError::InvalidPath {
            path: path.to_owned(),
        });
    }
    let normalized = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if normalized.is_empty() {
        Ok("/".to_owned())
    } else {
        Ok(format!("/{normalized}"))
    }
}

fn join_paths(prefix: &str, path: &str) -> String {
    if prefix == "/" {
        return path.to_owned();
    }
    if path == "/" {
        return prefix.to_owned();
    }
    format!("{prefix}{path}")
}

#[cfg(test)]
mod tests {
    use ironic_di::{ProviderDefinition, Scope};

    use super::*;
    use crate::{Json, PathParameter, handler_fn};

    struct Controller;

    fn controller_provider() -> ProviderDefinition {
        ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| Ok(Controller))
    }

    #[test]
    fn normalizes_and_joins_controller_routes() {
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "//:id/",
            "find_one",
            handler_fn(|_controller: Arc<Controller>, mut arguments| async move {
                Ok::<_, HttpError>(Json(arguments.take::<u64>(0).unwrap()))
            }),
        )
        .unwrap()
        .parameter(PathParameter::<u64>::new("id"));
        let controller = ControllerDefinition::new::<Controller>("//users/", controller_provider())
            .unwrap()
            .route(route);

        let routes = compile_controller_routes([controller]).unwrap();
        assert_eq!(routes[0].path(), "/users/:id");
    }

    #[test]
    fn rejects_non_absolute_paths() {
        assert!(matches!(
            normalize_path("users"),
            Err(RouteError::InvalidPath { .. })
        ));
    }

    #[test]
    fn ws_gateway_definition_round_trip() {
        let provider = controller_provider();
        let key = provider.key();
        let gateway = WsGatewayDefinition {
            path: "/ws".to_string(),
            controller: key,
            handler_name: "ChatGateway",
        };
        assert_eq!(gateway.path, "/ws");
        assert_eq!(gateway.handler_name, "ChatGateway");
        assert_eq!(gateway.controller, key);
    }

    #[test]
    fn compiled_app_holds_ws_gateways() {
        let container = ironic_di::ContainerBuilder::new().build();
        let app = CompiledHttpApplication::new(container, Vec::new());
        assert!(app.ws_gateways().is_empty());

        let provider = controller_provider();
        let gateway = WsGatewayDefinition {
            path: "/ws".to_string(),
            controller: provider.key(),
            handler_name: "EchoGateway",
        };
        let app = app.ws_gateway(gateway);
        assert_eq!(app.ws_gateways().len(), 1);
        assert_eq!(app.ws_gateways()[0].path, "/ws");
    }
}
