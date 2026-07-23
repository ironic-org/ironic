use std::{future::Future, sync::Arc};

use ironic_core::{
    CompiledApplicationGraph, Module, build_http_application_with_overrides, compile_module_graph,
};
use ironic_di::{Container, Dependency, ProviderDefinition, ResolveError, Scope};

use crate::TestBuildError;

/// Entry point for compiling a module with test-local provider overrides.
///
/// `TestModule` compiles a [`Module`] graph in isolation — no HTTP server,
/// no platform adapter, no lifecycle orchestration. Useful for testing
/// provider resolution without the overhead of a full [`TestApplication`].
///
/// # Example
///
/// ```rust,ignore
/// # use ironic::{Module, ModuleDefinition, TestModule};
/// # struct MyModule;
/// # impl Module for MyModule {
/// #     fn definition() -> ModuleDefinition {
/// #         ModuleDefinition::builder::<Self>().build()
/// #     }
/// # }
/// let compiled = TestModule::builder::<MyModule>().compile().await.unwrap();
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct TestModule;

impl TestModule {
    /// Starts an isolated compiler for module `M`.
    #[must_use]
    pub fn builder<M: Module>() -> TestModuleBuilder {
        TestModuleBuilder {
            root: M::definition(),
            overrides: Vec::new(),
        }
    }
}

/// Builds an isolated module container.
///
/// Collects provider overrides that shadow the module's original registrations
/// before compilation. See [`TestModule::builder()`] for usage.
pub struct TestModuleBuilder {
    root: ironic_core::ModuleDefinition,
    overrides: Vec<ProviderDefinition>,
}

impl TestModuleBuilder {
    /// Replaces a provider with a complete definition using the same concrete key.
    #[must_use]
    pub fn override_provider(mut self, provider: ProviderDefinition) -> Self {
        self.overrides.push(provider);
        self
    }

    /// Replaces provider `T` with a singleton test value.
    #[must_use]
    pub fn override_value<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        self.overrides.push(ProviderDefinition::value(value));
        self
    }

    /// Replaces provider `T` with an asynchronous factory.
    #[must_use]
    pub fn override_factory<T, F, Fut>(
        mut self,
        scope: Scope,
        dependencies: Vec<Dependency>,
        factory: F,
    ) -> Self
    where
        T: Send + Sync + 'static,
        F: Fn(ironic_di::Resolver) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, ResolveError>> + Send + 'static,
    {
        self.overrides
            .push(ProviderDefinition::factory(scope, dependencies, factory));
        self
    }

    /// Compiles the module graph, applies overrides, and initializes eager providers.
    ///
    /// # Errors
    ///
    /// Returns [`TestBuildError`] for invalid graphs, overrides, routes, or eager providers.
    pub async fn compile(self) -> Result<CompiledTestModule, TestBuildError> {
        let graph = compile_module_graph(self.root)?;
        let application = build_http_application_with_overrides(&graph, self.overrides)?;
        let container = application.container().clone();
        for module_id in graph.initialization_order() {
            let Some(module) = graph.module(*module_id) else {
                continue;
            };
            for provider in module
                .providers()
                .iter()
                .filter(|provider| provider.is_eager())
            {
                container.resolve_key(provider.key()).await?;
            }
        }
        Ok(CompiledTestModule { graph, container })
    }
}

/// A validated module graph and its isolated dependency container.
pub struct CompiledTestModule {
    graph: CompiledApplicationGraph,
    container: Container,
}

impl CompiledTestModule {
    /// Returns the validated module graph.
    #[must_use]
    pub const fn graph(&self) -> &CompiledApplicationGraph {
        &self.graph
    }

    /// Resolves a provider from the isolated container.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError`] when the provider is missing or construction fails.
    pub async fn resolve<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, ResolveError> {
        self.container.resolve::<T>().await
    }
}

#[cfg(test)]
mod tests {
    use ironic_core::{Module, ModuleDefinition, ModuleId};

    use super::*;

    struct EmptyTestModule;
    impl Module for EmptyTestModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>().build()
        }
    }

    struct WithStringModule;
    impl Module for WithStringModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(ProviderDefinition::value("original".to_string()))
                .build()
        }
    }

    struct WithU64Module;
    impl Module for WithU64Module {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>()
                .provider(ProviderDefinition::value(42u64))
                .build()
        }
    }

    #[tokio::test]
    async fn builder_defaults() {
        let builder = TestModule::builder::<EmptyTestModule>();
        assert!(builder.overrides.is_empty());
    }

    #[tokio::test]
    async fn compile_empty_module() {
        let compiled = TestModule::builder::<EmptyTestModule>()
            .compile()
            .await
            .unwrap();
        // Root module is always in the initialization order.
        assert_eq!(compiled.graph().initialization_order().len(), 1);
    }

    #[tokio::test]
    async fn override_value_is_applied() {
        let compiled = TestModule::builder::<WithStringModule>()
            .override_value("overridden".to_string())
            .compile()
            .await
            .unwrap();
        assert_eq!(
            compiled.resolve::<String>().await.unwrap().as_str(),
            "overridden"
        );
    }

    #[tokio::test]
    async fn override_provider_chains() {
        let provider = ProviderDefinition::value(42u64);
        let builder = TestModule::builder::<EmptyTestModule>().override_provider(provider);
        assert_eq!(builder.overrides.len(), 1);
    }

    #[tokio::test]
    async fn override_factory_chains() {
        let builder = TestModule::builder::<EmptyTestModule>().override_factory::<String, _, _>(
            Scope::Transient,
            Vec::new(),
            |_resolver| async { Ok("factory-value".to_string()) },
        );
        assert_eq!(builder.overrides.len(), 1);
    }

    #[tokio::test]
    async fn compile_with_override_provider() {
        let compiled = TestModule::builder::<WithU64Module>()
            .override_provider(ProviderDefinition::value(100u64))
            .compile()
            .await
            .unwrap();
        assert_eq!(*compiled.resolve::<u64>().await.unwrap(), 100);
    }

    #[tokio::test]
    async fn compile_with_override_factory() {
        let compiled = TestModule::builder::<WithStringModule>()
            .override_factory::<String, _, _>(Scope::Singleton, Vec::new(), |_resolver| async {
                Ok("factory-resolved".to_string())
            })
            .compile()
            .await
            .unwrap();
        assert_eq!(
            compiled.resolve::<String>().await.unwrap().as_str(),
            "factory-resolved"
        );
    }

    #[tokio::test]
    async fn compiled_module_returns_graph() {
        let compiled = TestModule::builder::<EmptyTestModule>()
            .compile()
            .await
            .unwrap();
        let graph = compiled.graph();
        assert_eq!(graph.root(), ModuleId::of::<EmptyTestModule>());
    }

    #[tokio::test]
    async fn compile_adds_providers_from_module() {
        let compiled = TestModule::builder::<WithU64Module>()
            .compile()
            .await
            .unwrap();
        assert_eq!(*compiled.resolve::<u64>().await.unwrap(), 42);
    }
}
