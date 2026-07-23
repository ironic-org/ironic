use ironic_core::{ApplicationError, HttpApplicationBuildError, ModuleError};
use ironic_di::ResolveError;

/// A failure while compiling an isolated test module.
///
/// This error aggregates the various failure modes that can occur during
/// [`TestModule::builder()`](crate::TestModule::builder())`.compile().await`:
/// invalid module graphs, provider registration failures, eager provider
/// resolution errors, and application initialization failures.
///
/// # Example
///
/// ```rust,ignore
/// use ironic::{TestBuildError, TestModule};
/// # use ironic::{Module, ModuleDefinition};
/// # struct BadModule;
/// # impl Module for BadModule {
/// #     fn definition() -> ModuleDefinition {
/// #         ModuleDefinition::builder::<Self>()
/// #             .build()
/// #     }
/// # }
/// let result = TestModule::builder::<BadModule>().compile().await;
/// match result {
///     Err(_) => {} // expected for misconfigured modules
///     Ok(_) => {}
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TestBuildError {
    /// The module graph is invalid.
    #[error(transparent)]
    Module(#[from] ModuleError),
    /// Provider registration or route compilation failed.
    #[error(transparent)]
    Http(#[from] HttpApplicationBuildError),
    /// An eager provider could not be resolved.
    #[error(transparent)]
    Resolve(#[from] ResolveError),
    /// A complete test application could not initialize.
    #[error(transparent)]
    Application(#[from] ApplicationError),
}

#[cfg(test)]
mod tests {
    use ironic_core::{Module, ModuleDefinition, ModuleError, ModuleId};
    use ironic_di::{ProviderKey, RegistrationError};

    use super::*;

    struct TestErrModule;
    impl Module for TestErrModule {
        fn definition() -> ModuleDefinition {
            ModuleDefinition::builder::<Self>().build()
        }
    }

    #[test]
    fn display_module_error() {
        let err = TestBuildError::Module(ModuleError::MissingProvider {
            module: ModuleId::of::<TestErrModule>(),
            consumer: ProviderKey::of::<String>(),
            provider: ProviderKey::of::<u64>(),
        });
        let display = err.to_string();
        assert!(display.contains("RF_MODULE_MISSING_PROVIDER"));
    }

    #[test]
    fn display_http_error() {
        let err = TestBuildError::Http(HttpApplicationBuildError::ProviderRegistration(
            RegistrationError::DuplicateProvider {
                key: ProviderKey::of::<String>(),
            },
        ));
        let display = err.to_string();
        assert!(display.contains("RF_DI_DUPLICATE_PROVIDER"));
    }

    #[test]
    fn display_resolve_error() {
        let err = TestBuildError::Resolve(ResolveError::MissingProvider {
            key: ProviderKey::of::<String>(),
            path: Vec::new(),
        });
        let display = err.to_string();
        assert!(display.contains("RF_DI_MISSING_PROVIDER"));
    }

    #[test]
    fn display_application_error() {
        let err = TestBuildError::Application(ApplicationError::MissingRootModule);
        let display = err.to_string();
        assert!(display.contains("RF_APP_MISSING_ROOT_MODULE"));
    }

    #[test]
    fn from_module() {
        let inner = ModuleError::MissingProvider {
            module: ModuleId::of::<TestErrModule>(),
            consumer: ProviderKey::of::<String>(),
            provider: ProviderKey::of::<u64>(),
        };
        let err: TestBuildError = inner.into();
        assert!(matches!(err, TestBuildError::Module(_)));
    }

    #[test]
    fn from_http() {
        let inner =
            HttpApplicationBuildError::ProviderRegistration(RegistrationError::DuplicateProvider {
                key: ProviderKey::of::<String>(),
            });
        let err: TestBuildError = inner.into();
        assert!(matches!(err, TestBuildError::Http(_)));
    }

    #[test]
    fn from_resolve() {
        let inner = ResolveError::MissingProvider {
            key: ProviderKey::of::<String>(),
            path: Vec::new(),
        };
        let err: TestBuildError = inner.into();
        assert!(matches!(err, TestBuildError::Resolve(_)));
    }

    #[test]
    fn from_application() {
        let err: TestBuildError = ApplicationError::MissingRootModule.into();
        assert!(matches!(err, TestBuildError::Application(_)));
    }
}
