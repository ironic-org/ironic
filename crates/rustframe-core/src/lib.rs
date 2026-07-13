#![doc = "Application graph compilation and lifecycle orchestration for RustFrame."]
// Rich graph errors retain all related type identities for actionable diagnostics.
#![allow(clippy::result_large_err)]

mod application;
mod health;
mod lifecycle;

use std::{
    any::{TypeId, type_name},
    collections::{HashMap, HashSet},
    fmt,
};

use rustframe_di::{ContainerBuilder, ProviderDefinition, ProviderKey, RegistrationError};
use rustframe_http::{
    CompiledHttpApplication, ControllerDefinition, RequestTracing, RouteError,
    compile_controller_routes,
};

pub use application::{
    ApplicationError, FrameworkApplication, FrameworkApplicationBuilder, MissingPlatform,
};
pub use health::{HealthModule, HealthStatus};
pub use lifecycle::{
    LifecycleDefinition, LifecycleDefinitionBuilder, LifecycleError, LifecycleFuture,
    OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
};
pub use rustframe_common::{FrameworkError, FrameworkResult};

/// A statically declared `RustFrame` application module.
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
#[derive(Clone, Copy)]
pub struct ModuleDefinitionFactory {
    id: ModuleId,
    define: fn() -> ModuleDefinition,
}

impl ModuleDefinitionFactory {
    fn of<T: Module>() -> Self {
        Self {
            id: ModuleId::of::<T>(),
            define: T::definition,
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
#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    id: ModuleId,
    imports: Vec<ModuleDefinitionFactory>,
    providers: Vec<ProviderDefinition>,
    controllers: Vec<ControllerDefinition>,
    exports: Vec<ProviderKey>,
    lifecycle: Vec<LifecycleDefinition>,
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
        }
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
}

impl ModuleDefinitionBuilder {
    /// Adds a direct module import.
    #[must_use]
    pub fn import<T: Module>(mut self) -> Self {
        self.imports.push(ModuleDefinitionFactory::of::<T>());
        self
    }

    /// Adds a provider owned by this module.
    #[must_use]
    pub fn provider(mut self, provider: ProviderDefinition) -> Self {
        self.providers.push(provider);
        self
    }

    /// Adds a controller owned by this module.
    #[must_use]
    pub fn controller(mut self, controller: ControllerDefinition) -> Self {
        self.controllers.push(controller);
        self
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
        }
    }
}

/// A validated application module with precomputed provider visibility.
#[derive(Clone, Debug)]
pub struct CompiledModule {
    id: ModuleId,
    imports: Vec<ModuleId>,
    providers: Vec<ProviderDefinition>,
    controllers: Vec<ControllerDefinition>,
    exports: Vec<ProviderKey>,
    visible_providers: HashMap<ProviderKey, ModuleId>,
    lifecycle: Vec<LifecycleDefinition>,
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
/// returned application. This is primarily intended for isolated application tests.
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
    })
}

fn validate_dependencies(
    module: ModuleId,
    consumer: ProviderKey,
    dependencies: &[rustframe_di::Dependency],
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
    use rustframe_di::{Dependency, ProviderDefinition, Scope};

    use super::*;

    struct Database;
    struct Repository;
    struct Service;
    struct Controller;

    fn provider<T: Send + Sync + 'static>(
        dependencies: Vec<rustframe_di::Dependency>,
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
