#![doc = "Application graph compilation and lifecycle orchestration for Ironic."]
// Rich graph errors retain all related type identities for actionable diagnostics.
#![allow(clippy::result_large_err)]

mod application;
mod health;
mod lifecycle;

use std::{
    any::{TypeId, type_name},
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use std::sync::OnceLock;

use ironic_di::{ContainerBuilder, ProviderDefinition, ProviderKey, ProviderValue, Container, RegistrationError};
use ironic_http::{
    CompiledHttpApplication, ControllerDefinition, RequestTracing, RouteError,
    compile_controller_routes,
};

pub use application::{
    Application, ApplicationBuilder, ApplicationError, MissingPlatform, ModuleConfigurationError,
};
pub use health::{
    BuildInfo, HealthConfig, HealthIndicator, HealthModule, HealthStatus,
    configure as configure_health, register as register_health_indicator,
};
pub use lifecycle::{
    AfterShutdown, AsyncModuleInit, BeforeShutdown, LifecycleDefinition, LifecycleDefinitionBuilder,
    LifecycleError, LifecycleFuture, OnApplicationBootstrap, OnApplicationShutdown, OnError,
    OnGuardDenied, OnModuleConfigure, OnModuleDestroy, OnModuleInit, OnModuleLoad, OnModuleUnload,
    OnRequestDestroy, OnRequestInit, OnServerReady,
};

/// A statically declared Ironic application module.
pub trait Module: Send + Sync + 'static {
    /// Returns the module's explicit definition.
    fn definition() -> ModuleDefinition;
}

/// The stable, type-based identity of a static module.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ModuleId {
    type_id: TypeId,
    type_name: &'static str,
}

impl ModuleId {
    /// Returns the identity of module type `T`.
    #[must_use]
    pub fn of<T: Module>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }

    /// Returns the fully qualified module type name used in diagnostics.
    #[must_use]
    pub const fn type_name(self) -> &'static str {
        self.type_name
    }
}

impl fmt::Debug for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ModuleId")
            .field(&self.type_name)
            .finish()
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.type_name)
    }
}

/// A lazily expanded static module import.
#[derive(Clone)]
pub struct ModuleDefinitionFactory {
    id: ModuleId,
    define: Arc<dyn Fn() -> ModuleDefinition + Send + Sync>,
}

impl ModuleDefinitionFactory {
    fn of<T: Module>() -> Self {
        Self {
            id: ModuleId::of::<T>(),
            define: Arc::new(T::definition),
        }
    }

    fn value(definition: ModuleDefinition) -> Self {
        let id = definition.id;
        Self {
            id,
            define: Arc::new(move || definition.clone()),
        }
    }
}

impl fmt::Debug for ModuleDefinitionFactory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModuleDefinitionFactory")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// The complete static declaration of one module.
#[derive(Clone)]
pub struct ModuleDefinition {
    id: ModuleId,
    imports: Vec<ModuleDefinitionFactory>,
    providers: Vec<ProviderDefinition>,
    controllers: Vec<ControllerDefinition>,
    exports: Vec<ProviderKey>,
    lifecycle: Vec<LifecycleDefinition>,
    async_init_callbacks: Vec<lifecycle::AsyncInitCallback>,
    /// When true, this module's exports are visible to every other module.
    global: bool,
}

impl std::fmt::Debug for ModuleDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleDefinition")
            .field("id", &self.id)
            .field("imports", &self.imports.len())
            .field("providers", &self.providers.len())
            .field("controllers", &self.controllers.len())
            .field("exports", &self.exports.len())
            .field("lifecycle", &self.lifecycle)
            .field("async_init_count", &self.async_init_callbacks.len())
            .field("global", &self.global)
            .finish()
    }
}

impl ModuleDefinition {
    /// Starts a definition for module `T`.
    #[must_use]
    pub fn builder<T: Module>() -> ModuleDefinitionBuilder {
        ModuleDefinitionBuilder {
            id: ModuleId::of::<T>(),
            imports: Vec::new(),
            providers: Vec::new(),
            controllers: Vec::new(),
            exports: Vec::new(),
            lifecycle: Vec::new(),
            async_init_callbacks: Vec::new(),
            global: false,
        }
    }

    /// Returns async initialization callbacks in declaration order.
    #[must_use]
    pub fn async_init_callbacks(&self) -> &[lifecycle::AsyncInitCallback] {
        &self.async_init_callbacks
    }

    /// Returns the module identity.
    #[must_use]
    pub const fn id(&self) -> ModuleId {
        self.id
    }
}

/// Builds a static module definition.
pub struct ModuleDefinitionBuilder {
    id: ModuleId,
    imports: Vec<ModuleDefinitionFactory>,
    providers: Vec<ProviderDefinition>,
    controllers: Vec<ControllerDefinition>,
    exports: Vec<ProviderKey>,
    lifecycle: Vec<LifecycleDefinition>,
    async_init_callbacks: Vec<lifecycle::AsyncInitCallback>,
    global: bool,
}

impl ModuleDefinitionBuilder {
    /// Adds a direct module import.
    #[must_use]
    pub fn import<T: Module>(mut self) -> Self {
        self.imports.push(ModuleDefinitionFactory::of::<T>());
        self
    }

    /// Adds an import whose static definition is expanded only during graph compilation.
    #[must_use]
    pub fn import_lazy<T: Module>(self) -> Self {
        self.import::<T>()
    }

    /// Adds a runtime-created module definition as a direct import.
    #[must_use]
    pub fn import_definition(mut self, definition: ModuleDefinition) -> Self {
        self.imports
            .push(ModuleDefinitionFactory::value(definition));
        self
    }

    /// Adds a static import only when `enabled` is true.
    #[must_use]
    pub fn import_if<T: Module>(self, enabled: bool) -> Self {
        if enabled { self.import::<T>() } else { self }
    }

    /// Adds a provider owned by this module.
    #[must_use]
    pub fn provider(mut self, provider: ProviderDefinition) -> Self {
        self.providers.push(provider);
        self
    }

    /// Adds a provider only when `enabled` is true.
    #[must_use]
    pub fn provider_if(self, enabled: bool, provider: ProviderDefinition) -> Self {
        if enabled {
            self.provider(provider)
        } else {
            self
        }
    }

    /// Adds a controller owned by this module.
    #[must_use]
    pub fn controller(mut self, controller: ControllerDefinition) -> Self {
        self.controllers.push(controller);
        self
    }

    /// Adds a controller only when `enabled` is true.
    #[must_use]
    pub fn controller_if(self, enabled: bool, controller: ControllerDefinition) -> Self {
        if enabled {
            self.controller(controller)
        } else {
            self
        }
    }

    /// Exports provider `T` to modules that directly import this module.
    #[must_use]
    pub fn export<T: Send + Sync + 'static>(mut self) -> Self {
        self.exports.push(ProviderKey::of::<T>());
        self
    }

    /// Registers explicit lifecycle callbacks for a provider owned by this module.
    #[must_use]
    pub fn lifecycle(mut self, lifecycle: LifecycleDefinition) -> Self {
        self.lifecycle.push(lifecycle);
        self
    }

    /// Marks this module's exports as globally visible without explicit imports.
    #[must_use]
    pub fn global(mut self) -> Self {
        self.global = true;
        self
    }

    /// Registers a provider for async initialization.
    ///
    /// The provider type must implement [`AsyncModuleInit`]. During
    /// `Application::build()`, the provider is resolved from the
    /// container and its `async_init()` method is called with access
    /// to the full container.
    #[must_use]
    pub fn async_init<T: AsyncModuleInit>(mut self) -> Self {
        let key = ProviderKey::of::<T>();
        let callback: lifecycle::AsyncInitCallback = std::sync::Arc::new(
            move |_provider: ProviderValue, container: std::sync::Arc<Container>| {
                let container = std::sync::Arc::clone(&container);
                Box::pin(async move {
                    let resolved = container
                        .resolve_key(key)
                        .await
                        .map_err(|e| LifecycleError::new(format!("async_init resolve failed: {e}")))?;
                    let init = resolved.downcast_ref::<T>()
                        .ok_or_else(|| LifecycleError::new("async_init downcast failed"))?;
                    init.async_init(&container).await
                })
            },
        );
        self.async_init_callbacks.push(callback);
        self
    }

    /// Completes the definition.
    #[must_use]
    pub fn build(self) -> ModuleDefinition {
        ModuleDefinition {
            id: self.id,
            imports: self.imports,
            providers: self.providers,
            controllers: self.controllers,
            exports: self.exports,
            lifecycle: self.lifecycle,
            async_init_callbacks: self.async_init_callbacks,
            global: self.global,
        }
    }
}

/// A validated application module with precomputed provider visibility.
#[derive(Clone)]
pub struct CompiledModule {
    id: ModuleId,
    imports: Vec<ModuleId>,
    providers: Vec<ProviderDefinition>,
    controllers: Vec<ControllerDefinition>,
    exports: Vec<ProviderKey>,
    visible_providers: HashMap<ProviderKey, ModuleId>,
    lifecycle: Vec<LifecycleDefinition>,
    async_init_callbacks: Vec<lifecycle::AsyncInitCallback>,
}

impl std::fmt::Debug for CompiledModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledModule")
            .field("id", &self.id)
            .field(
                "imports",
                &self.imports.iter().map(|i| i.type_name()).collect::<Vec<_>>(),
            )
            .field("providers", &self.providers.len())
            .field("controllers", &self.controllers.len())
            .field("exports", &self.exports.len())
            .field("lifecycle", &self.lifecycle)
            .field("async_init_count", &self.async_init_callbacks.len())
            .finish()
    }
}

impl CompiledModule {
    /// Returns the module identity.
    #[must_use]
    pub const fn id(&self) -> ModuleId {
        self.id
    }

    /// Returns direct imports in declaration order.
    #[must_use]
    pub fn imports(&self) -> &[ModuleId] {
        &self.imports
    }

    /// Returns provider definitions in declaration order.
    #[must_use]
    pub fn providers(&self) -> &[ProviderDefinition] {
        &self.providers
    }

    /// Returns controller definitions in declaration order.
    #[must_use]
    pub fn controllers(&self) -> &[ControllerDefinition] {
        &self.controllers
    }

    /// Returns provider keys exported by this module.
    #[must_use]
    pub fn exports(&self) -> &[ProviderKey] {
        &self.exports
    }

    /// Returns the owner of a provider visible from this module.
    #[must_use]
    pub fn provider_owner(&self, key: ProviderKey) -> Option<ModuleId> {
        self.visible_providers.get(&key).copied()
    }

    /// Returns lifecycle definitions in declaration order.
    #[must_use]
    pub fn lifecycle(&self) -> &[LifecycleDefinition] {
        &self.lifecycle
    }

    /// Returns async initialization callbacks in declaration order.
    #[must_use]
    pub fn async_init_callbacks(&self) -> &[lifecycle::AsyncInitCallback] {
        &self.async_init_callbacks
    }
}

/// A complete validated application graph.
#[derive(Clone, Debug)]
pub struct CompiledApplicationGraph {
    root: ModuleId,
    modules: Vec<CompiledModule>,
    initialization_order: Vec<ModuleId>,
    shutdown_order: Vec<ModuleId>,
}

impl CompiledApplicationGraph {
    /// Returns the root module identity.
    #[must_use]
    pub const fn root(&self) -> ModuleId {
        self.root
    }

    /// Returns modules in deterministic discovery post-order.
    #[must_use]
    pub fn modules(&self) -> &[CompiledModule] {
        &self.modules
    }

    /// Returns imported-first initialization order.
    #[must_use]
    pub fn initialization_order(&self) -> &[ModuleId] {
        &self.initialization_order
    }

    /// Returns reverse successful-initialization order for shutdown hooks.
    #[must_use]
    pub fn shutdown_order(&self) -> &[ModuleId] {
        &self.shutdown_order
    }

    /// Finds a compiled module by identity.
    #[must_use]
    pub fn module(&self, id: ModuleId) -> Option<&CompiledModule> {
        self.modules.iter().find(|module| module.id == id)
    }
}

/// A module graph validation failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ModuleError {
    /// An import chain contains a cycle.
    #[error("RF_MODULE_IMPORT_CYCLE: importing `{module}` creates a cycle")]
    ImportCycle {
        /// The repeated module.
        module: ModuleId,
        /// The cycle path, including the repeated module.
        path: Vec<ModuleId>,
    },
    /// A module imports the same module more than once.
    #[error("RF_MODULE_DUPLICATE_IMPORT: `{module}` imports `{imported}` more than once")]
    DuplicateImport {
        /// The importing module.
        module: ModuleId,
        /// The duplicated import.
        imported: ModuleId,
    },
    /// A module owns duplicate provider keys.
    #[error("RF_MODULE_DUPLICATE_PROVIDER: `{module}` registers `{provider}` more than once")]
    DuplicateProvider {
        /// The owning module.
        module: ModuleId,
        /// The duplicated provider.
        provider: ProviderKey,
    },
    /// A module owns duplicate controller keys.
    #[error("RF_MODULE_DUPLICATE_CONTROLLER: `{module}` registers `{controller}` more than once")]
    DuplicateController {
        /// The owning module.
        module: ModuleId,
        /// The duplicated controller.
        controller: ProviderKey,
    },
    /// One controller type is owned by multiple modules.
    #[error("RF_MODULE_CONTROLLER_REUSED: controller `{controller}` belongs to multiple modules")]
    ControllerReused {
        /// The duplicated controller.
        controller: ProviderKey,
        /// The first owner.
        first: ModuleId,
        /// The second owner.
        second: ModuleId,
    },
    /// A module exports a provider it does not own.
    #[error("RF_MODULE_INVALID_EXPORT: `{module}` does not own exported provider `{provider}`")]
    InvalidExport {
        /// The exporting module.
        module: ModuleId,
        /// The invalid provider key.
        provider: ProviderKey,
    },
    /// Two imports export the same provider key.
    #[error(
        "RF_MODULE_AMBIGUOUS_IMPORT: `{provider}` is exported by multiple imports of `{module}`"
    )]
    AmbiguousImport {
        /// The importing module.
        module: ModuleId,
        /// The ambiguous provider key.
        provider: ProviderKey,
        /// The exporting modules.
        owners: Vec<ModuleId>,
    },
    /// A dependency exists in an imported module but is private.
    #[error("RF_MODULE_PRIVATE_PROVIDER: `{consumer}` in `{module}` cannot access `{provider}`")]
    PrivateProvider {
        /// The consuming module.
        module: ModuleId,
        /// The provider or controller declaring the dependency.
        consumer: ProviderKey,
        /// The inaccessible dependency.
        provider: ProviderKey,
        /// The module that owns the private dependency.
        owner: ModuleId,
    },
    /// A declared dependency does not exist in the visible graph.
    #[error("RF_MODULE_MISSING_PROVIDER: `{consumer}` in `{module}` cannot resolve `{provider}`")]
    MissingProvider {
        /// The consuming module.
        module: ModuleId,
        /// The provider or controller declaring the dependency.
        consumer: ProviderKey,
        /// The missing dependency.
        provider: ProviderKey,
    },
    /// Lifecycle metadata references a provider not owned by the module.
    #[error("RF_MODULE_INVALID_LIFECYCLE: `{module}` does not own lifecycle provider `{provider}`")]
    InvalidLifecycle {
        /// The declaring module.
        module: ModuleId,
        /// The invalid lifecycle provider.
        provider: ProviderKey,
    },
}

/// Runtime access to the dependency injection container for lazy resolution
/// and dynamic provider access.
///
/// Register a provider for `ModuleRef` in any module and the framework will
/// populate the container reference during application initialization.
#[derive(Clone)]
pub struct ModuleRef {
    container: std::sync::Arc<OnceLock<ironic_di::Container>>,
}

impl ModuleRef {
    /// Creates a new empty module reference.
    #[must_use]
    pub fn new() -> Self {
        Self {
            container: std::sync::Arc::new(OnceLock::new()),
        }
    }

    /// Populates the container reference. Called once during application build.
    pub(crate) fn set_container(&self, container: ironic_di::Container) {
        let _ = self.container.set(container);
    }

    /// Resolves a provider by concrete type.
    ///
    /// # Errors
    ///
    /// Returns a resolve error when the container is not initialized or the
    /// provider cannot be constructed.
    pub async fn resolve<T: Send + Sync + 'static>(
        &self,
    ) -> Result<std::sync::Arc<T>, ironic_di::ResolveError> {
        self.container
            .get()
            .ok_or_else(|| {
                let key = ironic_di::ProviderKey::of::<T>();
                ironic_di::ResolveError::MissingProvider {
                    key,
                    path: Vec::new(),
                }
            })?
            .resolve::<T>()
            .await
    }

    /// Resolves an optional provider, returning `None` when it is not registered.
    ///
    /// # Errors
    ///
    /// Returns a resolve error when the container is not initialized or a
    /// registered provider fails to construct.
    pub async fn resolve_optional<T: Send + Sync + 'static>(
        &self,
    ) -> Result<Option<std::sync::Arc<T>>, ironic_di::ResolveError> {
        match self.resolve::<T>().await {
            Ok(provider) => Ok(Some(provider)),
            Err(ironic_di::ResolveError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

impl Default for ModuleRef {
    fn default() -> Self {
        Self::new()
    }
}

/// A failure while turning a validated module graph into HTTP runtime state.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum HttpApplicationBuildError {
    /// Runtime provider registration failed.
    #[error(transparent)]
    ProviderRegistration(#[from] RegistrationError),
    /// Controller route compilation failed.
    #[error(transparent)]
    Route(#[from] RouteError),
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Visited,
}

/// Compiles and validates a static module graph.
///
/// # Errors
///
/// Returns [`ModuleError`] when imports, ownership, exports, or dependencies are invalid.
pub fn compile_module_graph(
    root: ModuleDefinition,
) -> Result<CompiledApplicationGraph, ModuleError> {
    let root_id = root.id;
    let mut definitions = HashMap::new();
    let mut states = HashMap::new();
    let mut order = Vec::new();
    let mut path = Vec::new();
    discover(root, &mut definitions, &mut states, &mut order, &mut path)?;

    validate_controller_ownership(&definitions, &order)?;
    let mut compiled = Vec::with_capacity(order.len());
    for id in &order {
        compiled.push(compile_module(*id, &definitions)?);
    }

    let shutdown_order = order.iter().rev().copied().collect();
    Ok(CompiledApplicationGraph {
        root: root_id,
        modules: compiled,
        initialization_order: order,
        shutdown_order,
    })
}

/// Builds the DI container and executable route table for a validated graph.
///
/// # Errors
///
/// Returns [`HttpApplicationBuildError`] when runtime registrations or routes conflict.
pub fn build_http_application(
    graph: &CompiledApplicationGraph,
) -> Result<CompiledHttpApplication, HttpApplicationBuildError> {
    build_http_application_with_overrides(graph, Vec::new())
}

/// Builds HTTP runtime state and replaces selected provider registrations.
///
/// Overrides are applied after the validated module graph is registered and remain local to the
/// returned application. Additional providers are registered before overrides.
///
/// # Errors
///
/// Returns [`HttpApplicationBuildError`] when registrations, overrides, or routes are invalid.
pub fn build_http_application_with_overrides(
    graph: &CompiledApplicationGraph,
    overrides: impl IntoIterator<Item = ProviderDefinition>,
) -> Result<CompiledHttpApplication, HttpApplicationBuildError> {
    let mut container = ContainerBuilder::new();
    let mut controllers = Vec::new();

    for module in graph.modules() {
        for provider in module.providers() {
            container.register(provider.clone())?;
        }
        for controller in module.controllers() {
            container.register(controller.provider().clone())?;
            controllers.push(controller.clone());
        }
    }

    for provider in overrides {
        container.override_with(provider)?;
    }

    let routes = compile_controller_routes(controllers)?;
    Ok(CompiledHttpApplication::new(container.build(), routes).middleware(RequestTracing::new()))
}

/// Builds HTTP runtime state, registering additional providers and applying overrides.
///
/// # Errors
///
/// Returns [`HttpApplicationBuildError`] when registrations, overrides, or routes are invalid.
pub fn build_http_application_with_extra_providers(
    graph: &CompiledApplicationGraph,
    extra_providers: impl IntoIterator<Item = ProviderDefinition>,
    overrides: impl IntoIterator<Item = ProviderDefinition>,
) -> Result<CompiledHttpApplication, HttpApplicationBuildError> {
    let mut container = ContainerBuilder::new();
    let mut controllers = Vec::new();

    for module in graph.modules() {
        for provider in module.providers() {
            container.register(provider.clone())?;
        }
        for controller in module.controllers() {
            container.register(controller.provider().clone())?;
            controllers.push(controller.clone());
        }
    }

    for provider in extra_providers {
        container.register(provider)?;
    }

    for provider in overrides {
        container.override_with(provider)?;
    }

    let routes = compile_controller_routes(controllers)?;
    Ok(CompiledHttpApplication::new(container.build(), routes).middleware(RequestTracing::new()))
}

fn discover(
    definition: ModuleDefinition,
    definitions: &mut HashMap<ModuleId, ModuleDefinition>,
    states: &mut HashMap<ModuleId, VisitState>,
    order: &mut Vec<ModuleId>,
    path: &mut Vec<ModuleId>,
) -> Result<(), ModuleError> {
    let id = definition.id;
    match states.get(&id) {
        Some(VisitState::Visited) => return Ok(()),
        Some(VisitState::Visiting) => {
            let start = path.iter().position(|item| *item == id).unwrap_or(0);
            let mut cycle = path[start..].to_vec();
            cycle.push(id);
            return Err(ModuleError::ImportCycle {
                module: id,
                path: cycle,
            });
        }
        None => {}
    }

    validate_local_duplicates(&definition)?;
    states.insert(id, VisitState::Visiting);
    path.push(id);
    let imports = definition.imports.clone();
    definitions.insert(id, definition);

    for import in imports {
        if states.get(&import.id) == Some(&VisitState::Visiting) {
            let start = path.iter().position(|item| *item == import.id).unwrap_or(0);
            let mut cycle = path[start..].to_vec();
            cycle.push(import.id);
            return Err(ModuleError::ImportCycle {
                module: import.id,
                path: cycle,
            });
        }
        if states.get(&import.id) != Some(&VisitState::Visited) {
            discover((import.define)(), definitions, states, order, path)?;
        }
    }

    path.pop();
    states.insert(id, VisitState::Visited);
    order.push(id);
    Ok(())
}

fn validate_local_duplicates(definition: &ModuleDefinition) -> Result<(), ModuleError> {
    let mut imports = HashSet::new();
    for import in &definition.imports {
        if !imports.insert(import.id) {
            return Err(ModuleError::DuplicateImport {
                module: definition.id,
                imported: import.id,
            });
        }
    }

    let mut providers = HashSet::new();
    for provider in &definition.providers {
        if !providers.insert(provider.key()) {
            return Err(ModuleError::DuplicateProvider {
                module: definition.id,
                provider: provider.key(),
            });
        }
    }

    let mut controllers = HashSet::new();
    for controller in &definition.controllers {
        if !controllers.insert(controller.key()) {
            return Err(ModuleError::DuplicateController {
                module: definition.id,
                controller: controller.key(),
            });
        }
    }

    for export in &definition.exports {
        if !providers.contains(export) {
            return Err(ModuleError::InvalidExport {
                module: definition.id,
                provider: *export,
            });
        }
    }
    for lifecycle in &definition.lifecycle {
        if !providers.contains(&lifecycle.key()) {
            return Err(ModuleError::InvalidLifecycle {
                module: definition.id,
                provider: lifecycle.key(),
            });
        }
    }
    Ok(())
}

fn validate_controller_ownership(
    definitions: &HashMap<ModuleId, ModuleDefinition>,
    order: &[ModuleId],
) -> Result<(), ModuleError> {
    let mut owners = HashMap::new();
    for id in order {
        for controller in &definitions[id].controllers {
            if let Some(first) = owners.insert(controller.key(), *id) {
                return Err(ModuleError::ControllerReused {
                    controller: controller.key(),
                    first,
                    second: *id,
                });
            }
        }
    }
    Ok(())
}

fn compile_module(
    id: ModuleId,
    definitions: &HashMap<ModuleId, ModuleDefinition>,
) -> Result<CompiledModule, ModuleError> {
    let definition = &definitions[&id];
    let local: HashSet<_> = definition
        .providers
        .iter()
        .map(ProviderDefinition::key)
        .collect();
    let mut visible: HashMap<ProviderKey, ModuleId> =
        local.iter().copied().map(|key| (key, id)).collect();
    let mut imported_owners: HashMap<ProviderKey, Vec<ModuleId>> = HashMap::new();

    for import in &definition.imports {
        let imported = &definitions[&import.id];
        for key in &imported.exports {
            if !local.contains(key) {
                imported_owners.entry(*key).or_default().push(import.id);
            }
        }
    }

    // Global module exports are visible to every other module.
    for (other_id, other_def) in definitions {
        if *other_id == id {
            continue;
        }
        if other_def.global {
            for key in &other_def.exports {
                if !local.contains(key) {
                    imported_owners.entry(*key).or_default().push(*other_id);
                }
            }
        }
    }

    for (key, owners) in imported_owners {
        if owners.len() > 1 {
            return Err(ModuleError::AmbiguousImport {
                module: id,
                provider: key,
                owners,
            });
        }
        visible.insert(key, owners[0]);
    }

    for provider in &definition.providers {
        validate_dependencies(
            id,
            provider.key(),
            provider.dependencies(),
            &visible,
            definitions,
        )?;
    }
    for controller in &definition.controllers {
        validate_dependencies(
            id,
            controller.key(),
            controller.dependencies(),
            &visible,
            definitions,
        )?;
    }

    Ok(CompiledModule {
        id,
        imports: definition.imports.iter().map(|import| import.id).collect(),
        providers: definition.providers.clone(),
        controllers: definition.controllers.clone(),
        exports: definition.exports.clone(),
        visible_providers: visible,
        lifecycle: definition.lifecycle.clone(),
        async_init_callbacks: definition.async_init_callbacks.clone(),
    })
}

fn validate_dependencies(
    module: ModuleId,
    consumer: ProviderKey,
    dependencies: &[ironic_di::Dependency],
    visible: &HashMap<ProviderKey, ModuleId>,
    definitions: &HashMap<ModuleId, ModuleDefinition>,
) -> Result<(), ModuleError> {
    for dependency in dependencies {
        if dependency.is_optional() || visible.contains_key(&dependency.key()) {
            continue;
        }

        let private_owner = definitions[&module]
            .imports
            .iter()
            .map(|import| import.id)
            .find(|imported| {
                definitions[imported]
                    .providers
                    .iter()
                    .any(|provider| provider.key() == dependency.key())
            });

        if let Some(owner) = private_owner {
            return Err(ModuleError::PrivateProvider {
                module,
                consumer,
                provider: dependency.key(),
                owner,
            });
        }
        return Err(ModuleError::MissingProvider {
            module,
            consumer,
            provider: dependency.key(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ironic_di::{Dependency, ProviderDefinition, Scope};

    use super::*;

    #[tokio::test]
    async fn module_ref_resolves_registered_provider() {
        let mut builder = ContainerBuilder::new();
        builder.register(ProviderDefinition::value(42_u32)).unwrap();
        let container = builder.build();

        let module_ref = ModuleRef::new();
        module_ref.set_container(container);

        let value = module_ref.resolve::<u32>().await.unwrap();
        assert_eq!(*value, 42);
    }

    #[tokio::test]
    async fn module_ref_optional_returns_none_for_missing() {
        let container = ContainerBuilder::new().build();
        let module_ref = ModuleRef::new();
        module_ref.set_container(container);

        let value: Option<Arc<u32>> = module_ref.resolve_optional::<u32>().await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn module_ref_errors_when_uninitialized() {
        let module_ref = ModuleRef::new();
        let result = module_ref.resolve::<u32>().await;
        assert!(result.is_err());
    }

    struct Database;
    struct Repository;
    struct Service;
    struct Controller;

    fn provider<T: Send + Sync + 'static>(
        dependencies: Vec<ironic_di::Dependency>,
    ) -> ProviderDefinition {
        ProviderDefinition::factory::<T, _, _>(Scope::Singleton, dependencies, |_resolver| async {
            panic!("graph compilation must not instantiate providers")
        })
    }

    struct DatabaseModule;
    impl Module for DatabaseModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(provider::<Database>(Vec::new()))
                .export::<Database>()
                .build()
        }
    }

    struct UsersModule;
    impl Module for UsersModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<DatabaseModule>()
                .provider(provider::<Repository>(vec![
                    Dependency::required::<Database>(),
                ]))
                .provider(provider::<Service>(vec![
                    Dependency::required::<Repository>(),
                ]))
                .controller(
                    ControllerDefinition::new::<Controller>(
                        "/users",
                        provider::<Controller>(vec![Dependency::required::<Service>()]),
                    )
                    .unwrap(),
                )
                .export::<Service>()
                .build()
        }
    }

    struct AppModule;
    impl Module for AppModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<UsersModule>()
                .build()
        }
    }

    #[test]
    fn compiles_a_valid_graph_in_imported_first_order() {
        let graph = compile_module_graph(AppModule::definition()).unwrap();
        assert_eq!(
            graph.initialization_order(),
            &[
                ModuleId::of::<DatabaseModule>(),
                ModuleId::of::<UsersModule>(),
                ModuleId::of::<AppModule>()
            ]
        );
        assert_eq!(
            graph.shutdown_order(),
            &[
                ModuleId::of::<AppModule>(),
                ModuleId::of::<UsersModule>(),
                ModuleId::of::<DatabaseModule>()
            ]
        );
        let users = graph.module(ModuleId::of::<UsersModule>()).unwrap();
        assert_eq!(
            users.provider_owner(ProviderKey::of::<Database>()),
            Some(ModuleId::of::<DatabaseModule>())
        );
        assert_eq!(
            users.provider_owner(ProviderKey::of::<Repository>()),
            Some(ModuleId::of::<UsersModule>())
        );
        let app = graph.module(ModuleId::of::<AppModule>()).unwrap();
        assert_eq!(
            app.provider_owner(ProviderKey::of::<Service>()),
            Some(ModuleId::of::<UsersModule>())
        );
        assert_eq!(app.provider_owner(ProviderKey::of::<Database>()), None);
    }

    #[test]
    fn supports_dynamic_and_conditional_module_composition() {
        let dynamic_database = ModuleDefinition::builder::<DatabaseModule>()
            .provider_if(true, provider::<Database>(Vec::new()))
            .export::<Database>()
            .build();
        let root = ModuleDefinition::builder::<AppModule>()
            .import_definition(dynamic_database)
            .import_if::<UsersModule>(false)
            .build();

        let graph = compile_module_graph(root).unwrap();
        assert_eq!(
            graph.initialization_order(),
            &[
                ModuleId::of::<DatabaseModule>(),
                ModuleId::of::<AppModule>()
            ]
        );
        assert_eq!(
            graph
                .module(ModuleId::of::<DatabaseModule>())
                .unwrap()
                .providers()
                .len(),
            1
        );
    }

    struct CycleA;
    struct CycleB;
    impl Module for CycleA {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<CycleB>()
                .build()
        }
    }
    impl Module for CycleB {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<CycleA>()
                .build()
        }
    }

    #[test]
    fn rejects_import_cycles() {
        assert!(matches!(
            compile_module_graph(CycleA::definition()),
            Err(ModuleError::ImportCycle { .. })
        ));
    }

    struct DuplicateImportModule;
    impl Module for DuplicateImportModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<DatabaseModule>()
                .import::<DatabaseModule>()
                .build()
        }
    }

    #[test]
    fn rejects_duplicate_imports() {
        assert!(matches!(
            compile_module_graph(DuplicateImportModule::definition()),
            Err(ModuleError::DuplicateImport { .. })
        ));
    }

    struct InvalidExportModule;
    impl Module for InvalidExportModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .export::<Database>()
                .build()
        }
    }

    #[test]
    fn rejects_invalid_exports() {
        assert!(matches!(
            compile_module_graph(InvalidExportModule::definition()),
            Err(ModuleError::InvalidExport { .. })
        ));
    }

    struct PrivateDatabaseModule;
    impl Module for PrivateDatabaseModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(provider::<Database>(Vec::new()))
                .build()
        }
    }
    struct PrivateConsumerModule;
    impl Module for PrivateConsumerModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<PrivateDatabaseModule>()
                .provider(provider::<Repository>(vec![
                    Dependency::required::<Database>(),
                ]))
                .build()
        }
    }

    struct MissingModule;
    impl Module for MissingModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(provider::<Repository>(vec![
                    Dependency::required::<Database>(),
                ]))
                .build()
        }
    }

    #[test]
    fn distinguishes_private_from_missing_providers() {
        assert!(matches!(
            compile_module_graph(PrivateConsumerModule::definition()),
            Err(ModuleError::PrivateProvider { .. })
        ));

        assert!(matches!(
            compile_module_graph(MissingModule::definition()),
            Err(ModuleError::MissingProvider { .. })
        ));
    }

    struct OtherDatabaseModule;
    impl Module for OtherDatabaseModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(provider::<Database>(Vec::new()))
                .export::<Database>()
                .build()
        }
    }
    struct AmbiguousModule;
    impl Module for AmbiguousModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<DatabaseModule>()
                .import::<OtherDatabaseModule>()
                .build()
        }
    }

    #[test]
    fn rejects_ambiguous_imports() {
        assert!(matches!(
            compile_module_graph(AmbiguousModule::definition()),
            Err(ModuleError::AmbiguousImport { .. })
        ));
    }

    struct DuplicateProviderModule;
    impl Module for DuplicateProviderModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(provider::<Database>(Vec::new()))
                .provider(provider::<Database>(Vec::new()))
                .build()
        }
    }

    #[test]
    fn rejects_duplicate_providers() {
        assert!(matches!(
            compile_module_graph(DuplicateProviderModule::definition()),
            Err(ModuleError::DuplicateProvider { .. })
        ));
    }

    struct FirstControllerModule;
    impl Module for FirstControllerModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .controller(
                    ControllerDefinition::new::<Controller>(
                        "/first",
                        provider::<Controller>(Vec::new()),
                    )
                    .unwrap(),
                )
                .build()
        }
    }
    struct SecondControllerModule;
    impl Module for SecondControllerModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .controller(
                    ControllerDefinition::new::<Controller>(
                        "/second",
                        provider::<Controller>(Vec::new()),
                    )
                    .unwrap(),
                )
                .build()
        }
    }
    struct ControllerRoot;
    impl Module for ControllerRoot {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .import::<FirstControllerModule>()
                .import::<SecondControllerModule>()
                .build()
        }
    }

    #[test]
    fn rejects_controller_reuse() {
        assert!(matches!(
            compile_module_graph(ControllerRoot::definition()),
            Err(ModuleError::ControllerReused { .. })
        ));
    }
}
