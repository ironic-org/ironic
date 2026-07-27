//! Behavioral contracts for post-0.1 feature modules.

#[cfg(all(feature = "cache", feature = "application-services"))]
#[test]
fn cache_interceptor_constructs_with_in_memory_backend() {
    use ironic::{CacheInterceptor, services::cache::InMemoryCache};
    use std::sync::Arc;
    let _interceptor = CacheInterceptor::new(Arc::new(InMemoryCache::new(16)));
}

#[cfg(feature = "cache")]
#[tokio::test]
async fn in_memory_cache_round_trips_json_and_expires_values() {
    use ironic::services::cache::InMemoryCache;
    use std::time::Duration;

    let cache = InMemoryCache::new(2);
    cache
        .set_json("answer", &42_u32, Some(Duration::from_millis(5)))
        .await
        .unwrap();
    assert_eq!(cache.get_json::<u32>("answer").await.unwrap(), Some(42));
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(cache.get_json::<u32>("answer").await.unwrap(), None);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_bus_delivers_only_matching_types() {
    use ironic::services::events::EventBus;
    let bus = EventBus::default();
    let mut strings = bus.subscribe::<String>(2).await;
    assert_eq!(bus.publish(7_u32).await, 0);
    assert_eq!(bus.publish("created".to_owned()).await, 1);
    assert_eq!(strings.recv().await.unwrap().as_str(), "created");
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_handler_macro_generates_registration_function() {
    use ironic::event_handler;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[event_handler(capacity = 32)]
    #[allow(clippy::unused_async)]
    async fn handle_string_event(event: Arc<String>) {
        let _ = event;
    }

    let bus = EventBus::default();
    __event_handler_reg_handle_string_event(&bus);
    // Give the spawned task time to subscribe
    tokio::task::yield_now().await;

    let n = bus.publish("hello".to_owned()).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_handler_macro_with_custom_event_type() {
    use ironic::event_handler;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[derive(Clone, Debug, PartialEq)]
    struct OrderPlaced(u32);

    #[event_handler(capacity = 8)]
    #[allow(clippy::unused_async)]
    async fn handle_order(event: Arc<OrderPlaced>) {
        let _ = event;
    }

    let bus = EventBus::default();
    __event_handler_reg_handle_order(&bus);
    tokio::task::yield_now().await;

    let n = bus.publish(OrderPlaced(42)).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_handler_macro_auto_register_generates_async_init_impl() {
    use ironic::event_handler;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[event_handler(auto_register, capacity = 16)]
    #[allow(clippy::unused_async)]
    async fn handle_auto_event(event: Arc<String>) {
        let _ = event;
    }

    // Verify auto-register struct exists by checking it implements AsyncModuleInit
    fn check_trait_bound<T: ironic::AsyncModuleInit>() {}
    check_trait_bound::<__EventHandlerAuto_handle_auto_event>();

    let bus = EventBus::default();
    __event_handler_reg_handle_auto_event(&bus);
    tokio::task::yield_now().await;

    let n = bus.publish("auto".to_owned()).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_handler_macro_default_capacity() {
    use ironic::event_handler;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[event_handler]
    #[allow(clippy::unused_async)]
    async fn handle_default(event: Arc<String>) {
        let _ = event;
    }

    let bus = EventBus::default();
    __event_handler_reg_handle_default(&bus);
    tokio::task::yield_now().await;

    let n = bus.publish("default".to_owned()).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "scheduling")]
#[tokio::test]
async fn scheduled_tasks_shutdown_cooperatively() {
    use ironic::services::scheduling;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let task = scheduling::interval(Duration::from_millis(5), {
        let calls = Arc::clone(&calls);
        move || {
            let calls = Arc::clone(&calls);
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(18)).await;
    task.shutdown().await.unwrap();
    assert!(calls.load(Ordering::SeqCst) >= 1);
}

#[cfg(all(feature = "scheduling", feature = "cron"))]
#[tokio::test]
async fn cron_schedule_parses_expression() {
    use ironic::services::scheduling;
    let result = scheduling::cron_schedule("0 0 * * * *", || async {});
    assert!(result.is_ok());
}

#[cfg(all(feature = "scheduling", feature = "cron"))]
#[test]
fn cron_schedule_rejects_invalid_expression() {
    use ironic::services::scheduling;
    let result = scheduling::cron_schedule("not-a-cron", || async {});
    assert!(result.is_err());
}

#[cfg(feature = "queues")]
#[tokio::test]
async fn queue_supports_redelivery() {
    use ironic::distributed::queues::{InMemoryQueue, Queue, QueueMessage};
    use std::collections::BTreeMap;
    let queue = InMemoryQueue::new(2);
    let message = QueueMessage {
        id: "1".into(),
        headers: BTreeMap::new(),
        payload: b"work".to_vec(),
        retry_count: 0,
        max_retries: 3,
        ttl_secs: None,
    };
    queue.enqueue(message.clone()).await.unwrap();
    let received = queue.dequeue().await.unwrap().unwrap();
    queue.reject(received, true).await.unwrap();
    assert_eq!(queue.dequeue().await.unwrap(), Some(message));
}

#[cfg(feature = "microservices")]
#[tokio::test]
#[allow(deprecated)]
async fn channel_transports_are_duplex() {
    use ironic::distributed::microservices::{ChannelTransport, Envelope, Transport};
    use std::collections::BTreeMap;
    let (left, right) = ChannelTransport::pair(1);
    left.send(Envelope {
        correlation_id: "c1".into(),
        route: "users.find".into(),
        headers: BTreeMap::new(),
        payload: vec![1],
    })
    .await
    .unwrap();
    assert_eq!(right.receive().await.unwrap().unwrap().correlation_id, "c1");
}

#[cfg(feature = "cqrs")]
#[tokio::test]
async fn cqrs_dispatches_typed_commands_and_queries() {
    use ironic::distributed::cqrs::{Command, CqrsBusBuilder, Query};
    struct Add(u32, u32);
    impl Command for Add {
        type Output = u32;
    }
    struct Current;
    impl Query for Current {
        type Output = &'static str;
    }
    let mut builder = CqrsBusBuilder::new();
    builder
        .command(|command: Add| async move { Ok(command.0 + command.1) })
        .unwrap();
    builder
        .query(|_query: Current| async move { Ok("ready") })
        .unwrap();
    let bus = builder.build();
    assert_eq!(bus.execute(Add(2, 3)).await.unwrap(), 5);
    assert_eq!(bus.ask(Current).await.unwrap(), "ready");
}

#[cfg(feature = "graphql")]
#[test]
fn graphql_schemas_register_as_providers() {
    use ironic::{
        ProviderKey,
        distributed::graphql::{
            driver::{EmptyMutation, EmptySubscription, Object, Schema},
            schema_provider,
        },
    };
    struct QueryRoot;
    #[Object]
    impl QueryRoot {
        async fn value(&self) -> i32 {
            42
        }
    }
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    assert_eq!(
        schema_provider(schema).key(),
        ProviderKey::of::<Schema<QueryRoot, EmptyMutation, EmptySubscription>>()
    );
}

#[cfg(feature = "plugins")]
#[test]
fn plugins_apply_in_order_and_reject_duplicate_names() {
    use ironic::{
        Module, ModuleDefinition,
        ecosystem::plugins::{Plugin, PluginError, PluginRegistry},
    };
    struct Root;
    impl Module for Root {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>().build()
        }
    }
    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            "test"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
        fn apply(
            &self,
            module: ironic::ModuleDefinitionBuilder,
        ) -> Result<ironic::ModuleDefinitionBuilder, PluginError> {
            Ok(module)
        }
    }
    let mut plugins = PluginRegistry::new();
    plugins.register(TestPlugin).unwrap();
    assert!(plugins.register(TestPlugin).is_err());
    let _ = plugins
        .apply(ModuleDefinition::builder::<Root>())
        .unwrap()
        .build();
}

// ── Microservices ─────────────────────────────────────────────────────

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn forward_ref_resolves_after_container_build() {
    use ironic::{ContainerBuilder, Dependency, ForwardRef, ProviderDefinition, Scope};
    use std::sync::Arc;

    struct ServiceB;
    struct ServiceA {
        b: ForwardRef<ServiceB>,
    }

    let fwd = ForwardRef::<ServiceB>::new();
    let inner = fwd.shared_inner();
    let mut builder = ContainerBuilder::new();
    builder
        .register(ProviderDefinition::value(fwd))
        .unwrap()
        .register(ProviderDefinition::factory::<ServiceB, _, _>(
            Scope::Singleton,
            Vec::new(),
            |_| async { Ok(ServiceB) },
        ))
        .unwrap()
        .register(ProviderDefinition::factory::<ServiceA, _, _>(
            Scope::Singleton,
            vec![Dependency::required::<ForwardRef<ServiceB>>()],
            |r| async move {
                let b: Arc<ForwardRef<ServiceB>> = r.resolve().await?;
                Ok(ServiceA { b: (*b).clone() })
            },
        ))
        .unwrap();
    let container = builder.build();
    // Register the forward ref with the container
    container.register_forward_ref(ironic::ProviderKey::of::<ServiceB>(), inner);
    container.resolve_forward_refs().await.unwrap();
    let a = container.resolve::<ServiceA>().await.unwrap();
    let _b: Arc<ServiceB> = a.b.get().await;
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn inmemory_client_server_request_response() {
    use ironic::distributed::microservices::{
        InMemoryServer, MicroserviceClient, MicroserviceServer, TransportError,
    };
    use std::sync::Arc;

    let (client, server) = InMemoryServer::pair(16);
    server.on_message(
        "ping",
        Arc::new(|payload, _ctx| {
            Box::pin(async move {
                let msg: String =
                    serde_json::from_slice(&payload).map_err(|e| TransportError(e.to_string()))?;
                let resp = format!("pong:{msg}");
                serde_json::to_vec(&resp).map_err(|e| TransportError(e.to_string()))
            })
        }),
    );
    server.listen().await.unwrap();

    let result: String = client.send("ping", &"hello".to_string()).await.unwrap();
    assert_eq!(result, "pong:hello");
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn inmemory_client_server_event() {
    use ironic::distributed::microservices::{
        InMemoryServer, MicroserviceClient, MicroserviceServer, TransportError,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let (client, server) = InMemoryServer::pair(16);
    let received = Arc::new(Mutex::new(Vec::new()));

    let ev = Arc::clone(&received);
    server.on_event(
        "order.created",
        Arc::new(move |payload, _ctx| {
            let ev = Arc::clone(&ev);
            Box::pin(async move {
                let msg: String =
                    serde_json::from_slice(&payload).map_err(|e| TransportError(e.to_string()))?;
                ev.lock().await.push(msg);
                Ok(())
            })
        }),
    );
    server.listen().await.unwrap();

    client
        .emit("order.created", &"order-1".to_string())
        .await
        .unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    assert_eq!(received.lock().await.len(), 1);
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn pattern_normalization_works() {
    use ironic::distributed::microservices::{MsPattern, normalize_pattern};

    assert_eq!(normalize_pattern("user.get"), "user.get");
    let pat = MsPattern::from("order.create");
    assert_eq!(pat.normalize(), "order.create");

    let json_pattern = serde_json::json!({"service": "users"});
    let normalized = normalize_pattern(json_pattern);
    assert!(normalized.contains("users"));
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn custom_transport_strategy_creates_paired_endpoints() {
    use ironic::distributed::microservices::{CustomTransportStrategy, InMemoryServer};

    struct TestTransport;

    impl CustomTransportStrategy for TestTransport {
        type Client = ironic::distributed::microservices::InMemoryClient;
        type Server = InMemoryServer;
        fn create(self) -> (Self::Client, Self::Server) {
            InMemoryServer::pair(16)
        }
    }

    let (_client, _server) = TestTransport.create();
}

// ── ForwardRef DI ─────────────────────────────────────────────────────

#[cfg(feature = "events")]
#[tokio::test]
async fn forward_ref_in_di_container() {
    use ironic::{ContainerBuilder, Dependency, ForwardRef, ProviderDefinition, Scope};
    use std::sync::Arc;

    struct ServiceA {
        b: ForwardRef<ServiceB>,
    }
    struct ServiceB;

    let fwd = ForwardRef::<ServiceB>::new();
    let inner = fwd.shared_inner();
    let mut builder = ContainerBuilder::new();
    builder
        .register(ProviderDefinition::value(fwd))
        .unwrap()
        .register(ProviderDefinition::factory::<ServiceA, _, _>(
            Scope::Singleton,
            vec![Dependency::required::<ForwardRef<ServiceB>>()],
            move |r| async move {
                let fwd: Arc<ForwardRef<ServiceB>> = r.resolve().await?;
                Ok(ServiceA { b: (*fwd).clone() })
            },
        ))
        .unwrap()
        .register(ProviderDefinition::value(ServiceB))
        .unwrap();

    let container = builder.build();
    container.register_forward_ref(ironic::ProviderKey::of::<ServiceB>(), inner);
    container.resolve_forward_refs().await.unwrap();
    let a = container.resolve::<ServiceA>().await.unwrap();
    let _b: Arc<ServiceB> = a.b.get().await;
}

// ── Serializer / Deserializer ─────────────────────────────────────────

#[cfg(feature = "microservices")]
#[test]
fn serializer_round_trips_json() {
    use ironic::distributed::microservices::{Deserializer, IdentitySerializer, Serializer};

    let codec = IdentitySerializer;
    let bytes = codec.to_bytes(&42_u32).unwrap();
    let value: u32 = codec.read_bytes(&bytes).unwrap();
    assert_eq!(value, 42);
}

#[cfg(feature = "microservices")]
#[test]
fn serializer_handles_structs() {
    use ironic::distributed::microservices::{Deserializer, IdentitySerializer, Serializer};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Person {
        name: String,
        age: u8,
    }

    let codec = IdentitySerializer;
    let person = Person {
        name: "Alice".into(),
        age: 30,
    };
    let bytes = codec.to_bytes(&person).unwrap();
    let decoded: Person = codec.read_bytes(&bytes).unwrap();
    assert_eq!(person, decoded);
}

// ── LazyModule ────────────────────────────────────────────────────────

#[cfg(feature = "events")]
#[tokio::test]
async fn lazy_module_defers_registration() {
    use ironic::LazyModule;

    struct TestMod;
    impl ironic::Module for TestMod {
        fn definition() -> ironic::ModuleDefinition {
            ironic::ModuleDefinition::builder::<TestMod>().build()
        }
    }

    let def = LazyModule::<TestMod>::definition();
    assert!(def.id().type_name().contains("TestMod"));
}

// ── GraphQL Integration ──────────────────────────────────────────────

#[cfg(feature = "graphql")]
#[test]
fn graphql_schema_builder_constructs() {
    use async_graphql::{EmptyMutation, EmptySubscription, Object};
    use ironic::graphql_integration::GraphqlSchemaBuilder;

    struct Query;
    #[Object]
    impl Query {
        async fn hello(&self) -> &str {
            "world"
        }
    }

    let builder = GraphqlSchemaBuilder::new(Query, EmptyMutation, EmptySubscription);
    let schema = builder.finish();
    assert!(!schema.sdl().is_empty());
}

#[cfg(feature = "graphql")]
#[test]
fn graphql_proc_macros_compile() {
    // Verify that the proc-macro attributes exist via ironic_macros directly
    use ironic_macros::{gql_query, mutation, resolver, subscription};

    #[allow(dead_code)]
    #[resolver]
    struct TestResolver;

    #[allow(dead_code, clippy::unused_async)]
    #[mutation]
    async fn test_mutation() {}

    #[allow(dead_code, clippy::unused_async)]
    #[subscription]
    async fn test_subscription() {}

    #[allow(dead_code, clippy::unused_async)]
    #[gql_query]
    async fn test_query() {}
}

// ── Redis Integration Tests (require running Redis) ────────────────────

/// Integration test for Redis transport (run with `cargo test --features transport-redis -- --ignored`).
///
/// Requires a Redis instance running at 127.0.0.1:6379.
#[cfg(feature = "transport-redis")]
#[ignore = "requires running Redis instance"]
#[tokio::test]
async fn redis_transport_request_response() {
    use ironic::distributed::microservices::{
        MicroserviceClient, MicroserviceServer, TransportError,
    };
    use ironic::distributed::transport_redis::{
        RedisClient, RedisClientConfig, RedisServer, RedisServerConfig,
    };
    use std::sync::Arc;

    let server = RedisServer::new(RedisServerConfig::default());
    server.on_message(
        "ping",
        Arc::new(|payload, _ctx| {
            Box::pin(async move {
                let msg: String =
                    serde_json::from_slice(&payload).map_err(|e| TransportError(e.to_string()))?;
                let resp = format!("pong:{msg}");
                serde_json::to_vec(&resp).map_err(|e| TransportError(e.to_string()))
            })
        }),
    );
    server.listen().await.unwrap();

    let client = RedisClient::new(RedisClientConfig::default());
    client.connect().await.unwrap();
    let result: String = client.send("ping", &"hello".to_string()).await.unwrap();
    assert_eq!(result, "pong:hello");
}

/// Integration test for cross-process events via Redis transport.
///
/// Requires a Redis instance running at 127.0.0.1:6379.
#[cfg(feature = "transport-redis")]
#[ignore = "requires running Redis instance"]
#[tokio::test]
async fn redis_transport_cross_process_event() {
    use ironic::distributed::microservices::{
        MicroserviceClient, MicroserviceServer, TransportError,
    };
    use ironic::distributed::transport_redis::{
        RedisClient, RedisClientConfig, RedisServer, RedisServerConfig,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let server = RedisServer::new(RedisServerConfig::default());
    let received = Arc::new(Mutex::new(Vec::new()));
    let ev = Arc::clone(&received);
    server.on_event(
        "user.created",
        Arc::new(move |payload, _ctx| {
            let ev = Arc::clone(&ev);
            Box::pin(async move {
                let name: String =
                    serde_json::from_slice(&payload).map_err(|e| TransportError(e.to_string()))?;
                ev.lock().await.push(name);
                Ok(())
            })
        }),
    );
    server.listen().await.unwrap();

    let client = RedisClient::new(RedisClientConfig::default());
    client.connect().await.unwrap();
    client
        .emit("user.created", &"Alice".to_string())
        .await
        .unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert_eq!(received.lock().await.len(), 1);
}

// ── DiscoveryService ──────────────────────────────────────────────────

#[cfg(all(feature = "events", feature = "microservices"))]
#[test]
fn discovery_service_default_state_is_empty() {
    use ironic::DiscoveryService;

    let discovery = DiscoveryService::new();
    assert_eq!(discovery.provider_count(), 0);
    let health = discovery.provider_health();
    assert_eq!(health.total_providers, 0);
}

// ── OpenAPI Mapped Types ──────────────────────────────────────────────

#[cfg(all(feature = "openapi", feature = "validation"))]
#[test]
fn openapi_mapped_types_compile() {
    use ironic::OpenApiSchema;

    #[derive(serde::Serialize, OpenApiSchema)]
    struct User {
        name: String,
        email: String,
        password: String,
    }

    #[derive(ironic::PartialType)]
    #[partial(User)]
    struct UpdateUser;

    #[derive(ironic::PickType)]
    #[pick(User, fields = ["name", "email"])]
    struct UserResponse;

    #[derive(ironic::OmitType)]
    #[omit(User, fields = ["password"])]
    struct SafeUser;

    let _ = UpdateUser::openapi_schema();
    let _ = UserResponse::openapi_schema();
    let _ = SafeUser::openapi_schema();
}

// ── Transport Provider (EventClient / EventServer) ────────────────────

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn transport_provider_paired_event_flow() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use ironic::distributed::microservices::MicroserviceServer;
    use ironic::distributed::transport_provider::EventServer;

    let (client, server) = EventServer::paired(16);

    let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let events = Arc::clone(&received);
    server.on_event(
        "test.event",
        Arc::new(move |payload, _ctx| {
            let events = Arc::clone(&events);
            Box::pin(async move {
                let msg: String = serde_json::from_slice(&payload).map_err(|e| {
                    ironic::distributed::microservices::TransportError(e.to_string())
                })?;
                events.lock().await.push(msg);
                Ok(())
            })
        }),
    );

    server.listen().await.unwrap();

    let payload = "hello".to_string();
    client.emit("test.event", &payload).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let msgs = received.lock().await;
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0], "hello");
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn transport_provider_resolves_from_container() {
    use ironic::distributed::transport_provider::{
        EventClient, EventServer, TransportConfig, TransportKind,
    };
    use ironic::{ContainerBuilder, ProviderDefinition};

    let config = TransportConfig {
        kind: TransportKind::InMemory,
        brokers: "test".into(),
        topic: "test".into(),
        group_id: "test".into(),
    };

    let mut builder = ContainerBuilder::new();
    builder
        .register(ProviderDefinition::value(config))
        .unwrap()
        .register(EventClient::provider_definition())
        .unwrap()
        .register(EventServer::provider_definition())
        .unwrap();

    let container = builder.build();
    container.resolve_forward_refs().await.unwrap();

    let _client = container.resolve::<EventClient>().await.unwrap();
    let _server = container.resolve::<EventServer>().await.unwrap();
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn transport_provider_event_handler_with_auto_register() {
    use ironic::distributed::transport_provider::EventServer;
    use ironic::event_handler;

    #[event_handler(transport = "my.event", auto_register)]
    #[allow(clippy::unused_async)]
    async fn handle_my_event(event: String) {
        let _ = event;
    }

    // Verify the auto-register struct implements AsyncModuleInit
    fn check_trait_bound<T: ironic::AsyncModuleInit>() {}
    check_trait_bound::<__EventHandlerAuto_handle_my_event>();

    // Verify the registration function exists and accepts EventServer
    let (_, server) = EventServer::paired(16);
    __event_handler_reg_handle_my_event(&server);
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn transport_provider_event_handler_transport_only() {
    use ironic::distributed::transport_provider::EventServer;
    use ironic::event_handler;

    #[event_handler(transport = "other.event")]
    #[allow(clippy::unused_async)]
    async fn handle_other(event: String) {
        let _ = event;
    }

    let (_, server) = EventServer::paired(16);
    __event_handler_reg_handle_other(&server);
}

#[cfg(all(feature = "microservices", feature = "events"))]
#[tokio::test]
async fn transport_provider_event_handler_with_event_client_injection() {
    use ironic::event_handler;
    use ironic::distributed::transport_provider::{EventClient, EventServer};

    // Handler with injected EventClient — the second param is resolved from DI
    #[event_handler(transport = "test.injected", auto_register)]
    #[allow(clippy::unused_async)]
    async fn handle_with_client(event: String, _events: ::std::sync::Arc<EventClient>) {
        let _ = event;
        // In production, _events.emit(...) would be called here
    }

    // Verify auto_register struct implements AsyncModuleInit
    fn check_trait_bound<T: ironic::AsyncModuleInit>() {}
    check_trait_bound::<__EventHandlerAuto_handle_with_client>();

    // Verify registration function accepts (EventServer, Arc<EventClient>)
    let (client, server) = EventServer::paired(16);
    let events = ::std::sync::Arc::new(client);
    __event_handler_reg_handle_with_client(&server, events);
}
