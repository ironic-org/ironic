# RFC 0001: Module System

- Status: Accepted for initial implementation
- Target: RustFrame 0.1
- Authors: RustFrame contributors

## Summary

RustFrame modules are static application-assembly descriptions. A module has a stable type-based identity, imports other modules, owns providers and controllers, and explicitly exports providers that imported modules may use. The module compiler validates the complete graph before any provider is instantiated.

## Motivation

Large applications need boundaries that are stronger than directory conventions but lighter than separate Rust crates. Modules must make provider ownership and visibility predictable without runtime reflection or string identifiers.

## Goals

- Represent modules with ordinary Rust types and generated or handwritten definitions.
- Detect invalid imports, exports, and provider access during application build.
- Produce deterministic initialization and shutdown order.
- Keep module compilation independent of HTTP platforms.
- Expose enough graph information for tests and future developer tools.

## Non-goals

- Dynamic, lazy, conditional, or asynchronously configured modules.
- Module re-exports.
- Loading modules from shared libraries at runtime.
- Treating source directories as implicit module declarations.

## Public API

The explicit API is the source of truth. Procedural macros will generate equivalent definitions later.

```rust
use std::any::{TypeId, type_name};

pub trait Module: Send + Sync + 'static {
    fn definition() -> ModuleDefinition;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ModuleId {
    type_id: TypeId,
    type_name: &'static str,
}

impl ModuleId {
    pub fn of<T: Module>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }
}

pub struct ModuleDefinition {
    pub id: ModuleId,
    pub imports: Vec<ModuleDefinitionFactory>,
    pub providers: Vec<ProviderDefinition>,
    pub controllers: Vec<ControllerDefinition>,
    pub exports: Vec<ProviderKey>,
}

pub struct ModuleDefinitionFactory {
    pub id: ModuleId,
    pub define: fn() -> ModuleDefinition,
}

impl ModuleDefinition {
    pub fn new<T: Module>() -> ModuleDefinitionBuilder { /* ... */ }
}
```

Definitions contain factories for imports rather than recursively materialized definitions. This keeps the declaration simple while allowing the compiler to detect cycles before expanding the same module repeatedly.

Builder usage:

```rust
pub struct UsersModule;

impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::new::<Self>()
            .import::<DatabaseModule>()
            .provider(UserRepository::definition())
            .provider(UsersService::definition())
            .controller(UsersController::definition())
            .export::<UsersService>()
            .build()
    }
}
```

## Identity

Module identity is `TypeId` within a running process. The fully qualified Rust type name is retained for diagnostics but does not participate in equality.

Consequences:

- Importing the same module type through multiple paths produces one compiled module.
- Two distinct module types with identical contents remain distinct modules.
- Generic instantiations are distinct because Rust gives each instantiation a distinct `TypeId`.
- A module type may appear only once in an application graph in 0.1.

String names are never used as identity. Future dynamic modules will require a separate instance-key design and are therefore deferred.

## Ownership

- A provider definition is owned by exactly one module.
- A controller definition is owned by exactly one module.
- Registering the same provider key twice in one module is an error.
- Registering the same provider key in separate modules is allowed; visibility resolution selects the registration owned by the requesting module or an explicitly imported export.
- Registering one controller type in multiple modules is an error for the 0.1 application graph because it would make route ownership ambiguous.

The compiler records ownership rather than flattening all registrations into one global container.

## Imports and exports

An import establishes a directed edge from the importing module to the imported module.

```text
UsersModule ──imports──▶ DatabaseModule
```

A module can resolve:

1. Providers it owns.
2. Providers explicitly exported by modules it directly imports.

Imports are not transitive. If `A` imports `B` and `B` imports `C`, providers exported by `C` are not visible to `A` unless `B` owns and exports an explicit binding of its own. Module re-export syntax is deferred.

An export is valid only when the exporting module owns the provider key. Exporting an unknown provider or a provider visible only through an import is an `InvalidExport` error.

There are no global modules in the 0.1 kernel. This removes implicit visibility and keeps dependency paths reviewable. Global modules may be reconsidered after the base semantics are stable.

## Compilation

The compiler receives one root `ModuleDefinition` and performs two passes.

### Pass 1: discover and validate modules

1. Start a depth-first traversal at the root.
2. Mark a module `Visiting` before traversing imports.
3. Encountering a `Visiting` module reports an import cycle with the full path.
4. Encountering a `Visited` module reuses the existing compiled node.
5. Validate local duplicate providers, controllers, imports, and exports.
6. Mark the module `Visited` after its imports are complete.

Duplicate import declarations in one module are rejected even though graph traversal could deduplicate them. Rejecting them catches accidental metadata generation errors.

### Pass 2: resolve visibility and dependencies

1. Build each module's local provider table.
2. Build its imported-export table from direct imports.
3. Reject an imported provider key exported by multiple direct imports as ambiguous unless the module owns a local provider with that key.
4. Validate every declared provider and controller dependency against the module's visible table.
5. Produce the compiled application graph.

The module compiler validates declared dependency metadata. Runtime DI still tracks a resolution stack as defense in depth for factory behavior that cannot be described statically.

## Initialization order

Initialization uses deterministic post-order depth-first traversal:

- Imported modules initialize before importing modules.
- Imports are visited in declaration order.
- Providers within a module initialize in declaration order after dependency validation.
- Controllers are created after their module's eager providers.

If a module is reachable through more than one path, it initializes only once at its first post-order position.

Shutdown uses the exact reverse of successful initialization order. Only components whose initialization completed successfully participate in shutdown.

## Compiled representation

```rust
pub struct CompiledApplicationGraph {
    pub root: ModuleId,
    pub modules: Vec<CompiledModule>,
    pub initialization_order: Vec<ModuleId>,
}

pub struct CompiledModule {
    pub id: ModuleId,
    pub imports: Vec<ModuleId>,
    pub providers: Vec<ProviderDefinition>,
    pub controllers: Vec<ControllerDefinition>,
    pub exports: Vec<ProviderKey>,
    pub visible_providers: ProviderVisibilityMap,
}
```

The vector order is deterministic and safe to expose to diagnostics. Internal lookup maps may use hash-based storage without making their iteration order observable.

## Error behavior

The compiler returns structured errors with stable codes and human-readable type names:

- `RF_MODULE_IMPORT_CYCLE`
- `RF_MODULE_DUPLICATE_IMPORT`
- `RF_MODULE_DUPLICATE_PROVIDER`
- `RF_MODULE_DUPLICATE_CONTROLLER`
- `RF_MODULE_CONTROLLER_REUSED`
- `RF_MODULE_INVALID_EXPORT`
- `RF_MODULE_AMBIGUOUS_IMPORT`
- `RF_MODULE_PRIVATE_PROVIDER`
- `RF_MODULE_MISSING_PROVIDER`

Example:

```text
RF_MODULE_PRIVATE_PROVIDER: `UsersService` cannot resolve `DatabasePool`.

Module: app::users::UsersModule
Dependency chain: UsersService -> UserRepository -> DatabasePool

`DatabasePool` is owned by app::database::DatabaseModule but is not exported.
Suggested fix: export `DatabasePool` from DatabaseModule or expose a public repository provider instead.
```

## Alternatives considered

### Globally flattened providers

Rejected because it removes module encapsulation, makes duplicate bindings unpredictable, and weakens diagnostics.

### String module and provider tokens

Rejected as the default because renames are not compiler checked and collisions become runtime concerns.

### Recursively owned module definitions

Rejected because they expand duplicate imports before cycle detection and encourage cloning large metadata graphs.

### Transitive imports

Rejected because dependencies become available through distant implementation details. Direct imports make coupling explicit.

## Performance impact

Module compilation is startup-only. The algorithm is linear in modules, import edges, providers, controllers, and declared dependency edges, excluding diagnostic string construction. Runtime provider resolution uses precomputed visibility tables and does not traverse module imports.

## Testing strategy

- Unit tests for identity and definition builders.
- Graph tests for diamond imports and deterministic ordering.
- Error tests for every structured module error.
- Visibility tests for local, exported, private, ambiguous, and non-transitive providers.
- Lifecycle tests proving initialization and reverse shutdown order.
- Property tests may later generate acyclic and cyclic graphs, but are not required for the first implementation.

## Migration considerations

This is the initial contract. Before 1.0, adding dynamic module instances or module re-exports may require new definition types. Existing static module identity and direct-export behavior should remain supported.

## Unresolved questions

None for the initial implementation. Global modules, dynamic module instances, and re-exports require separate RFCs.
