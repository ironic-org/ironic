//! Integration coverage for the public testing utilities.

use std::sync::{Arc, Mutex};

use rustframe_core::{
    HealthModule, LifecycleDefinition, LifecycleFuture, Module, ModuleDefinition,
    OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
};
use rustframe_di::{Dependency, ProviderDefinition, Scope};
use rustframe_http::{
    ControllerDefinition, HeaderParameter, HttpError, HttpMethod, Json, JsonBody, PathParameter,
    QueryParameters, RouteDefinition, handler_fn,
};
use rustframe_platform::ShutdownSignal;
use rustframe_testing::{TestApplication, TestModule};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct UsersService {
    name: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct UserView {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateUser {
    id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserQuery {
    suffix: String,
}

struct UsersController {
    users: Arc<UsersService>,
}

fn controller() -> ControllerDefinition {
    let provider = ProviderDefinition::factory(
        Scope::Singleton,
        vec![Dependency::required::<UsersService>()],
        |resolver| async move {
            Ok(UsersController {
                users: resolver.resolve().await?,
            })
        },
    );
    let find = RouteDefinition::new(
        HttpMethod::GET,
        "/:id",
        "find",
        handler_fn(
            |controller: Arc<UsersController>, mut arguments| async move {
                let id = arguments.take::<u64>(0)?;
                let suffix = arguments.take::<UserQuery>(1)?;
                let request_id = arguments.take::<String>(2)?;
                Ok::<_, HttpError>(Json(UserView {
                    id,
                    name: format!("{}-{}-{request_id}", controller.users.name, suffix.suffix),
                }))
            },
        ),
    )
    .unwrap()
    .parameter(PathParameter::<u64>::new("id"))
    .parameter(QueryParameters::<UserQuery>::new())
    .parameter(HeaderParameter::<String>::new("x-request-id"));
    let create = RouteDefinition::new(
        HttpMethod::POST,
        "/",
        "create",
        handler_fn(
            |controller: Arc<UsersController>, mut arguments| async move {
                let request = arguments.take::<CreateUser>(0)?;
                Ok::<_, HttpError>(Json(UserView {
                    id: request.id,
                    name: controller.users.name.to_owned(),
                }))
            },
        ),
    )
    .unwrap()
    .parameter(JsonBody::<CreateUser>::new());

    ControllerDefinition::new::<UsersController>("/users", provider)
        .unwrap()
        .route(find)
        .route(create)
}

struct UsersModule;

impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::value(UsersService { name: "real" }))
            .controller(controller())
            .build()
    }
}

#[tokio::test]
async fn test_module_supports_definition_value_and_factory_overrides() {
    let definition = TestModule::builder::<UsersModule>()
        .override_provider(ProviderDefinition::value(UsersService {
            name: "definition",
        }))
        .compile()
        .await
        .unwrap();
    assert_eq!(
        definition.resolve::<UsersService>().await.unwrap().name,
        "definition"
    );

    let value = TestModule::builder::<UsersModule>()
        .override_value(UsersService { name: "value" })
        .compile()
        .await
        .unwrap();
    assert_eq!(value.resolve::<UsersService>().await.unwrap().name, "value");

    let factory = TestModule::builder::<UsersModule>()
        .override_factory(Scope::Transient, Vec::new(), |_resolver| async {
            Ok(UsersService { name: "factory" })
        })
        .compile()
        .await
        .unwrap();
    assert_eq!(
        factory.resolve::<UsersService>().await.unwrap().name,
        "factory"
    );
}

#[tokio::test]
async fn test_application_dispatches_without_a_socket_and_asserts_responses() {
    let application = TestApplication::builder::<UsersModule>()
        .override_value(UsersService { name: "mock" })
        .build()
        .await
        .unwrap();

    let response = application
        .get("/users/7")
        .query(&UserQuery {
            suffix: "query".to_owned(),
        })
        .header("x-request-id", "abc")
        .send()
        .await;
    response.assert_status(200);
    response.assert_header("content-type", "application/json");
    response.assert_json(&UserView {
        id: 7,
        name: "mock-query-abc".to_owned(),
    });
    assert_eq!(response.json::<UserView>().unwrap().id, 7);

    let response = application
        .post("/users")
        .json(&CreateUser { id: 9 })
        .send()
        .await;
    response.assert_status(200);
    response.assert_json(&UserView {
        id: 9,
        name: "mock".to_owned(),
    });

    let response = application.get("/missing").send().await;
    response.assert_status(404);
    response.assert_error("RF_HTTP_ROUTE_NOT_FOUND");

    application.shutdown().await.unwrap();
}

#[tokio::test]
async fn built_in_health_and_request_correlation_are_enabled() {
    let application = TestApplication::new::<HealthModule>().await.unwrap();

    let response = application
        .get("/health")
        .header("x-request-id", "test-request")
        .send()
        .await;
    response.assert_status(200);
    response.assert_header("x-request-id", "test-request");
    response.assert_json(&serde_json::json!({"status": "ok"}));

    application.shutdown().await.unwrap();
}

type Events = Arc<Mutex<Vec<&'static str>>>;

struct LifecycleService {
    events: Events,
}

impl OnModuleInit for LifecycleService {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().unwrap().push("init");
            Ok(())
        })
    }
}

impl OnApplicationBootstrap for LifecycleService {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().unwrap().push("bootstrap");
            Ok(())
        })
    }
}

impl OnApplicationShutdown for LifecycleService {
    fn on_application_shutdown(&self, _signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().unwrap().push("shutdown");
            Ok(())
        })
    }
}

impl OnModuleDestroy for LifecycleService {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.events.lock().unwrap().push("destroy");
            Ok(())
        })
    }
}

struct LifecycleModule;

impl Module for LifecycleModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::constructor(
                Scope::Singleton,
                Vec::new(),
                |_resolver| {
                    Ok(LifecycleService {
                        events: Arc::new(Mutex::new(Vec::new())),
                    })
                },
            ))
            .lifecycle(
                LifecycleDefinition::builder::<LifecycleService>()
                    .module_init()
                    .application_bootstrap()
                    .application_shutdown()
                    .module_destroy()
                    .build(),
            )
            .build()
    }
}

#[tokio::test]
async fn shutdown_runs_lifecycle_cleanup() {
    let application = TestApplication::new::<LifecycleModule>().await.unwrap();
    let events = Arc::clone(
        &application
            .resolve::<LifecycleService>()
            .await
            .unwrap()
            .events,
    );
    assert_eq!(*events.lock().unwrap(), ["init", "bootstrap"]);

    application.shutdown().await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        ["init", "bootstrap", "shutdown", "destroy"]
    );
}

#[tokio::test]
async fn dropping_an_application_also_runs_lifecycle_cleanup() {
    let application = TestApplication::new::<LifecycleModule>().await.unwrap();
    let events = Arc::clone(
        &application
            .resolve::<LifecycleService>()
            .await
            .unwrap()
            .events,
    );

    drop(application);
    assert_eq!(
        *events.lock().unwrap(),
        ["init", "bootstrap", "shutdown", "destroy"]
    );
}
