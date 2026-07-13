#![doc = "Typed, validated configuration with redacted secret values for `RustFrame`."]

use std::{fmt, path::Path};

use config::{Config, Environment, File, FileFormat, builder::DefaultState};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Validates a fully deserialized application configuration.
pub trait ValidateConfiguration {
    /// Returns an actionable validation message when configuration is invalid.
    ///
    /// # Errors
    ///
    /// Returns a message safe to show in startup diagnostics.
    fn validate(&self) -> Result<(), String>;
}

/// A typed configuration loading or validation failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigurationError {
    /// A source could not be loaded or deserialized.
    #[error("RF_CONFIG_SOURCE: {0}")]
    Source(#[from] config::ConfigError),
    /// The typed value failed application validation.
    #[error("RF_CONFIG_VALIDATION: {message}")]
    Validation {
        /// Safe validation diagnostic.
        message: String,
    },
}

/// Builds a typed configuration from layered files, JSON, and environment variables.
pub struct ConfigurationLoader {
    builder: config::ConfigBuilder<DefaultState>,
}

impl ConfigurationLoader {
    /// Creates an empty loader.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builder: Config::builder(),
        }
    }

    /// Adds a required TOML configuration file.
    #[must_use]
    pub fn file(mut self, path: impl AsRef<Path>) -> Self {
        self.builder = self
            .builder
            .add_source(File::from(path.as_ref().to_owned()).required(true));
        self
    }

    /// Adds a JSON configuration layer.
    #[must_use]
    pub fn json(mut self, source: &str) -> Self {
        self.builder = self
            .builder
            .add_source(File::from_str(source, FileFormat::Json));
        self
    }

    /// Adds environment variables using `__` for nested keys.
    ///
    /// For prefix `APP`, `APP__SERVER__PORT=3000` maps to `server.port`.
    #[must_use]
    pub fn environment(mut self, prefix: &str) -> Self {
        self.builder = self.builder.add_source(
            Environment::with_prefix(prefix)
                .prefix_separator("__")
                .separator("__")
                .try_parsing(true),
        );
        self
    }

    /// Deserializes and validates the final typed configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigurationError`] when a source, deserialization, or validation step fails.
    pub fn load<T>(self) -> Result<T, ConfigurationError>
    where
        T: DeserializeOwned + ValidateConfiguration,
    {
        let configuration = self.builder.build()?.try_deserialize::<T>()?;
        configuration
            .validate()
            .map_err(|message| ConfigurationError::Validation { message })?;
        Ok(configuration)
    }
}

impl Default for ConfigurationLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// A sensitive configuration value whose formatting and serialization are redacted.
#[derive(Clone, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wraps a secret value.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Exposes the secret to code that explicitly needs it.
    #[must_use]
    pub const fn expose_secret(&self) -> &T {
        &self.0
    }

    /// Consumes the wrapper and returns the secret.
    #[must_use]
    pub fn into_secret(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret([REDACTED])")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl<T> Serialize for Secret<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

/// A secret UTF-8 string.
pub type SecretString = Secret<String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct ApplicationConfig {
        port: u16,
        token: SecretString,
    }

    impl ValidateConfiguration for ApplicationConfig {
        fn validate(&self) -> Result<(), String> {
            if self.port == 0 {
                Err("port must be greater than zero".to_owned())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn loads_and_validates_typed_json() {
        let configuration = ConfigurationLoader::new()
            .json(r#"{"port":3000,"token":"private"}"#)
            .load::<ApplicationConfig>()
            .unwrap();
        assert_eq!(configuration.port, 3000);
        assert_eq!(configuration.token.expose_secret(), "private");
    }

    #[test]
    fn rejects_invalid_typed_configuration() {
        let error = ConfigurationLoader::new()
            .json(r#"{"port":0,"token":"private"}"#)
            .load::<ApplicationConfig>()
            .unwrap_err();
        assert!(error.to_string().contains("port must be greater than zero"));
    }

    #[test]
    fn secrets_are_redacted_in_all_safe_outputs() {
        let secret = SecretString::new("private".to_owned());
        assert_eq!(format!("{secret}"), "[REDACTED]");
        assert_eq!(format!("{secret:?}"), "Secret([REDACTED])");
        assert_eq!(serde_json::to_string(&secret).unwrap(), r#""[REDACTED]""#);
    }
}
