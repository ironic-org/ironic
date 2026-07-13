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
    };
    queue.enqueue(message.clone()).await.unwrap();
    let received = queue.dequeue().await.unwrap().unwrap();
    queue.reject(received, true).await.unwrap();
    assert_eq!(queue.dequeue().await.unwrap(), Some(message));
}

#[cfg(feature = "microservices")]
#[tokio::test]
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
