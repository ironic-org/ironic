---
title: Coming from NestJS
description: A guide for NestJS developers transitioning to Ironic. Concepts, patterns, and code comparisons.
---

# Coming from NestJS

If you know NestJS, you already understand most of Ironic's architecture. The patterns are intentionally similar, adapted to Rust's type system and idioms.

## Concept mapping

| NestJS | Ironic | Notes |
|--------|--------|-------|
| `@Module({})` | `impl Module for X` | Module wiring is explicit in code, not decorators |
| `@Injectable()` | `#[injectable]` | Proc macro that generates DI registration |
| `@Controller()` | `#[controller]` | Path prefix on a struct |
| `@Get()` / `@Post()` | `#[get]` / `#[post]` | Route method decorators |
| `@Body()` / `@Param()` | `JsonBody<T>` / `PathParameter` | Typed extractors |
| `@Guard()` | `#[guard]` | Authorization guard |
| `@UsePipes()` | Pipe traits | Parameter transformation |
| `@Catch()` | `ExceptionFilter` | Error handling |
| `constructor(private svc: Svc)` | `Arc<Svc>` via DI | Constructor injection via resolver |
| `ConfigService` | `ConfigurationLoader` | Typed configuration |
| `OnModuleInit` | `ModuleLifecycle` | Lifecycle hooks |

## Project structure

NestJS:
```
src/
  app.module.ts
  users/
    users.module.ts
    users.controller.ts
    users.service.ts
```

Ironic:
```
src/
  app.rs              # Application root with modules
  users/
    mod.rs            # Module implementation
    controller.rs     # Controller with routes
    service.rs        # Injectable service
```

## Dependency Injection

NestJS (decorator-based):
```typescript
@Injectable()
export class UsersService {
  constructor(
    @InjectRepository(User) private repo: Repository<User>,
    private config: ConfigService,
  ) {}
}
```

Ironic (trait-based):
```rust
#[injectable]
impl UsersService {
    fn new(repo: Arc<UserRepository>, config: Arc<AppConfig>) -> Self {
        Self { repo, config }
    }
}
```

## Module definition

NestJS:
```typescript
@Module({
  imports: [DatabaseModule],
  controllers: [UsersController],
  providers: [UsersService],
  exports: [UsersService],
})
export class UsersModule {}
```

Ironic:
```rust
impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DatabaseModule>()
            .provider(ProviderDefinition::value(UsersService::new))
            .controller(UsersController::definition())
            .build()
    }
}
```

## Guards

NestJS:
```typescript
@Injectable()
export class AuthGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    // ...
  }
}

@UseGuards(AuthGuard)
@Get('/profile')
getProfile() {}
```

Ironic:
```rust
struct AuthGuard;

impl Guard for AuthGuard {
    type Error = AuthError;
    async fn decide(&self, ctx: &GuardContext) -> GuardDecision<Self::Error> {
        GuardDecision::Allow
    }
}

#[routes]
impl UsersController {
    #[guard(AuthGuard)]
    #[get("/profile")]
    async fn get_profile(&self) -> Json<User> {
        // ...
    }
}
```

## Key differences

| Aspect | NestJS | Ironic |
|--------|--------|--------|
| Language | TypeScript/JS | Rust |
| Runtime | Node.js (single-threaded) | Native (multi-threaded) |
| DI | Runtime reflection | Compile-time generics |
| Decorators | JS decorators (runtime) | Proc macros (compile time) |
| Errors | Exceptions | `Result<T, E>` with `HttpError` |
| Async | `Promise` / `Observable` | `Future` / `tokio` |
| Config | `ConfigService` with YAML | Typed `ConfigurationLoader` |
| ORM | TypeORM / Mongoose | SQLx / SeaORM / Diesel |

## Migration tips

1. **Start with module structure** — Map your NestJS modules to Ironic's `Module` trait
2. **Convert services to `#[injectable]`** — Constructor injection works the same way
3. **Replace decorators with proc macros** — `@Get()` → `#[get]`, `@Body()` → `JsonBody<T>`
4. **Use `Result<T, HttpError>`** — Replace NestJS exceptions with typed error returns
5. **Leverage the CLI** — `ironic gen controller` and `ironic gen service` scaffold the boilerplate
6. **Run `cargo check` early and often** — Rust's compiler catches wiring errors that NestJS would only catch at runtime
