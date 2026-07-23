//! Safe, statically linked framework plugins.

use std::collections::HashSet;

use crate::ModuleDefinitionBuilder;

/// Plugin registration failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum PluginError {
    /// Two plugins declared the same stable name.
    #[error("IRONIC_PLUGIN_DUPLICATE: plugin `{0}` is registered more than once")]
    Duplicate(&'static str),
    /// Plugin configuration failed.
    #[error("IRONIC_PLUGIN_CONFIGURATION: plugin `{plugin}`: {message}")]
    Configuration {
        /// Stable plugin name.
        plugin: &'static str,
        /// Safe failure message.
        message: String,
    },
}

/// A statically linked extension that contributes to a module definition.
///
/// # Errors
///
/// [`apply`](Plugin::apply) returns [`PluginError::Configuration`] when
/// the plugin cannot safely register its components.
///
/// # Panics
///
/// Implementations should not panic.
pub trait Plugin: Send + Sync + 'static {
    /// Stable package-style plugin name.
    fn name(&self) -> &'static str;
    /// Plugin version for diagnostics and compatibility checks.
    fn version(&self) -> &'static str;
    /// Applies providers, controllers, imports, exports, or lifecycle hooks.
    ///
    /// # Errors
    /// Returns a safe configuration failure before module graph compilation.
    fn apply(
        &self,
        module: ModuleDefinitionBuilder,
    ) -> Result<ModuleDefinitionBuilder, PluginError>;
}

/// Ordered plugin registry with duplicate-name validation.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
    names: HashSet<&'static str>,
}

impl PluginRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a plugin.
    ///
    /// # Errors
    /// Returns [`PluginError::Duplicate`] when its name is already registered.
    pub fn register(&mut self, plugin: impl Plugin) -> Result<&mut Self, PluginError> {
        if !self.names.insert(plugin.name()) {
            return Err(PluginError::Duplicate(plugin.name()));
        }
        self.plugins.push(Box::new(plugin));
        Ok(self)
    }

    /// Applies every plugin in registration order.
    ///
    /// # Errors
    /// Returns the first plugin configuration failure.
    pub fn apply(
        &self,
        mut module: ModuleDefinitionBuilder,
    ) -> Result<ModuleDefinitionBuilder, PluginError> {
        for plugin in &self.plugins {
            module = plugin.apply(module)?;
        }
        Ok(module)
    }

    /// Returns registered plugin names and versions in registration order.
    #[must_use]
    pub fn inventory(&self) -> Vec<(&'static str, &'static str)> {
        self.plugins
            .iter()
            .map(|plugin| (plugin.name(), plugin.version()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: &'static str,
        version: &'static str,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            self.name
        }
        fn version(&self) -> &'static str {
            self.version
        }
        fn apply(
            &self,
            module: ModuleDefinitionBuilder,
        ) -> Result<ModuleDefinitionBuilder, PluginError> {
            Ok(module)
        }
    }

    #[test]
    fn register_and_inventory() {
        let mut registry = PluginRegistry::new();
        registry
            .register(TestPlugin {
                name: "alpha",
                version: "1.0.0",
            })
            .unwrap()
            .register(TestPlugin {
                name: "beta",
                version: "2.0.0",
            })
            .unwrap();

        let inv = registry.inventory();
        assert_eq!(
            inv,
            vec![("alpha", "1.0.0"), ("beta", "2.0.0")]
        );
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut registry = PluginRegistry::new();
        registry
            .register(TestPlugin {
                name: "dup",
                version: "1.0.0",
            })
            .unwrap();
        let result = registry.register(TestPlugin {
            name: "dup",
            version: "2.0.0",
        });
        assert!(matches!(result, Err(PluginError::Duplicate("dup"))));
    }

    #[test]
    fn plugin_error_display() {
        let err = PluginError::Duplicate("my_plugin");
        assert_eq!(
            err.to_string(),
            "IRONIC_PLUGIN_DUPLICATE: plugin `my_plugin` is registered more than once"
        );

        let err = PluginError::Configuration {
            plugin: "my_plugin",
            message: "missing config key".into(),
        };
        assert_eq!(
            err.to_string(),
            "IRONIC_PLUGIN_CONFIGURATION: plugin `my_plugin`: missing config key"
        );
    }

    #[test]
    fn plugin_error_clone_eq() {
        let a = PluginError::Duplicate("x");
        let b = PluginError::Duplicate("x");
        assert_eq!(a, b);
    }
}
