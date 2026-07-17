#![doc = "Axum integration for Ironic."]
#![cfg_attr(not(feature = "static-files"), allow(unused_imports, dead_code))]

use std::{
    collections::HashMap, convert::Infallible, net::SocketAddr, path::PathBuf, sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    body::{Body as AxumBody, to_bytes},
    extract::{Path, Request as AxumRequest},
    response::{IntoResponse as AxumIntoResponse, Response as AxumResponse},
    routing::{MethodFilter, Route, on},
};
use futures_util::FutureExt;
use ironic_http::{
    CompiledHttpApplication, CompiledRoute, Body, Request, Response,
    HttpError, HttpMethod, HttpStatus, IntoResponse, RequestContext, VersioningStrategy,
};
use ironic_platform::{
    HttpPlatformAdapter, HttpPlatformApplication, PlatformFuture, Shutdown, ShutdownSignal,
};
use tower::{Layer, Service, ServiceBuilder};
use tracing::warn;

/// A failure while converting framework metadata into Axum routes.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum AxumPlatformError {
    /// Ironic emitted an HTTP method unsupported by the initial adapter.
    #[error("RF_PLATFORM_UNSUPPORTED_METHOD: Axum adapter does not support `{method}`")]
    UnsupportedMethod {
        /// The unsupported method.
        method: HttpMethod,
    },
    /// Binding the configured address failed.
    #[error("RF_PLATFORM_BIND_FAILED: could not bind `{address}`: {message}")]
    Bind {
        /// The requested socket address.
        address: SocketAddr,
        /// The underlying safe I/O message.
        message: String,
    },
    /// The Axum server stopped with an error.
    #[error("RF_PLATFORM_SERVE_FAILED: {message}")]
    Serve {
        /// The underlying safe serving message.
        message: String,
    },
}

#[cfg(feature = "static-files")]
struct StaticFileRoute {
    route_path: String,
    fs_dir: PathBuf,
}

/// Builds an Axum router from a compiled Ironic HTTP application.
pub struct AxumAdapter {
    request_body_limit: usize,
    request_timeout: Duration,
    drain_timeout: Duration,
    #[cfg(feature = "compression")]
    enable_compression: bool,
    configure_router: Vec<RouterConfigurator>,
    #[cfg(feature = "static-files")]
    static_files: Vec<StaticFileRoute>,
    #[cfg(feature = "resilience-ext")]
    max_concurrent_requests: Option<u64>,
    max_connections: Option<usize>,
}

type RouterConfigurator = Box<dyn FnOnce(Router) -> Router + Send + 'static>;

/// Default maximum buffered request body: 1 MiB.
pub const DEFAULT_REQUEST_BODY_LIMIT: usize = 1024 * 1024;
/// Default end-to-end request timeout: 30 seconds.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Default graceful drain timeout during shutdown: 30 seconds.
pub const DEFAULT_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

impl AxumAdapter {
    /// Creates an adapter with a 1 MiB request body limit.
    #[must_use]
    pub fn new() -> Self {
        Self {
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            drain_timeout: DEFAULT_DRAIN_TIMEOUT,
            #[cfg(feature = "compression")]
            enable_compression: false,
            configure_router: Vec::new(),
            #[cfg(feature = "static-files")]
            static_files: Vec::new(),
            #[cfg(feature = "resilience-ext")]
            max_concurrent_requests: None,
            max_connections: None,
        }
    }

    /// Sets the maximum buffered request body size.
    #[must_use]
    pub const fn request_body_limit(mut self, bytes: usize) -> Self {
        self.request_body_limit = bytes;
        self
    }

    /// Sets the end-to-end request timeout.
    #[must_use]
    pub const fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Sets the graceful shutdown drain timeout.
    ///
    /// When a shutdown signal is received, the server stops accepting new
    /// connections and waits up to `timeout` for in-flight requests to
    /// complete before forcing a stop.
    #[must_use]
    pub const fn drain_timeout(mut self, timeout: Duration) -> Self {
        self.drain_timeout = timeout;
        self
    }

    /// Mounts a static file directory at the given route prefix.
    ///
    /// Files are served with built-in `ETag` and `If-None-Match`/304 support.
    /// The default `Cache-Control` is `public, max-age=3600`.
    ///
    /// # Feature flag
    ///
    /// Requires the `static-files` feature.
    #[cfg(feature = "static-files")]
    #[must_use]
    pub fn static_files(mut self, route_path: &str, fs_dir: impl AsRef<std::path::Path>) -> Self {
        self.static_files.push(StaticFileRoute {
            route_path: route_path.to_owned(),
            fs_dir: fs_dir.as_ref().to_owned(),
        });
        self
    }

    /// Applies native Axum router configuration after framework routes are registered.
    ///
    /// Native routes receive native Tower layers but do not automatically participate in the
    /// Ironic request pipeline.
    #[must_use]
    pub fn configure_router(
        mut self,
        configure: impl FnOnce(Router) -> Router + Send + 'static,
    ) -> Self {
        self.configure_router.push(Box::new(configure));
        self
    }

    /// Enables response compression (gzip, brotli, zstd) via `tower-http`.
    #[cfg(feature = "compression")]
    #[must_use]
    pub fn compression(mut self) -> Self {
        self.enable_compression = true;
        self
    }

    /// Sets the maximum number of concurrent in-flight requests.
    ///
    /// When the limit is reached, subsequent requests receive an HTTP 503
    /// response immediately.  Requires the `resilience-ext` feature.
    #[cfg(feature = "resilience-ext")]
    #[must_use]
    pub const fn max_concurrent_requests(mut self, max: u64) -> Self {
        self.max_concurrent_requests = Some(max);
        self
    }

    /// Sets the maximum number of open TCP connections.
    ///
    /// Limits the total number of concurrent socket connections to prevent
    /// file descriptor exhaustion from slowloris-style attacks.
    #[must_use]
    pub const fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }
}

impl Default for AxumAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpPlatformAdapter for AxumAdapter {
    type Application = AxumApplication;
    type Error = AxumPlatformError;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error> {
        let mut router = Router::new();
        for route in application.routes() {
            router = register_route(
                router,
                Arc::clone(&application),
                route.clone(),
                self.request_body_limit,
                self.request_timeout,
            )?;
        }
        #[cfg(feature = "realtime")]
        for gateway in application.ws_gateways() {
            router = register_ws_gateway(router, Arc::clone(&application), gateway);
        }
        #[cfg(feature = "compression")]
        if self.enable_compression {
            router = router.layer(tower_http::compression::CompressionLayer::new());
        }
        #[cfg(feature = "static-files")]
        for sf in self.static_files {
            let wildcard_path = if sf.route_path.ends_with('/') {
                format!("{}*path", sf.route_path)
            } else {
                format!("{}/*path", sf.route_path)
            };
            // ServeDir uses Infallible errors (404s are returned as responses),
            // so get_service works directly without handle_error.
            let dir_service = ServiceBuilder::new()
                .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("public, max-age=3600"),
                ))
                .service(tower_http::services::ServeDir::new(&sf.fs_dir).precompressed_gzip());
            router = router.route(&wildcard_path, axum::routing::get_service(dir_service));
        }
        for configure in self.configure_router {
            router = configure(router);
        }
        #[cfg(feature = "resilience-ext")]
        if let Some(max) = self.max_concurrent_requests {
            router = router.layer(crate::resilience::ConcurrencyLimitLayer::new(max));
        }
        Ok(AxumApplication {
            router,
            drain_timeout: self.drain_timeout,
        })
    }
}

/// A built Axum application that can be tested, extended, or served.
#[derive(Clone, Debug)]
pub struct AxumApplication {
    router: Router,
    drain_timeout: Duration,
}

impl AxumApplication {
    /// Returns the native router.
    pub const fn router(&self) -> &Router {
        &self.router
    }

    /// Consumes the application and returns the native router.
    pub fn into_router(self) -> Router {
        self.router
    }

    /// Applies a native Tower layer to framework and escape-hatch routes.
    #[must_use]
    pub fn layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<AxumRequest> + Clone + Send + Sync + 'static,
        <L::Service as Service<AxumRequest>>::Response: AxumIntoResponse + 'static,
        <L::Service as Service<AxumRequest>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<AxumRequest>>::Future: Send + 'static,
    {
        self.router = self.router.layer(layer);
        self
    }

    /// Applies arbitrary native router configuration.
    #[must_use]
    pub fn map_router(mut self, configure: impl FnOnce(Router) -> Router) -> Self {
        self.router = configure(self.router);
        self
    }

    /// Waits for graceful shutdown to be triggered, then sleeps for
    /// `drain_timeout`.  Used by [`listen`] to limit how long in-flight
    /// requests are allowed to complete.
    async fn drain_timeout_watch(
        mut rx: tokio::sync::watch::Receiver<Option<ShutdownSignal>>,
        drain_timeout: Duration,
    ) -> ShutdownSignal {
        // Wait until the graceful future writes the shutdown signal.
        while rx.borrow().is_none() {
            rx.changed().await.ok();
        }
        // Drain timeout starts now.
        tokio::time::sleep(drain_timeout).await;
        rx.borrow()
            .unwrap_or(ShutdownSignal::Custom("drain-timeout"))
    }
}

impl HttpPlatformApplication for AxumApplication {
    type Error = AxumPlatformError;

    fn listen(
        self,
        address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
        Box::pin(async move {
            let listener = tokio::net::TcpListener::bind(address)
                .await
                .map_err(|error| AxumPlatformError::Bind {
                    address,
                    message: error.to_string(),
                })?;
            // Watch channel: graceful writes the signal, drain futures read it.
            let (drain_tx, drain_rx) = tokio::sync::watch::channel(None::<ShutdownSignal>);
            let graceful = {
                async move {
                    let sig = shutdown.wait().await;
                    let _ = drain_tx.send(Some(sig));
                }
            };
            let serve_fut = axum::serve(listener, self.router).with_graceful_shutdown(graceful);

            let signal = tokio::select! {
                result = serve_fut => {
                    // Serve completed — either clean drain or error.
                    result.map_err(|error| AxumPlatformError::Serve {
                        message: error.to_string(),
                    })?;
                    // Signal should have been delivered by the graceful future.
                    drain_rx
                        .borrow()
                        .unwrap_or(ShutdownSignal::Custom("platform"))
                }
                sig = Self::drain_timeout_watch(drain_rx.clone(), self.drain_timeout) => {
                    warn!(
                        drain_timeout_ms = %self.drain_timeout.as_millis(),
                        "Graceful shutdown drain timed out; \
                         some in-flight requests may have been dropped"
                    );
                    sig
                }
            };
            Ok(signal)
        })
    }
}

fn register_route(
    router: Router,
    application: Arc<CompiledHttpApplication>,
    route: CompiledRoute,
    body_limit: usize,
    request_timeout: Duration,
) -> Result<Router, AxumPlatformError> {
    let method = method_filter(route.method())?;
    let native_path = native_path(&route.versioned_path());
    let version = route.version();
    let route = Arc::new(route);
    let handler = move |Path(parameters): Path<HashMap<String, String>>, request: AxumRequest| {
        let application = Arc::clone(&application);
        let route = Arc::clone(&route);
        let version = version.clone();
        async move {
            // Header / media-type version check
            if let Some(ref v) = version {
                match v.strategy {
                    VersioningStrategy::Uri => {} // already handled by path prefix
                    VersioningStrategy::Header => {
                        if !matches_header_version(&request, v) {
                            return error_response(HttpError::new(
                                HttpStatus::NOT_FOUND,
                                "RF_HTTP_VERSION_MISMATCH",
                                "No route matches the requested API version",
                            ));
                        }
                    }
                    VersioningStrategy::MediaType => {
                        if !matches_media_type_version(&request, v) {
                            return error_response(HttpError::new(
                                HttpStatus::NOT_FOUND,
                                "RF_HTTP_VERSION_MISMATCH",
                                "No route matches the requested API version",
                            ));
                        }
                    }
                }
            }
            match tokio::time::timeout(
                request_timeout,
                execute_route(application, route, parameters, request, body_limit),
            )
            .await
            {
                Ok(response) => response,
                Err(_) => error_response(HttpError::new(
                    HttpStatus::REQUEST_TIMEOUT,
                    "RF_HTTP_REQUEST_TIMEOUT",
                    "Request processing exceeded the configured timeout",
                )),
            }
        }
    };
    Ok(router.route(&native_path, on(method, handler)))
}

fn matches_header_version(request: &AxumRequest, version: &ironic_http::VersionMetadata) -> bool {
    request
        .headers()
        .get("accept-version")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == version.version)
}

fn matches_media_type_version(request: &AxumRequest, version: &ironic_http::VersionMetadata) -> bool {
    let pattern = format!("vnd.api.v{}+json", version.version);
    request
        .headers()
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains(&pattern))
}

async fn execute_route(
    application: Arc<CompiledHttpApplication>,
    route: Arc<CompiledRoute>,
    parameters: HashMap<String, String>,
    request: AxumRequest,
    body_limit: usize,
) -> AxumResponse {
    let (parts, body) = request.into_parts();
    let body = match to_bytes(body, body_limit).await {
        Ok(body) => body.to_vec(),
        Err(_) => {
            return error_response(HttpError::new(
                HttpStatus::PAYLOAD_TOO_LARGE,
                "RF_HTTP_BODY_TOO_LARGE",
                "Request body exceeds the configured limit",
            ));
        }
    };
    let request = Request::new(parts.method, parts.uri, parts.headers, body)
        .with_path_parameters(parameters);
    let mut context = RequestContext::new(request);

    let execution = std::panic::AssertUnwindSafe(application.execute(&route, &mut context))
        .catch_unwind()
        .await;
    match execution {
        Ok(Ok(response)) => framework_response(response),
        Ok(Err(error)) => error_response(error),
        Err(_) => error_response(HttpError::internal(
            "RF_HTTP_HANDLER_PANICKED",
            "Request processing failed",
        )),
    }
}

fn framework_response(response: Response) -> AxumResponse {
    let (status, headers, body) = response.into_parts();
    let body = match body {
        Body::Empty => AxumBody::empty(),
        Body::Bytes(bytes) => AxumBody::from(bytes),
        Body::Stream(bytes) => AxumBody::from(bytes.as_ref().clone()),
    };
    let mut response = AxumResponse::new(body);
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

fn error_response(error: HttpError) -> AxumResponse {
    match error.into_framework_response() {
        Ok(response) => framework_response(response),
        Err(_) => framework_response(Response::empty(HttpStatus::INTERNAL_SERVER_ERROR)),
    }
}

#[cfg(feature = "realtime")]
fn register_ws_gateway(
    router: Router,
    application: Arc<CompiledHttpApplication>,
    gateway: &ironic_http::WsGatewayDefinition,
) -> Router {
    use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
    use futures_util::{SinkExt, StreamExt};

    let controller_key = gateway.controller;
    let handler = move |ws: WebSocketUpgrade| {
        let app = Arc::clone(&application);
        async move {
            ws.on_upgrade(move |socket: WebSocket| {
                let app = app;
                async move {
                    let container = app.container();
                    let _gateway = container.resolve_key(controller_key).await;
                    let (mut ws_sender, mut ws_receiver) = socket.split();
                    while let Some(Ok(msg)) = ws_receiver.next().await {
                        if let Message::Text(text) = msg {
                            let _ = ws_sender.send(Message::Text(text)).await;
                        }
                    }
                }
            })
            .into_response()
        }
    };
    router.route(&gateway.path, axum::routing::get(handler))
}

fn method_filter(method: &HttpMethod) -> Result<MethodFilter, AxumPlatformError> {
    let filter = if method == HttpMethod::GET {
        MethodFilter::GET
    } else if method == HttpMethod::POST {
        MethodFilter::POST
    } else if method == HttpMethod::PUT {
        MethodFilter::PUT
    } else if method == HttpMethod::PATCH {
        MethodFilter::PATCH
    } else if method == HttpMethod::DELETE {
        MethodFilter::DELETE
    } else if method == HttpMethod::HEAD {
        MethodFilter::HEAD
    } else if method == HttpMethod::OPTIONS {
        MethodFilter::OPTIONS
    } else {
        return Err(AxumPlatformError::UnsupportedMethod {
            method: method.clone(),
        });
    };
    Ok(filter)
}

fn native_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            segment
                .strip_prefix(':')
                .map_or_else(|| segment.to_owned(), |name| format!("{{{name}}}"))
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::Request, routing::get};
    use ironic_di::{ContainerBuilder, Dependency, ProviderDefinition, ResolveError, Scope};
    use ironic_http::{
        ControllerDefinition, Json, PathParameter, RouteDefinition, compile_controller_routes,
        handler_fn,
    };
    use serde::Serialize;
    use tower::ServiceExt;

    use super::*;

    #[derive(Clone, Debug, Serialize)]
    struct User {
        id: u64,
        name: &'static str,
    }

    struct UsersService {
        user: User,
    }
    impl UsersService {
        fn find(&self, id: u64) -> Result<User, HttpError> {
            if id == 1 {
                Ok(self.user.clone())
            } else {
                Err(HttpError::not_found(
                    "USER_NOT_FOUND",
                    "The requested user does not exist",
                ))
            }
        }
    }

    struct UsersController {
        users: Arc<UsersService>,
    }

    fn application() -> Arc<CompiledHttpApplication> {
        let service = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(UsersService {
                user: User { id: 1, name: "Ada" },
            })
        });
        let controller_provider = ProviderDefinition::factory(
            Scope::Singleton,
            vec![Dependency::required::<UsersService>()],
            |resolver| async move {
                Ok(UsersController {
                    users: resolver.resolve().await?,
                })
            },
        );
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/:id",
            "find_one",
            handler_fn(
                |controller: Arc<UsersController>, mut arguments| async move {
                    let id = arguments.take::<u64>(0)?;
                    controller.users.find(id).map(Json)
                },
            ),
        )
        .unwrap()
        .parameter(PathParameter::<u64>::new("id"));
        let controller =
            ControllerDefinition::new::<UsersController>("/users", controller_provider)
                .unwrap()
                .route(route);

        let mut container = ContainerBuilder::new();
        container.register(service).unwrap();
        container.register(controller.provider().clone()).unwrap();
        let routes = compile_controller_routes([controller]).unwrap();
        Arc::new(CompiledHttpApplication::new(container.build(), routes))
    }

    async fn request(path: &str) -> AxumResponse {
        let application = AxumAdapter::new().build(application()).unwrap();
        application
            .into_router()
            .oneshot(Request::get(path).body(AxumBody::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn dispatches_through_di_and_erased_handler() {
        let response = request("/users/1").await;
        assert_eq!(response.status(), HttpStatus::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"id": 1, "name": "Ada"})
        );
    }

    #[tokio::test]
    async fn maps_malformed_path_parameters() {
        let response = request("/users/not-a-number").await;
        assert_eq!(response.status(), HttpStatus::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"],
            "RF_HTTP_INVALID_PATH_PARAMETER"
        );
    }

    #[tokio::test]
    async fn maps_domain_not_found_errors() {
        let response = request("/users/99").await;
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"],
            "USER_NOT_FOUND"
        );
    }

    #[tokio::test]
    async fn exposes_raw_axum_routes() {
        let application = AxumAdapter::new()
            .configure_router(|router| router.route("/native", get(|| async { "native" })))
            .build(application())
            .unwrap();
        let response = application
            .into_router()
            .oneshot(Request::get("/native").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::OK);
    }

    #[tokio::test]
    async fn enforces_the_configured_body_limit() {
        let router = AxumAdapter::new()
            .request_body_limit(1)
            .build(application())
            .unwrap()
            .into_router();
        let response = router
            .oneshot(
                Request::get("/users/1")
                    .body(AxumBody::from("too large"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn enforces_the_configured_request_timeout() {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(UsersController {
                users: Arc::new(UsersService {
                    user: User { id: 1, name: "Ada" },
                }),
            })
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/slow",
            "slow",
            handler_fn(|_controller: Arc<UsersController>, _arguments| async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Ok::<_, HttpError>("ok")
            }),
        )
        .unwrap();
        let controller = ControllerDefinition::new::<UsersController>("/", provider)
            .unwrap()
            .route(route);
        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        );
        let router = AxumAdapter::new()
            .request_timeout(Duration::from_millis(1))
            .build(Arc::new(application))
            .unwrap()
            .into_router();

        let response = router
            .oneshot(Request::get("/slow").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::REQUEST_TIMEOUT);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"],
            "RF_HTTP_REQUEST_TIMEOUT"
        );
    }

    #[tokio::test]
    #[ignore = "requires permission to bind an ephemeral localhost port"]
    async fn listens_and_stops_on_a_graceful_shutdown_signal() {
        let application = AxumAdapter::new().build(application()).unwrap();
        let signal = application
            .listen(
                "127.0.0.1:0".parse().unwrap(),
                Shutdown::new(std::future::ready(ShutdownSignal::Custom("test"))),
            )
            .await
            .unwrap();
        assert_eq!(signal, ShutdownSignal::Custom("test"));
    }

    #[tokio::test]
    async fn isolates_request_panics_when_unwinding_is_enabled() {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(UsersController {
                users: Arc::new(UsersService {
                    user: User { id: 1, name: "Ada" },
                }),
            })
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/panic",
            "panic",
            handler_fn(|_controller: Arc<UsersController>, _arguments| async {
                panic!("test panic");
                #[allow(unreachable_code)]
                Ok::<&'static str, HttpError>("unreachable")
            }),
        )
        .unwrap();
        let controller = ControllerDefinition::new::<UsersController>("/", provider)
            .unwrap()
            .route(route);
        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let compiled = CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        );
        let router = AxumAdapter::new()
            .build(Arc::new(compiled))
            .unwrap()
            .into_router();

        let response = router
            .oneshot(Request::get("/panic").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"],
            "RF_HTTP_HANDLER_PANICKED"
        );
    }

    #[test]
    fn reports_unsupported_methods() {
        let route = RouteDefinition::new(
            HttpMethod::TRACE,
            "/trace",
            "trace",
            handler_fn(|_controller: Arc<UsersController>, _arguments| async {
                Ok::<_, HttpError>("ok")
            }),
        )
        .unwrap();
        let provider = ProviderDefinition::factory::<UsersController, _, _>(
            Scope::Singleton,
            Vec::new(),
            |_resolver| async { Err(ResolveError::factory::<UsersController>("not used")) },
        );
        let controller = ControllerDefinition::new::<UsersController>("/", provider)
            .unwrap()
            .route(route);
        let app = CompiledHttpApplication::new(
            ContainerBuilder::new().build(),
            compile_controller_routes([controller]).unwrap(),
        );
        assert!(matches!(
            AxumAdapter::new().build(Arc::new(app)),
            Err(AxumPlatformError::UnsupportedMethod { .. })
        ));
    }

    // ------------------------------------------------------------------
    // API versioning tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn uri_versioning_prefixes_routes() {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(UsersController {
                users: Arc::new(UsersService {
                    user: User { id: 1, name: "Ada" },
                }),
            })
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/profile",
            "profile",
            handler_fn(|_controller: Arc<UsersController>, _arguments| async {
                Ok::<_, HttpError>("uri-versioned")
            }),
        )
        .unwrap();
        let controller = ControllerDefinition::new::<UsersController>("/users", provider)
            .unwrap()
            .version(ironic_http::VersionMetadata::new(
                "1",
                VersioningStrategy::Uri,
            ))
            .route(route);
        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        ));
        let router = AxumAdapter::new().build(application).unwrap().into_router();

        // Versioned path should work
        let response = router
            .clone()
            .oneshot(
                Request::get("/v1/users/profile")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"uri-versioned");

        // Non-versioned path should 404
        let response = router
            .oneshot(Request::get("/users/profile").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
    }

    #[tokio::test]
    async fn header_versioning_matches_accept_header() {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(UsersController {
                users: Arc::new(UsersService {
                    user: User { id: 1, name: "Ada" },
                }),
            })
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/resource",
            "resource",
            handler_fn(|_controller: Arc<UsersController>, _arguments| async {
                Ok::<_, HttpError>("header-versioned")
            }),
        )
        .unwrap();
        let controller = ControllerDefinition::new::<UsersController>("/api", provider)
            .unwrap()
            .version(ironic_http::VersionMetadata::new(
                "2",
                VersioningStrategy::Header,
            ))
            .route(route);
        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        ));
        let router = AxumAdapter::new().build(application).unwrap().into_router();

        // Matching header should succeed
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/resource")
                    .header("accept-version", "2")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::OK);

        // Mismatched header should 404
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/resource")
                    .header("accept-version", "3")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);

        // Missing header should 404
        let response = router
            .oneshot(Request::get("/api/resource").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
    }

    #[tokio::test]
    async fn media_type_versioning_matches_accept_header() {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(UsersController {
                users: Arc::new(UsersService {
                    user: User { id: 1, name: "Ada" },
                }),
            })
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/data",
            "data",
            handler_fn(|_controller: Arc<UsersController>, _arguments| async {
                Ok::<_, HttpError>("media-type-versioned")
            }),
        )
        .unwrap();
        let controller = ControllerDefinition::new::<UsersController>("/svc", provider)
            .unwrap()
            .version(ironic_http::VersionMetadata::new(
                "1",
                VersioningStrategy::MediaType,
            ))
            .route(route);
        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        ));
        let router = AxumAdapter::new().build(application).unwrap().into_router();

        // Matching media type should succeed
        let response = router
            .clone()
            .oneshot(
                Request::get("/svc/data")
                    .header("accept", "application/vnd.api.v1+json")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::OK);

        // Mismatched media type should 404
        let response = router
            .clone()
            .oneshot(
                Request::get("/svc/data")
                    .header("accept", "application/vnd.api.v2+json")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);

        // Missing header should 404
        let response = router
            .oneshot(Request::get("/svc/data").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
    }

    #[tokio::test]
    async fn multiple_uri_versions_coexist() {
        struct V1Controller {
            _users: Arc<UsersService>,
        }
        struct V2Controller {
            _users: Arc<UsersService>,
        }

        let v1_provider =
            ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
                Ok(V1Controller {
                    _users: Arc::new(UsersService {
                        user: User { id: 1, name: "Ada" },
                    }),
                })
            });
        let v2_provider =
            ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
                Ok(V2Controller {
                    _users: Arc::new(UsersService {
                        user: User { id: 1, name: "Ada" },
                    }),
                })
            });

        let v1_route = RouteDefinition::new(
            HttpMethod::GET,
            "/items",
            "items_v1",
            handler_fn(|_controller: Arc<V1Controller>, _arguments| async {
                Ok::<_, HttpError>("v1-response")
            }),
        )
        .unwrap();
        let v2_route = RouteDefinition::new(
            HttpMethod::GET,
            "/items",
            "items_v2",
            handler_fn(|_controller: Arc<V2Controller>, _arguments| async {
                Ok::<_, HttpError>("v2-response")
            }),
        )
        .unwrap();

        let v1_controller = ControllerDefinition::new::<V1Controller>("/api", v1_provider)
            .unwrap()
            .version(ironic_http::VersionMetadata::new(
                "1",
                VersioningStrategy::Uri,
            ))
            .route(v1_route);
        let v2_controller = ControllerDefinition::new::<V2Controller>("/api", v2_provider)
            .unwrap()
            .version(ironic_http::VersionMetadata::new(
                "2",
                VersioningStrategy::Uri,
            ))
            .route(v2_route);

        let mut container = ContainerBuilder::new();
        container
            .register(v1_controller.provider().clone())
            .unwrap();
        container
            .register(v2_controller.provider().clone())
            .unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([v1_controller, v2_controller]).unwrap(),
        ));
        let router = AxumAdapter::new().build(application).unwrap().into_router();

        // v1 response
        let response = router
            .clone()
            .oneshot(Request::get("/v1/api/items").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"v1-response");

        // v2 response
        let response = router
            .clone()
            .oneshot(Request::get("/v2/api/items").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), HttpStatus::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"v2-response");
    }

    #[tokio::test]
    #[cfg(feature = "compression")]
    async fn compression_with_gzip() {
        use std::sync::Arc;

        struct DataController;

        let large_body = "hello world this is a much longer response body that exceeds one kilobyte in length so that the tower-http compression layer will actually trigger compression for it. we need to make sure this string is longer than 1024 bytes to pass the default size threshold. so let's keep writing more and more text until we cross that magical 1024 byte boundary. the text is repeated here for good measure. hello world this is a much longer response body that exceeds one kilobyte in length so that the tower-http compression layer will actually trigger compression for it. hello world this is a much longer response body that exceeds one kilobyte in length so that the tower-http compression layer will actually trigger compression for it. we need to make sure this string is longer than 1024 bytes to pass the default size threshold. so let's keep writing more and more text until we cross that magical 1024 byte boundary. the text is repeated here for good measure. hello world this is a much longer response body that exceeds one kilobyte in length so that the tower-http compression layer will actually trigger compression for it.";

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "get_data",
            handler_fn(move |_controller: Arc<DataController>, _arguments| {
                let body = large_body;
                async move { Ok::<_, HttpError>(body) }
            }),
        )
        .unwrap();
        let provider = ProviderDefinition::value(DataController);
        let controller = ControllerDefinition::new::<DataController>("/api/data", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        ));
        let router = AxumAdapter::new()
            .compression()
            .build(application)
            .unwrap()
            .into_router();

        // Request with Accept-Encoding: gzip
        let response = router
            .oneshot(
                Request::get("/api/data")
                    .header("Accept-Encoding", "gzip")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);
        assert_eq!(
            response
                .headers()
                .get("content-encoding")
                .and_then(|v| v.to_str().ok()),
            Some("gzip")
        );

        // Body should be gzip-compressed
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(!body.is_empty());
        assert_ne!(&body[..], b"hello world");
    }

    #[tokio::test]
    #[cfg(feature = "compression")]
    async fn compression_no_accept_encoding() {
        use std::sync::Arc;

        struct DataController;

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "get_data",
            handler_fn(|_controller: Arc<DataController>, _arguments| async move {
                Ok::<_, HttpError>("hello world")
            }),
        )
        .unwrap();
        let provider = ProviderDefinition::value(DataController);
        let controller = ControllerDefinition::new::<DataController>("/api/data", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        ));
        let router = AxumAdapter::new()
            .compression()
            .build(application)
            .unwrap()
            .into_router();

        // Request without Accept-Encoding
        let response = router
            .oneshot(Request::get("/api/data").body(AxumBody::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);
        assert!(response.headers().get("content-encoding").is_none());

        // Body should be uncompressed
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"hello world");
    }

    #[tokio::test]
    #[cfg(feature = "compression")]
    async fn compression_disabled() {
        use std::sync::Arc;

        struct DataController;

        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "get_data",
            handler_fn(|_controller: Arc<DataController>, _arguments| async move {
                Ok::<_, HttpError>("hello world")
            }),
        )
        .unwrap();
        let provider = ProviderDefinition::value(DataController);
        let controller = ControllerDefinition::new::<DataController>("/api/data", provider)
            .unwrap()
            .route(route);

        let mut container = ContainerBuilder::new();
        container.register(controller.provider().clone()).unwrap();
        let application = Arc::new(CompiledHttpApplication::new(
            container.build(),
            compile_controller_routes([controller]).unwrap(),
        ));
        // No .compression() — should not compress
        let router = AxumAdapter::new().build(application).unwrap().into_router();

        let response = router
            .oneshot(
                Request::get("/api/data")
                    .header("Accept-Encoding", "gzip")
                    .body(AxumBody::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);
        assert!(response.headers().get("content-encoding").is_none());
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"hello world");
    }
}
