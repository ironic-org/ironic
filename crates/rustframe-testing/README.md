# `RustFrame` Testing

`rustframe-testing` provides isolated module and HTTP tests without global overrides or bound
network ports. Every builder creates a new dependency container.

## Unit-testing services

Compile a module and replace selected registrations locally:

```rust,ignore
let module = TestModule::builder::<UsersModule>()
    .override_value(UserRepository::in_memory())
    .compile()
    .await?;

let service = module.resolve::<UsersService>().await?;
```

Use `override_provider` for a complete `ProviderDefinition`, `override_value` for a singleton
value, or `override_factory` for an asynchronous singleton or transient factory. Overrides must
retain the registered concrete type because dependency lookup is type-based.

## Integration-testing controllers

`TestApplication` initializes the real module graph, container, routes, request pipeline, and
lifecycle hooks. Requests execute in process:

```rust,ignore
let application = TestApplication::builder::<AppModule>()
    .override_value(UserRepository::in_memory())
    .build()
    .await?;

let response = application
    .post("/users")
    .header("x-request-id", "test-1")
    .json(&CreateUserRequest { name: "Ada".into() })
    .send()
    .await;

response.assert_status(200);
response.assert_header("content-type", "application/json");
response.assert_json(&serde_json::json!({"name": "Ada"}));

application.shutdown().await?;
```

Call `shutdown` to observe cleanup failures. If it is omitted, dropping the test application still
runs application-shutdown and module-destruction hooks on an isolated cleanup runtime.
