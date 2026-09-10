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
async fn event_macro_generates_registration_function() {
    use ironic::event;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[event(capacity = 32)]
    #[allow(clippy::unused_async)]
    async fn handle_string_event(event: Arc<String>) {
        let _ = event;
    }

    let bus = EventBus::default();
    __event_reg_handle_string_event(&bus);
    // Give the spawned task time to subscribe
    tokio::task::yield_now().await;

    let n = bus.publish("hello".to_owned()).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_macro_with_custom_event_type() {
    use ironic::event;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[derive(Clone, Debug, PartialEq)]
    struct OrderPlaced(u32);

    #[event(capacity = 8)]
    #[allow(clippy::unused_async)]
    async fn handle_order(event: Arc<OrderPlaced>) {
        let _ = event;
    }

    let bus = EventBus::default();
    __event_reg_handle_order(&bus);
    tokio::task::yield_now().await;

    let n = bus.publish(OrderPlaced(42)).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_macro_auto_register_generates_async_init_impl() {
    use ironic::event;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[event(auto_register, capacity = 16)]
    #[allow(clippy::unused_async)]
    async fn handle_auto_event(event: Arc<String>) {
        let _ = event;
    }

    // Verify auto-register struct exists by checking it implements AsyncModuleInit
    fn check_trait_bound<T: ironic::AsyncModuleInit>() {}
    check_trait_bound::<__EventAuto_handle_auto_event>();

    let bus = EventBus::default();
    __event_reg_handle_auto_event(&bus);
    tokio::task::yield_now().await;

    let n = bus.publish("auto".to_owned()).await;
    assert_eq!(n, 1);
}

#[cfg(feature = "events")]
#[tokio::test]
async fn event_macro_default_capacity() {
    use ironic::event;
    use ironic::services::events::EventBus;
    use std::sync::Arc;

    #[event]
    #[allow(clippy::unused_async)]
    async fn handle_default(event: Arc<String>) {
        let _ = event;
    }

    let bus = EventBus::default();
    __event_reg_handle_default(&bus);
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
