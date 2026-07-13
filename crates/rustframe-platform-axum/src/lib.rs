#![doc = "Axum integration for RustFrame."]

use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Path, Request},
    response::{IntoResponse, Response},
    routing::{MethodFilter, Route, on},
};
use futures_util::FutureExt;
use rustframe_http::{
    CompiledHttpApplication, CompiledRoute, FrameworkBody, FrameworkRequest, FrameworkResponse,
    HttpError, HttpMethod, HttpStatus, IntoFrameworkResponse, RequestContext,
};
use rustframe_platform::{
    HttpPlatformAdapter, HttpPlatformApplication, PlatformFuture, Shutdown, ShutdownSignal,
};
use tower::{Layer, Service};

/// A failure while converting framework metadata into Axum routes.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum AxumPlatformError {
    /// `RustFrame` emitted an HTTP method unsupported by the initial adapter.
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

/// Builds an Axum router from a compiled `RustFrame` HTTP application.
pub struct AxumAdapter {
    request_body_limit: usize,
    request_timeout: Duration,
    configure_router: Vec<RouterConfigurator>,
}

type RouterConfigurator = Box<dyn FnOnce(Router) -> Router + Send + 'static>;

/// Default maximum buffered request body: 1 MiB.
pub const DEFAULT_REQUEST_BODY_LIMIT: usize = 1024 * 1024;
/// Default end-to-end request timeout: 30 seconds.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

impl AxumAdapter {
    /// Creates an adapter with a 1 MiB request body limit.
    #[must_use]
    pub fn new() -> Self {
        Self {
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            configure_router: Vec::new(),
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

    /// Applies native Axum router configuration after framework routes are registered.
    ///
    /// Native routes receive native Tower layers but do not automatically participate in the
    /// `RustFrame` request pipeline.
    #[must_use]
    pub fn configure_router(
        mut self,
        configure: impl FnOnce(Router) -> Router + Send + 'static,
    ) -> Self {
        self.configure_router.push(Box::new(configure));
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
        for configure in self.configure_router {
            router = configure(router);
        }
        Ok(AxumApplication { router })
    }
}

/// A built Axum application that can be tested, extended, or served.
#[derive(Clone, Debug)]
pub struct AxumApplication {
    router: Router,
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
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
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
            let (sender, receiver) = tokio::sync::oneshot::channel();
            let graceful = async move {
                let signal = shutdown.wait().await;
                let _ = sender.send(signal);
            };
            axum::serve(listener, self.router)
                .with_graceful_shutdown(graceful)
                .await
                .map_err(|error| AxumPlatformError::Serve {
                    message: error.to_string(),
                })?;
            Ok(receiver.await.unwrap_or(ShutdownSignal::Custom("platform")))
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
    let native_path = native_path(route.path());
    let route = Arc::new(route);
    let handler = move |Path(parameters): Path<HashMap<String, String>>, request: Request| {
        let application = Arc::clone(&application);
        let route = Arc::clone(&route);
        async move {
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

async fn execute_route(
    application: Arc<CompiledHttpApplication>,
    route: Arc<CompiledRoute>,
    parameters: HashMap<String, String>,
    request: Request,
    body_limit: usize,
) -> Response {
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
    let request = FrameworkRequest::new(parts.method, parts.uri, parts.headers, body)
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

fn framework_response(response: FrameworkResponse) -> Response {
    let (status, headers, body) = response.into_parts();
    let body = match body {
        FrameworkBody::Empty => Body::empty(),
        FrameworkBody::Bytes(bytes) => Body::from(bytes),
        _ => Body::from(Vec::<u8>::new()),
    };
    let mut response = Response::new(body);
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

fn error_response(error: HttpError) -> Response {
    match error.into_framework_response() {
        Ok(response) => framework_response(response),
        Err(_) => framework_response(FrameworkResponse::empty(HttpStatus::INTERNAL_SERVER_ERROR)),
    }
}

fn method_filter(method: &HttpMethod) -> Result<MethodFilter, AxumPlatformError> {
    let filter = match *method {
        HttpMethod::GET => MethodFilter::GET,
        HttpMethod::POST => MethodFilter::POST,
        HttpMethod::PUT => MethodFilter::PUT,
        HttpMethod::PATCH => MethodFilter::PATCH,
        HttpMethod::DELETE => MethodFilter::DELETE,
        HttpMethod::HEAD => MethodFilter::HEAD,
        HttpMethod::OPTIONS => MethodFilter::OPTIONS,
        _ => {
            return Err(AxumPlatformError::UnsupportedMethod {
                method: method.clone(),
            });
        }
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
    use rustframe_di::{ContainerBuilder, Dependency, ProviderDefinition, ResolveError, Scope};
    use rustframe_http::{
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

    async fn request(path: &str) -> Response {
        let application = AxumAdapter::new().build(application()).unwrap();
        application
            .into_router()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
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
            .oneshot(Request::get("/native").body(Body::empty()).unwrap())
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
                    .body(Body::from("too large"))
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
            .oneshot(Request::get("/slow").body(Body::empty()).unwrap())
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
            .oneshot(Request::get("/panic").body(Body::empty()).unwrap())
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
}
