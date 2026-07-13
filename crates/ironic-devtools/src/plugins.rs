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
