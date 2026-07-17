#![doc = "Type-safe dependency registration and asynchronous resolution for Ironic."]

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    fmt,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use tokio::sync::OnceCell;

/// A type-erased provider value used by framework internals.
pub type ProviderValue = Arc<dyn Any + Send + Sync>;
type ProviderFuture =
    Pin<Box<dyn Future<Output = Result<ProviderValue, ResolveError>> + Send + 'static>>;
type ErasedFactory = dyn Fn(Resolver) -> ProviderFuture + Send + Sync + 'static;

/// A stable, type-based provider identifier.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProviderKey {
    type_id: TypeId,
    type_name: &'static str,
}

impl ProviderKey {
    /// Returns the provider key for `T`.
    #[must_use]
    pub fn of<T: Send + Sync + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }

    /// Returns the fully qualified type name retained for diagnostics.
    #[must_use]
    pub const fn type_name(self) -> &'static str {
        self.type_name
    }
}

impl fmt::Debug for ProviderKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ProviderKey")
            .field(&self.type_name)
            .finish()
    }
}

impl fmt::Display for ProviderKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.type_name)
    }
}

/// The lifetime policy of a provider registration.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum Scope {
    /// One value is initialized and shared by the container.
    #[default]
    Singleton,
    /// A new value is constructed for every resolution.
    Transient,
    /// One value is constructed and shared within a single request.
    Request,
}

/// A dependency declared by a provider factory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dependency {
    key: ProviderKey,
    optional: bool,
}

impl Dependency {
    /// Declares a required dependency on `T`.
    #[must_use]
    pub fn required<T: Send + Sync + 'static>() -> Self {
        Self {
            key: ProviderKey::of::<T>(),
            optional: false,
        }
    }

    /// Declares an optional dependency on `T`.
    #[must_use]
    pub fn optional<T: Send + Sync + 'static>() -> Self {
        Self {
            key: ProviderKey::of::<T>(),
            optional: true,
        }
    }

    /// Returns the dependency's provider key.
    #[must_use]
    pub const fn key(self) -> ProviderKey {
        self.key
    }

    /// Returns whether a missing registration is allowed.
    #[must_use]
    pub const fn is_optional(self) -> bool {
        self.optional
    }
}

/// A complete provider registration description.
#[derive(Clone)]
pub struct ProviderDefinition {
    key: ProviderKey,
    scope: Scope,
    eager: bool,
    dependencies: Arc<[Dependency]>,
    factory: Arc<ErasedFactory>,
}

impl fmt::Debug for ProviderDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderDefinition")
            .field("key", &self.key)
            .field("scope", &self.scope)
            .field("eager", &self.eager)
            .field("dependencies", &self.dependencies)
            .finish_non_exhaustive()
    }
}

impl ProviderDefinition {
    /// Creates a provider definition from an asynchronous factory.
    #[must_use]
    pub fn factory<T, F, Fut>(scope: Scope, dependencies: Vec<Dependency>, factory: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(Resolver) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, ResolveError>> + Send + 'static,
    {
        let erased_factory = move |resolver: Resolver| {
            let future = factory(resolver);
            Box::pin(async move {
                future
                    .await
                    .map(|provider| Arc::new(provider) as ProviderValue)
            }) as ProviderFuture
        };

        Self {
            key: ProviderKey::of::<T>(),
            scope,
            eager: false,
            dependencies: dependencies.into(),
            factory: Arc::new(erased_factory),
        }
    }

    /// Creates a provider definition from a synchronous constructor.
    #[must_use]
    pub fn constructor<T, F>(scope: Scope, dependencies: Vec<Dependency>, factory: F) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(Resolver) -> Result<T, ResolveError> + Send + Sync + Clone + 'static,
    {
        Self::factory(scope, dependencies, move |resolver| {
            let factory = factory.clone();
            async move { factory(resolver) }
        })
    }

    /// Creates a singleton definition containing an existing value.
    #[must_use]
    pub fn value<T: Send + Sync + 'static>(value: T) -> Self {
        let value = Arc::new(value);
        let factory = move |_resolver: Resolver| -> ProviderFuture {
            let value = Arc::clone(&value);
            Box::pin(async move { Ok(value as ProviderValue) })
        };
        Self {
            key: ProviderKey::of::<T>(),
            scope: Scope::Singleton,
            eager: false,
            dependencies: Arc::from([]),
            factory: Arc::new(factory),
        }
    }

    /// Returns the registered provider key.
    #[must_use]
    pub const fn key(&self) -> ProviderKey {
        self.key
    }

    /// Returns the provider scope.
    #[must_use]
    pub const fn scope(&self) -> Scope {
        self.scope
    }

    /// Marks this provider for construction during application bootstrap.
    #[must_use]
    pub const fn eager(mut self) -> Self {
        self.eager = true;
        self
    }

    /// Returns whether application bootstrap should construct this provider eagerly.
    #[must_use]
    pub const fn is_eager(&self) -> bool {
        self.eager
    }

    /// Returns the declared dependencies.
    #[must_use]
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}

/// An error encountered while building a container.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum RegistrationError {
    /// A key was registered more than once.
    #[error("RF_DI_DUPLICATE_PROVIDER: provider `{key}` is already registered")]
    DuplicateProvider {
        /// The duplicated key.
        key: ProviderKey,
    },
    /// An override did not match an existing registration.
    #[error("RF_DI_INVALID_OVERRIDE: provider `{key}` is not registered")]
    InvalidOverride {
        /// The unmatched key.
        key: ProviderKey,
    },
}

/// A provider resolution failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ResolveError {
    /// No provider definition exists for a requested key.
    #[error("RF_DI_MISSING_PROVIDER: provider `{key}` is not registered")]
    MissingProvider {
        /// The missing key.
        key: ProviderKey,
        /// The dependency chain leading to the failure.
        path: Vec<ProviderKey>,
    },
    /// The current resolution chain contains a cycle.
    #[error("RF_DI_CIRCULAR_DEPENDENCY: resolving `{key}` would create a cycle")]
    CircularDependency {
        /// The repeated key.
        key: ProviderKey,
        /// The dependency chain including the repeated key.
        path: Vec<ProviderKey>,
    },
    /// A factory could not construct its provider.
    #[error("RF_DI_FACTORY_FAILED: provider `{key}` failed: {message}")]
    FactoryFailed {
        /// The provider being constructed.
        key: ProviderKey,
        /// A safe diagnostic message.
        message: String,
        /// The dependency chain leading to the factory.
        path: Vec<ProviderKey>,
    },
    /// The erased provider value did not have its declared concrete type.
    #[error("RF_DI_TYPE_MISMATCH: provider `{expected}` returned an unexpected type")]
    TypeMismatch {
        /// The type declared by the registration.
        expected: ProviderKey,
        /// The dependency chain leading to the mismatch.
        path: Vec<ProviderKey>,
    },
    /// A request-scoped provider was resolved without a request scope.
    #[error("IRONIC_DI_REQUEST_SCOPE_REQUIRED: provider `{key}` requires a request scope")]
    RequestScopeRequired {
        /// The request-scoped provider key.
        key: ProviderKey,
        /// The dependency chain leading to the failure.
        path: Vec<ProviderKey>,
    },
    /// A singleton attempted to depend on request-scoped state.
    #[error(
        "IRONIC_DI_SCOPE_VIOLATION: singleton construction cannot resolve request provider `{key}`"
    )]
    ScopeViolation {
        /// The invalid request-scoped dependency.
        key: ProviderKey,
        /// The dependency chain leading to the failure.
        path: Vec<ProviderKey>,
    },
}

impl ResolveError {
    /// Creates a safe factory failure associated with `T`.
    #[must_use]
    pub fn factory<T: Send + Sync + 'static>(message: impl Into<String>) -> Self {
        Self::FactoryFailed {
            key: ProviderKey::of::<T>(),
            message: message.into(),
            path: Vec::new(),
        }
    }

    /// Returns the resolution path captured by this error.
    #[must_use]
    pub fn path(&self) -> &[ProviderKey] {
        match self {
            Self::MissingProvider { path, .. }
            | Self::CircularDependency { path, .. }
            | Self::FactoryFailed { path, .. }
            | Self::TypeMismatch { path, .. }
            | Self::RequestScopeRequired { path, .. }
            | Self::ScopeViolation { path, .. } => path,
        }
    }

    fn with_path_if_empty(mut self, path: &[ProviderKey]) -> Self {
        let target = match &mut self {
            Self::MissingProvider { path, .. }
            | Self::CircularDependency { path, .. }
            | Self::FactoryFailed { path, .. }
            | Self::TypeMismatch { path, .. }
            | Self::RequestScopeRequired { path, .. }
            | Self::ScopeViolation { path, .. } => path,
        };
        if target.is_empty() {
            target.extend_from_slice(path);
        }
        self
    }
}

/// Builds an immutable dependency injection container.
#[derive(Default)]
pub struct ContainerBuilder {
    definitions: HashMap<ProviderKey, ProviderDefinition>,
}

impl ContainerBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a provider definition.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError::DuplicateProvider`] when the key already exists.
    pub fn register(
        &mut self,
        definition: ProviderDefinition,
    ) -> Result<&mut Self, RegistrationError> {
        let key = definition.key();
        if self.definitions.contains_key(&key) {
            return Err(RegistrationError::DuplicateProvider { key });
        }
        self.definitions.insert(key, definition);
        Ok(self)
    }

    /// Replaces an existing provider definition.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError::InvalidOverride`] when the key is not registered.
    pub fn override_with(
        &mut self,
        definition: ProviderDefinition,
    ) -> Result<&mut Self, RegistrationError> {
        let key = definition.key();
        let Some(slot) = self.definitions.get_mut(&key) else {
            return Err(RegistrationError::InvalidOverride { key });
        };
        *slot = definition;
        Ok(self)
    }

    /// Consumes the builder and returns an immutable container.
    #[must_use]
    pub fn build(self) -> Container {
        let registrations = self
            .definitions
            .into_iter()
            .map(|(key, definition)| {
                (
                    key,
                    Arc::new(Registration {
                        definition,
                        singleton: OnceCell::new(),
                    }),
                )
            })
            .collect();

        Container {
            inner: Arc::new(ContainerInner {
                registrations,
                health: Mutex::new(HashMap::new()),
            }),
        }
    }
}

struct Registration {
    definition: ProviderDefinition,
    singleton: OnceCell<ProviderValue>,
}

struct ContainerInner {
    registrations: HashMap<ProviderKey, Arc<Registration>>,
    health: Mutex<HashMap<ProviderKey, ProviderHealth>>,
}

/// Per-provider health statistics.
#[derive(Clone, Debug, Default)]
pub struct ProviderHealth {
    /// Total successful constructions.
    pub construct_count: u64,
    /// Total failed constructions.
    pub error_count: u64,
    /// Last error message, if any.
    pub last_error: Option<String>,
}

/// Consolidated health summary for the container.
#[derive(Clone, Debug)]
pub struct ProviderHealthSummary {
    /// Total registered providers.
    pub total_providers: usize,
    /// Per-provider health data.
    pub providers: HashMap<ProviderKey, ProviderHealth>,
}

/// An immutable dependency injection container.
#[derive(Clone)]
pub struct Container {
    inner: Arc<ContainerInner>,
}

impl Container {
    /// Resolves a provider by concrete type.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when registration lookup, construction, or downcasting fails.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ResolveError> {
        Resolver {
            container: self.clone(),
            path: Arc::from([]),
            request_cache: None,
            request_allowed: false,
        }
        .resolve::<T>()
        .await
    }

    /// Resolves a provider whose concrete type is known only through metadata.
    ///
    /// This supports framework internals such as erased controller dispatch. Application code
    /// should normally use [`Container::resolve`].
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when registration lookup or construction fails.
    pub async fn resolve_key(&self, key: ProviderKey) -> Result<ProviderValue, ResolveError> {
        Resolver {
            container: self.clone(),
            path: Arc::from([]),
            request_cache: None,
            request_allowed: false,
        }
        .resolve_erased(key)
        .await
    }

    /// Resolves an optional provider, returning `None` only when it is unregistered.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when a registered provider cannot be constructed or downcast.
    pub async fn resolve_optional<T: Send + Sync + 'static>(
        &self,
    ) -> Result<Option<Arc<T>>, ResolveError> {
        Resolver {
            container: self.clone(),
            path: Arc::from([]),
            request_cache: None,
            request_allowed: false,
        }
        .resolve_optional::<T>()
        .await
    }

    /// Creates an isolated resolver that caches request-scoped providers.
    #[must_use]
    pub fn request_scope(&self) -> RequestScope {
        RequestScope {
            container: self.clone(),
            cache: Arc::new(RequestCache::default()),
        }
    }

    /// Returns consolidated provider health statistics.
    #[must_use]
    pub fn health(&self) -> ProviderHealthSummary {
        let providers = self.inner.health.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        ProviderHealthSummary {
            total_providers: self.inner.registrations.len(),
            providers,
        }
    }

    /// Creates a new container with one provider registration replaced.
    ///
    /// Use for post-bootstrap hot-swapping of providers (e.g. A/B testing,
    /// feature-flag-gated implementations). The original container is
    /// unchanged.
    #[must_use]
    pub fn with_override(self, provider: ProviderDefinition) -> Self {
        let mut registrations = self.inner.registrations.clone();
        registrations.insert(
            provider.key(),
            Arc::new(Registration {
                definition: provider,
                singleton: OnceCell::new(),
            }),
        );
        Self {
            inner: Arc::new(ContainerInner {
                registrations,
                health: Mutex::new(HashMap::new()),
            }),
        }
    }
}

#[derive(Default)]
struct RequestCache {
    providers: Mutex<HashMap<ProviderKey, Arc<OnceCell<ProviderValue>>>>,
}

impl RequestCache {
    fn cell(&self, key: ProviderKey) -> Arc<OnceCell<ProviderValue>> {
        let mut providers = self
            .providers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(
            providers
                .entry(key)
                .or_insert_with(|| Arc::new(OnceCell::new())),
        )
    }
}

/// An isolated dependency resolver for one request.
#[derive(Clone)]
pub struct RequestScope {
    container: Container,
    cache: Arc<RequestCache>,
}

impl RequestScope {
    fn resolver(&self) -> Resolver {
        Resolver {
            container: self.container.clone(),
            path: Arc::from([]),
            request_cache: Some(Arc::clone(&self.cache)),
            request_allowed: true,
        }
    }

    /// Resolves a provider, caching request-scoped values within this scope.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when lookup, construction, or downcasting fails.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ResolveError> {
        self.resolver().resolve().await
    }

    /// Resolves a provider by erased metadata within this scope.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when lookup or construction fails.
    pub async fn resolve_key(&self, key: ProviderKey) -> Result<ProviderValue, ResolveError> {
        self.resolver().resolve_erased(key).await
    }

    /// Resolves an optional provider within this scope.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] if a registered provider fails to construct or downcast.
    pub async fn resolve_optional<T: Send + Sync + 'static>(
        &self,
    ) -> Result<Option<Arc<T>>, ResolveError> {
        self.resolver().resolve_optional().await
    }
}

/// A restricted, clonable container view passed to provider factories.
#[derive(Clone)]
pub struct Resolver {
    container: Container,
    path: Arc<[ProviderKey]>,
    request_cache: Option<Arc<RequestCache>>,
    request_allowed: bool,
}

impl Resolver {
    /// Resolves a required dependency.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when registration lookup, construction, or downcasting fails.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ResolveError> {
        let key = ProviderKey::of::<T>();
        let provider = self.resolve_erased(key).await?;
        provider
            .downcast::<T>()
            .map_err(|_| ResolveError::TypeMismatch {
                expected: key,
                path: self.path.to_vec(),
            })
    }

    /// Resolves an optional dependency.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when a registered provider cannot be constructed or downcast.
    pub async fn resolve_optional<T: Send + Sync + 'static>(
        &self,
    ) -> Result<Option<Arc<T>>, ResolveError> {
        match self.resolve::<T>().await {
            Ok(provider) => Ok(Some(provider)),
            Err(ResolveError::MissingProvider { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    async fn resolve_erased(&self, key: ProviderKey) -> Result<ProviderValue, ResolveError> {
        if self.path.contains(&key) {
            let mut path = self.path.to_vec();
            path.push(key);
            return Err(ResolveError::CircularDependency { key, path });
        }

        let Some(registration) = self.container.inner.registrations.get(&key).cloned() else {
            return Err(ResolveError::MissingProvider {
                key,
                path: self.path.to_vec(),
            });
        };

        if registration.definition.scope == Scope::Request {
            if self.request_cache.is_none() {
                return Err(ResolveError::RequestScopeRequired {
                    key,
                    path: self.path.to_vec(),
                });
            }
            if !self.request_allowed {
                return Err(ResolveError::ScopeViolation {
                    key,
                    path: self.path.to_vec(),
                });
            }
        }

        let mut path = self.path.to_vec();
        path.push(key);
        let child = Self {
            container: self.container.clone(),
            path: path.clone().into(),
            request_cache: self.request_cache.clone(),
            request_allowed: self.request_allowed
                && registration.definition.scope != Scope::Singleton,
        };

        let construct = || async {
            (registration.definition.factory)(child)
                .await
                .map_err(|error| error.with_path_if_empty(&path))
        };

        match registration.definition.scope {
            Scope::Singleton => registration
                .singleton
                .get_or_try_init(construct)
                .await
                .cloned(),
            Scope::Transient => construct().await,
            Scope::Request => {
                let cell = self
                    .request_cache
                    .as_ref()
                    .expect("request scope was validated")
                    .cell(key);
                cell.get_or_try_init(construct).await.cloned()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use tokio::sync::{Barrier, Notify};

    use super::*;

    #[derive(Debug)]
    struct Repository;

    #[derive(Debug)]
    struct Service {
        repository: Arc<Repository>,
    }

    fn service_definition(scope: Scope) -> ProviderDefinition {
        ProviderDefinition::factory(
            scope,
            vec![Dependency::required::<Repository>()],
            |resolver| async move {
                Ok(Service {
                    repository: resolver.resolve().await?,
                })
            },
        )
    }

    #[tokio::test]
    async fn resolves_concrete_dependencies() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::value(Repository))
            .unwrap()
            .register(service_definition(Scope::Singleton))
            .unwrap();

        let container = builder.build();
        let service = container.resolve::<Service>().await.unwrap();
        let repository = container.resolve::<Repository>().await.unwrap();
        assert!(Arc::ptr_eq(&service.repository, &repository));
    }

    #[tokio::test]
    async fn singleton_is_initialized_once_under_concurrency() {
        const TASKS: usize = 32;
        let calls = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(TASKS));
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::factory::<Repository, _, _>(
                Scope::Singleton,
                Vec::new(),
                {
                    let calls = Arc::clone(&calls);
                    move |_resolver| {
                        let calls = Arc::clone(&calls);
                        async move {
                            calls.fetch_add(1, Ordering::SeqCst);
                            tokio::task::yield_now().await;
                            Ok(Repository)
                        }
                    }
                },
            ))
            .unwrap();
        let container = builder.build();

        let mut tasks = Vec::new();
        for _ in 0..TASKS {
            let barrier = Arc::clone(&barrier);
            let container = container.clone();
            tasks.push(tokio::spawn(async move {
                barrier.wait().await;
                container.resolve::<Repository>().await.unwrap()
            }));
        }

        let mut values = Vec::new();
        for task in tasks {
            values.push(task.await.unwrap());
        }

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(
            values
                .windows(2)
                .all(|pair| Arc::ptr_eq(&pair[0], &pair[1]))
        );
    }

    #[tokio::test]
    async fn transient_creates_distinct_values() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::constructor(
                Scope::Transient,
                Vec::new(),
                |_resolver| Ok(Repository),
            ))
            .unwrap();
        let container = builder.build();

        let first = container.resolve::<Repository>().await.unwrap();
        let second = container.resolve::<Repository>().await.unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
    }

    #[tokio::test]
    async fn reports_the_missing_dependency_path() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(service_definition(Scope::Singleton))
            .unwrap();

        let error = builder.build().resolve::<Service>().await.unwrap_err();
        assert!(matches!(error, ResolveError::MissingProvider { .. }));
        assert_eq!(error.path(), &[ProviderKey::of::<Service>()]);
    }

    #[derive(Debug)]
    struct First;
    #[derive(Debug)]
    struct Second;

    #[tokio::test]
    async fn detects_runtime_cycles() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::factory(
                Scope::Singleton,
                vec![Dependency::required::<Second>()],
                |resolver| async move {
                    resolver.resolve::<Second>().await?;
                    Ok(First)
                },
            ))
            .unwrap()
            .register(ProviderDefinition::factory(
                Scope::Singleton,
                vec![Dependency::required::<First>()],
                |resolver| async move {
                    resolver.resolve::<First>().await?;
                    Ok(Second)
                },
            ))
            .unwrap();

        let error = builder.build().resolve::<First>().await.unwrap_err();
        assert!(matches!(error, ResolveError::CircularDependency { .. }));
        assert_eq!(
            error.path(),
            &[
                ProviderKey::of::<First>(),
                ProviderKey::of::<Second>(),
                ProviderKey::of::<First>()
            ]
        );
    }

    #[tokio::test]
    async fn failed_singleton_initialization_can_retry() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::factory::<Repository, _, _>(
                Scope::Singleton,
                Vec::new(),
                {
                    let calls = Arc::clone(&calls);
                    move |_resolver| {
                        let attempt = calls.fetch_add(1, Ordering::SeqCst);
                        async move {
                            if attempt == 0 {
                                Err(ResolveError::factory::<Repository>("not ready"))
                            } else {
                                Ok(Repository)
                            }
                        }
                    }
                },
            ))
            .unwrap();
        let container = builder.build();

        assert!(container.resolve::<Repository>().await.is_err());
        assert!(container.resolve::<Repository>().await.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn cancelled_singleton_initialization_can_retry() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::factory::<Repository, _, _>(
                Scope::Singleton,
                Vec::new(),
                {
                    let attempts = Arc::clone(&attempts);
                    let started = Arc::clone(&started);
                    move |_resolver| {
                        let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                        let started = Arc::clone(&started);
                        async move {
                            if attempt == 0 {
                                started.notify_one();
                                std::future::pending().await
                            } else {
                                Ok(Repository)
                            }
                        }
                    }
                },
            ))
            .unwrap();
        let container = builder.build();

        let first = tokio::spawn({
            let container = container.clone();
            async move { container.resolve::<Repository>().await }
        });
        started.notified().await;
        first.abort();
        assert!(first.await.unwrap_err().is_cancelled());

        assert!(container.resolve::<Repository>().await.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn optional_resolution_only_suppresses_missing_providers() {
        let container = ContainerBuilder::new().build();
        assert!(
            container
                .resolve_optional::<Repository>()
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn overrides_are_local_to_the_builder() {
        #[derive(Debug, Eq, PartialEq)]
        struct Port(u16);

        let mut production = ContainerBuilder::new();
        production
            .register(ProviderDefinition::value(Port(3000)))
            .unwrap();
        let production = production.build();

        let mut testing = ContainerBuilder::new();
        testing
            .register(ProviderDefinition::value(Port(3000)))
            .unwrap()
            .override_with(ProviderDefinition::value(Port(4000)))
            .unwrap();
        let testing = testing.build();

        assert_eq!(production.resolve::<Port>().await.unwrap().0, 3000);
        assert_eq!(testing.resolve::<Port>().await.unwrap().0, 4000);
    }

    #[test]
    fn rejects_duplicate_registrations_and_unknown_overrides() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::value(Repository))
            .unwrap();
        assert!(matches!(
            builder.register(ProviderDefinition::value(Repository)),
            Err(RegistrationError::DuplicateProvider { .. })
        ));
        assert!(matches!(
            builder.override_with(ProviderDefinition::value(Service {
                repository: Arc::new(Repository)
            })),
            Err(RegistrationError::InvalidOverride { .. })
        ));
    }

    #[tokio::test]
    async fn concrete_wrapper_tokens_support_trait_dependencies() {
        trait Clock: Send + Sync {
            fn now(&self) -> u64;
        }
        struct FixedClock;
        impl Clock for FixedClock {
            fn now(&self) -> u64 {
                42
            }
        }
        struct ClockToken(Arc<dyn Clock>);

        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::value(ClockToken(Arc::new(FixedClock))))
            .unwrap();

        assert_eq!(
            builder
                .build()
                .resolve::<ClockToken>()
                .await
                .unwrap()
                .0
                .now(),
            42
        );
    }

    #[tokio::test]
    async fn request_providers_are_shared_only_within_one_request() {
        #[derive(Debug)]
        struct RequestValue(usize);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::constructor(
                Scope::Request,
                Vec::new(),
                {
                    let calls = Arc::clone(&calls);
                    move |_resolver| Ok(RequestValue(calls.fetch_add(1, Ordering::SeqCst)))
                },
            ))
            .unwrap();
        let container = builder.build();

        let first_request = container.request_scope();
        let (first, repeated) = tokio::join!(
            first_request.resolve::<RequestValue>(),
            first_request.resolve::<RequestValue>()
        );
        let first = first.unwrap();
        let repeated = repeated.unwrap();
        assert!(Arc::ptr_eq(&first, &repeated));
        assert_eq!(first.0, 0);

        let second = container
            .request_scope()
            .resolve::<RequestValue>()
            .await
            .unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(second.0, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn request_provider_requires_an_explicit_request_scope() {
        struct RequestValue;

        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::constructor(
                Scope::Request,
                Vec::new(),
                |_resolver| Ok(RequestValue),
            ))
            .unwrap();

        assert!(matches!(
            builder.build().resolve::<RequestValue>().await,
            Err(ResolveError::RequestScopeRequired { .. })
        ));
    }

    #[tokio::test]
    async fn singleton_cannot_capture_request_scoped_state() {
        struct RequestValue;
        struct Singleton;

        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::constructor(
                Scope::Request,
                Vec::new(),
                |_resolver| Ok(RequestValue),
            ))
            .unwrap()
            .register(ProviderDefinition::factory(
                Scope::Singleton,
                vec![Dependency::required::<RequestValue>()],
                |resolver| async move {
                    resolver.resolve::<RequestValue>().await?;
                    Ok(Singleton)
                },
            ))
            .unwrap();

        assert!(matches!(
            builder.build().request_scope().resolve::<Singleton>().await,
            Err(ResolveError::ScopeViolation { .. })
        ));
    }

    #[derive(Debug)]
    struct OptionalService;

    #[tokio::test]
    async fn optional_dependency_resolves_to_some_when_present() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::value(Repository))
            .unwrap()
            .register(ProviderDefinition::factory(
                Scope::Singleton,
                vec![Dependency::optional::<Repository>()],
                |resolver| async move {
                    let _repo = resolver.resolve_optional::<Repository>().await?;
                    Ok(OptionalService)
                },
            ))
            .unwrap();

        let container = builder.build();
        assert!(container.resolve::<OptionalService>().await.is_ok());
    }

    #[tokio::test]
    async fn optional_dependency_resolves_to_none_when_missing() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::factory(
                Scope::Singleton,
                vec![Dependency::optional::<Repository>()],
                |resolver| async move {
                    let repo = resolver.resolve_optional::<Repository>().await?;
                    assert!(repo.is_none());
                    Ok(OptionalService)
                },
            ))
            .unwrap();

        let container = builder.build();
        assert!(container.resolve::<OptionalService>().await.is_ok());
    }

    #[tokio::test]
    async fn required_dependency_missing_is_error() {
        let mut builder = ContainerBuilder::new();
        builder
            .register(ProviderDefinition::factory(
                Scope::Singleton,
                vec![Dependency::required::<Repository>()],
                |resolver| async move {
                    let repository = resolver.resolve::<Repository>().await?;
                    Ok(Service { repository })
                },
            ))
            .unwrap();

        let error = builder.build().resolve::<Service>().await.unwrap_err();
        assert!(matches!(error, ResolveError::MissingProvider { .. }));
    }
}
