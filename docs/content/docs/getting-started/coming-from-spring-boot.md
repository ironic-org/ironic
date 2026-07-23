---
title: Coming from Spring Boot
description: A guide for Spring Boot / Java developers transitioning to Ironic. Concepts, patterns, and code comparisons.
---

# Coming from Spring Boot

If you're coming from Spring Boot, Ironic's architecture will feel familiar — dependency injection, controllers, middleware, configuration profiles, and aspect-oriented patterns are all core concepts in both frameworks.

## Concept mapping

| Spring Boot | Ironic | Notes |
|-------------|--------|-------|
| `@SpringBootApplication` | `Application::create()` | Application entry point |
| `@RestController` | `#[controller]` | Controller with route prefix |
| `@GetMapping` / `@PostMapping` | `#[get]` / `#[post]` | Route method attributes |
| `@Autowired` / `@Inject` | `#[injectable]` constructor | DI resolution |
| `@Service` | `#[injectable]` + `ProviderDefinition` | Service registration |
| `@Repository` | Database provider | Data access layer |
| `@Configuration` / `@Bean` | `Module` trait + providers | Bean/module definitions |
| `@PathVariable` | `PathParameter` | Path parameter extraction |
| `@RequestBody` | `JsonBody<T>` | Request body extraction |
| `@RequestParam` | `QueryParameters` | Query string parameters |
| `@RequestHeader` | `HeaderParameter` | Header extraction |
| `@ComponentScan` | Module imports | Module discovery |
| `@Profile` | `profile("staging")` | Environment profiles |
| `application.yml` / `application.properties` | `.toml` / `.json` files | Configuration files |
| `application-{profile}.yml` | `config.{env}.toml` | Profile-specific config |
| `HandlerInterceptor` | `Interceptor` | Pre/post handler hooks |
| `@ExceptionHandler` | `ExceptionFilter` | Error handling |
| `SecurityFilterChain` | Guard + middleware | Auth middleware |
| `@PreAuthorize` | `#[guard]` | Method-level authorization |
| `@Async` | Background services + event bus | Async task execution |
| `@Scheduled` | Cron scheduling | Scheduled jobs |
| `@EventListener` | Event bus | Event-driven architecture |
| `Spring AOP` | Interceptors + middleware | Aspect-oriented patterns |
| `Spring Data JPA` | SQLx / SeaORM / Diesel | Database access |
| `Maven` / `Gradle` | Cargo | Build tool |
| `Spring Actuator` | Metrics + health endpoint | Production monitoring |
| `Swagger / OpenAPI` | Auto-generated OpenAPI | API documentation |
| `JUnit + Mockito` | `#[cfg(test)]` + TestModule | Testing |

## Controller comparison

Spring Boot:
```java
@RestController
@RequestMapping("/api/users")
public class UserController {

    @Autowired
    private UserService userService;

    @GetMapping("/{id}")
    public ResponseEntity<User> getUser(@PathVariable Long id) {
        User user = userService.findById(id);
        return ResponseEntity.ok(user);
    }
}
```

Ironic:
```rust
#[controller("/api/users")]
struct UserController {
    service: Arc<UserService>,
}

#[routes]
impl UserController {
    #[get("/{id}")]
    async fn get(&self, id: PathParameter<i64>) -> Result<Json<User>, HttpError> {
        let user = self.service.find_by_id(*id).await?;
        Ok(Json(user))
    }
}
```

## Dependency injection

Spring Boot:
```java
@Service
public class UserService {

    private final UserRepository repository;

    public UserService(UserRepository repository) {
        this.repository = repository;
    }
}
```

Ironic:
```rust
#[injectable]
impl UserService {
    fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }
}
```

## Module / configuration

Spring Boot:
```java
@Configuration
@EnableWebSecurity
public class SecurityConfig {

    @Bean
    public SecurityFilterChain filterChain(HttpSecurity http) {
        return http.cors().and().csrf().disable().build();
    }
}
```

Ironic:
```rust
impl Module for SecurityModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::constructor(
                Scope::Singleton,
                Vec::new(),
                |_| Ok(SecurityConfig::default()),
            ))
            .build()
    }
}
```

## Key differences

| Aspect | Spring Boot | Ironic |
|--------|-------------|--------|
| Runtime | JVM (JDK, GC) | Rust (native, compiled) |
| Memory management | Garbage Collector | Ownership + borrow checker |
| Startup time | 2-10 seconds (JVM warmup) | <100ms (native binary) |
| Memory footprint | 100-500 MB (JVM heap) | 2-15 MB |
| DI resolution | Runtime (reflection + proxies) | Compile-time (generics + macros) |
| Async model | Reactive (WebFlux) or Thread-per-request | tokio (async/await) |
| Configuration | YAML / properties / env | TOML / JSON / env |
| Build tool | Maven / Gradle | Cargo |
| Package format | JAR / WAR | Single binary |
| ORM | JPA / Hibernate | SQLx (compile-time checked SQL) |
| Migration | Flyway / Liquibase | SQLx migrations |
| Testing | JUnit + Mockito + TestContainers | Built-in test module + in-process client |
| Deployment | JVM + JAR / Docker | Docker (scratch image possible) |

## What you'll need to learn

- **Rust ownership and borrowing** — The biggest shift from Java. The compiler enforces memory safety at compile time. Start with the [Rust Book](https://doc.rust-lang.org/book/)
- **`Result<T, E>` instead of exceptions** — Errors are values, not control flow. No try/catch, no checked exceptions
- **No null** — `Option<T>` replaces all nullable references. The compiler ensures you handle both cases
- **Traits instead of interfaces** — More powerful (associated types, default methods, generic impls)
- **No inheritance** — Composition over inheritance is enforced by the language
- **`Arc<T>` instead of `@Autowired`** — Shared ownership is explicit via atomic reference counting
- **Macros instead of annotations** — Proc macros generate code at compile time, not runtime proxies
- **Cargo instead of Maven** — Build, test, benchmark, and document from one tool

## Migration tips

1. **Translate packages to modules** — Each Spring Boot package becomes an Ironic `Module`
2. **Convert `@Bean` to `ProviderDefinition`** — Factory methods become provider registrations
3. **Replace `@Autowired` with constructor injection** — Ironic only supports constructor injection
4. **Move from JPA to SQLx or SeaORM** — Compile-time checked SQL queries instead of Hibernate magic
5. **Use `HttpError` for all error responses** — Replace `ResponseEntity` with `Result<Json<T>, HttpError>`
6. **Generate OpenAPI specs automatically** — No need for `@Operation` and `@Schema` annotations everywhere
7. **Write tests with the built-in test module** — No need for Mockito mocks; use real instances with DI overrides
