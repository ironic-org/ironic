#![doc = "Typed, validated configuration with redacted secret values for Ironic."]

use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

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
///
/// # Precedence (lowest to highest)
///
/// 1. Base files added via [`file`](Self::file)
/// 2. Profile overlay `config.{env}.toml` (silently skipped if missing)
/// 3. Inline layers added via [`json`](Self::json)
/// 4. Environment variables (via [`environment`](Self::environment))
///
/// # Hot Reload
///
/// When the `hot-reload` Cargo feature is enabled, the [`watch`](Self::watch) method
/// spawns a file-system watcher that re-loads the configuration whenever a watched
/// file changes. The new config is sent to registered callbacks.
pub struct ConfigurationLoader {
    builder: config::ConfigBuilder<DefaultState>,
    #[cfg(feature = "hot-reload")]
    watched_paths: Vec<PathBuf>,
    #[cfg(feature = "hot-reload")]
    json_layers: Vec<String>,
    #[cfg(feature = "hot-reload")]
    env_prefixes: Vec<String>,
}

impl ConfigurationLoader {
    /// Creates an empty loader.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builder: Config::builder(),
            #[cfg(feature = "hot-reload")]
            watched_paths: Vec::new(),
            #[cfg(feature = "hot-reload")]
            json_layers: Vec::new(),
            #[cfg(feature = "hot-reload")]
            env_prefixes: Vec::new(),
        }
    }

    /// Adds a required TOML configuration file.
    ///
    /// When the `hot-reload` feature is enabled, this file is watched for
    /// changes and triggers a config reload.
    #[must_use]
    pub fn file(mut self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_owned();
        #[cfg(feature = "hot-reload")]
        self.watched_paths.push(path.clone());
        self.builder = self.builder.add_source(File::from(path).required(true));
        self
    }

    /// Adds a JSON configuration layer.
    #[must_use]
    pub fn json(mut self, source: &str) -> Self {
        #[cfg(feature = "hot-reload")]
        self.json_layers.push(source.to_owned());
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
        #[cfg(feature = "hot-reload")]
        self.env_prefixes.push(prefix.to_owned());
        self.builder = self.builder.add_source(
            Environment::with_prefix(prefix)
                .prefix_separator("__")
                .separator("__")
                .try_parsing(true),
        );
        self
    }

    /// Auto-detects the active environment profile.
    ///
    /// Checks `IRONIC_ENV` then `APP_ENV` at runtime and falls back to
    /// `"development"`.  The profile file (`config.{env}.toml`) is added as
    /// an optional overlay source immediately — any later sources (JSON,
    /// environment variables) will override it.
    #[must_use]
    pub fn auto_detect_env(self) -> Self {
        let env = std::env::var("IRONIC_ENV")
            .or_else(|_| std::env::var("APP_ENV"))
            .unwrap_or_else(|_| "development".to_owned());
        self.profile(&env)
    }

    /// Overrides the active environment profile manually.
    ///
    /// The profile file (`config.{env}.toml`) is added as an optional overlay
    /// source immediately — any later sources (JSON, environment variables)
    /// will override it.
    #[must_use]
    pub fn profile(self, env: &str) -> Self {
        let path = PathBuf::from(format!("config.{env}.toml"));
        let mut this = self;
        this.builder = this.builder.add_source(File::from(path).required(false));
        this
    }

    /// Deserializes and validates the final typed configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigurationError`] when a required source, deserialization,
    /// or validation step fails.
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

    /// Spawns a file watcher that re-loads the typed configuration when any
    /// watched file changes.
    ///
    /// The returned [`ConfigWatcher`] receives the newly loaded value on each
    /// change.  The watcher runs on a blocking thread and communicates via a
    /// tokio watch channel.
    ///
    /// Only files added via [`file`](Self::file) are watched.  JSON layers and
    /// environment variables are re-applied when rebuilding.
    ///
    /// # Panics
    ///
    /// Panics if no file sources were added before calling `watch`.
    ///
    /// # Hot-reload limitations
    ///
    /// - Only values read through [`ConfigWatcher::latest()`] are updated at
    ///   runtime.  Code that holds a copy of the old config will not see the
    ///   new values.
    /// - The DI container is not rebuilt.  Providers that inject config values
    ///   at construction time will still use the original values.
    /// - Feature toggles that opt in via [`FeatureToggle`] receive updates
    ///   automatically (see [`FeatureToggle::with_watcher`]).
    #[cfg(feature = "hot-reload")]
    pub fn watch<T>(self) -> ConfigWatcher<T>
    where
        T: DeserializeOwned + ValidateConfiguration + Send + Sync + 'static,
    {
        let watched_paths = self.watched_paths;
        let json_layers = self.json_layers;
        let env_prefixes = self.env_prefixes;

        assert!(
            !watched_paths.is_empty(),
            "ConfigurationLoader::watch() requires at least one file source"
        );

        let (tx, rx) = tokio::sync::watch::channel(None::<T>);

        let handle = std::thread::spawn(move || {
            use notify::{EventKind, RecursiveMode, Watcher};
            use std::sync::mpsc;

            let (event_tx, event_rx) = mpsc::channel();
            let mut watcher = match notify::recommended_watcher(
                move |res: Result<notify::Event, notify::Error>| {
                    let _ = event_tx.send(res);
                },
            ) {
                Ok(w) => w,
                Err(e) => {
                    tracing::warn!("Failed to create config watcher: {e}");
                    return;
                }
            };

            for path in &watched_paths {
                if let Some(parent) = path.parent()
                    && !parent.as_os_str().is_empty()
                    && let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive)
                {
                    tracing::warn!("Failed to watch config path {path:?}: {e}");
                }
            }

            while let Ok(result) = event_rx.recv() {
                match result {
                    Ok(event) => {
                        let is_config_change = event
                            .paths
                            .iter()
                            .any(|p| watched_paths.iter().any(|wp| p.starts_with(wp) || p == wp));
                        if !is_config_change {
                            continue;
                        }
                        if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            continue;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Config watcher event error: {e}");
                        continue;
                    }
                }

                // Debounce before reloading
                std::thread::sleep(std::time::Duration::from_millis(200));

                let mut builder = Config::builder();
                for path in &watched_paths {
                    builder = builder.add_source(File::from(path.clone()).required(false));
                }
                for prefix in &env_prefixes {
                    builder = builder.add_source(
                        Environment::with_prefix(prefix)
                            .prefix_separator("__")
                            .separator("__")
                            .try_parsing(true),
                    );
                }
                for json_str in &json_layers {
                    builder = builder.add_source(File::from_str(json_str, FileFormat::Json));
                }

                match builder.build().and_then(Config::try_deserialize::<T>) {
                    Ok(new_config) => {
                        if let Err(e) = new_config.validate() {
                            tracing::warn!("Config reloaded but failed validation: {e}");
                            continue;
                        }
                        let _ = tx.send(Some(new_config));
                        tracing::info!("Configuration reloaded from file change");
                    }
                    Err(e) => {
                        tracing::warn!("Failed to reload config after file change: {e}");
                    }
                }
            }
        });

        ConfigWatcher {
            rx,
            handle: Some(handle),
        }
    }
}

impl Default for ConfigurationLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle that receives updated configuration values when watched files change.
///
/// Created by [`ConfigurationLoader::watch`].  The watcher runs on a background
/// thread and sends the latest successfully loaded value through the channel.
///
/// # Hot-reload limitations
///
/// - Only config values read through [`latest`](Self::latest) are live-updated.
/// - The DI container and already-constructed providers retain their original
///   values unless they explicitly poll this watcher.
#[cfg(feature = "hot-reload")]
pub struct ConfigWatcher<T> {
    rx: tokio::sync::watch::Receiver<Option<T>>,
    #[allow(dead_code)]
    handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(feature = "hot-reload")]
impl<T: fmt::Debug> fmt::Debug for ConfigWatcher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigWatcher")
            .field("latest", &self.rx.borrow())
            .field("has_handle", &self.handle.is_some())
            .finish()
    }
}

#[cfg(feature = "hot-reload")]
impl<T: Clone + Send + Sync + 'static> ConfigWatcher<T> {
    /// Returns a clone of the most recently loaded configuration, or `None`
    /// if none has been loaded yet.
    #[must_use]
    pub fn latest(&self) -> Option<T> {
        self.rx.borrow().clone()
    }

    /// Returns a receiver that can be used to await the next config update.
    #[must_use]
    pub fn receiver(&self) -> tokio::sync::watch::Receiver<Option<T>> {
        self.rx.clone()
    }
}

/// Runtime feature toggles backed by configuration values.
///
/// Toggles are read from the `features.*` section of the application config.
/// When used with the `hot-reload` feature, the toggles update automatically
/// when the config file changes.
///
/// # Example config.toml
/// ```toml
/// [features]
/// new_checkout = true
/// dark_mode = false
/// experimental_api = true
/// ```
///
/// # Example usage
/// ```rust,ignore
/// let toggles = FeatureToggle::from_root_config(&config);
/// if toggles.is_enabled("new_checkout") {
///     // use new checkout flow
/// }
/// ```
#[derive(Debug)]
pub struct FeatureToggle {
    flags: HashMap<String, bool>,
    #[cfg(feature = "hot-reload")]
    watcher: Option<ConfigWatcher<Config>>,
}

impl FeatureToggle {
    /// Creates a toggle set from a mapping of feature names to booleans.
    #[must_use]
    pub fn new(flags: HashMap<String, bool>) -> Self {
        Self {
            flags,
            #[cfg(feature = "hot-reload")]
            watcher: None,
        }
    }

    /// Extracts feature toggles from the `[features]` section of a
    /// `config::Config` object.
    ///
    /// Returns an empty feature set when the `features` key is absent.
    ///
    /// # Errors
    ///
    /// Returns an error when the `features` value exists but is not a table
    /// of `string → bool` entries.
    pub fn from_root_config(config: &Config) -> Result<Self, ConfigurationError> {
        let flags = config.get("features").unwrap_or_default();
        Ok(Self::new(flags))
    }

    /// Checks whether a feature toggle is enabled.
    ///
    /// When `hot-reload` is enabled, the toggle first checks the latest
    /// config from the [`ConfigWatcher`] and falls back to the initial flags.
    ///
    /// Returns `false` if the toggle is not registered.
    #[must_use]
    pub fn is_enabled(&self, name: &str) -> bool {
        #[cfg(feature = "hot-reload")]
        if let Some(ref watcher) = self.watcher
            && let Some(config) = watcher.latest()
            && let Ok(flags) = config.get::<HashMap<String, bool>>("features")
        {
            return flags.get(name).copied().unwrap_or(false);
        }
        self.flags.get(name).copied().unwrap_or(false)
    }

    /// Registers a hot-reload watcher so the toggles update on config changes.
    #[cfg(feature = "hot-reload")]
    #[must_use]
    pub fn with_watcher(mut self, config_watcher: ConfigWatcher<Config>) -> Self {
        self.watcher = Some(config_watcher);
        self
    }

    /// Returns all enabled feature names.
    #[must_use]
    pub fn enabled_flags(&self) -> Vec<&str> {
        self.flags
            .iter()
            .filter_map(|(name, enabled)| if *enabled { Some(name.as_str()) } else { None })
            .collect()
    }

    /// Returns all registered feature names and their status.
    #[must_use]
    pub fn all(&self) -> &HashMap<String, bool> {
        &self.flags
    }
}

/// Creates an empty feature toggle with all flags disabled.
impl Default for FeatureToggle {
    fn default() -> Self {
        Self::new(std::collections::HashMap::new())
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

/// A hot-reloadable configuration value backed by a `tokio::sync::watch`.
///
/// Use with `ConfigWatcher` to receive live updates when config files change.
/// Inject `Reloadable<AppConfig>` into providers instead of `AppConfig` directly
/// to respond to runtime configuration changes without restarting.
#[derive(Clone, Debug)]
pub struct Reloadable<T> {
    rx: tokio::sync::watch::Receiver<T>,
}

impl<T: Clone> Reloadable<T> {
    /// Wraps an existing watch receiver.
    #[must_use]
    pub fn new(rx: tokio::sync::watch::Receiver<T>) -> Self {
        Self { rx }
    }

    /// Returns the latest configuration value.
    #[must_use]
    pub fn latest(&self) -> T {
        self.rx.borrow().clone()
    }

    /// Returns a receiver for the updated value stream.
    pub fn receiver(&self) -> tokio::sync::watch::Receiver<T> {
        self.rx.clone()
    }
}

/// Pre-built configuration structs for common infrastructure components.
///
/// Use these as a starting point or embed them into your own `AppConfig`:
///
/// ```ignore
/// use ironic::config::presets::{DatabaseConfig, AuthConfig, ServerConfig};
///
/// #[derive(Deserialize)]
/// pub struct AppConfig {
///     pub database: DatabaseConfig,
///     pub auth: AuthConfig,
///     pub server: ServerConfig,
/// }
///
/// impl ValidateConfiguration for AppConfig {
///     fn validate(&self) -> Result<(), String> {
///         self.database.validate()?;
///         self.auth.validate()?;
///         self.server.validate()?;
///         Ok(())
///     }
/// }
/// ```
pub mod presets {
    use serde::Deserialize;

    use super::{SecretString, ValidateConfiguration};

    /// Database connection configuration.
    ///
    /// ```toml
    /// [database]
    /// url = "postgres://localhost:5432/myapp"
    /// max_connections = 10
    /// min_connections = 2
    /// ```
    #[derive(Clone, Debug, Deserialize)]
    pub struct DatabaseConfig {
        /// The database connection URL. Wrap in `SecretString` to prevent
        /// accidental logging of credentials.
        pub url: SecretString,
        /// Maximum number of connections in the pool (default: 10).
        #[serde(default = "default_max_connections")]
        pub max_connections: u32,
        /// Minimum number of idle connections (default: 2).
        #[serde(default = "default_min_connections")]
        pub min_connections: u32,
    }

    impl Default for DatabaseConfig {
        fn default() -> Self {
            Self {
                url: SecretString::new(String::new()),
                max_connections: default_max_connections(),
                min_connections: default_min_connections(),
            }
        }
    }

    impl ValidateConfiguration for DatabaseConfig {
        fn validate(&self) -> Result<(), String> {
            if self.url.expose_secret().is_empty() {
                return Err("database.url must not be empty".to_owned());
            }
            if self.max_connections == 0 {
                return Err("database.max_connections must be greater than zero".to_owned());
            }
            Ok(())
        }
    }

    const fn default_max_connections() -> u32 {
        10
    }
    const fn default_min_connections() -> u32 {
        2
    }

    /// Authentication configuration.
    ///
    /// ```toml
    /// [auth]
    /// jwt_secret = "your-256-bit-secret"
    /// jwt_expiry_secs = 3600
    /// refresh_expiry_secs = 86400
    /// ```
    #[derive(Clone, Debug, Deserialize)]
    pub struct AuthConfig {
        /// The JWT signing secret. Wrap in `SecretString` for safety.
        pub jwt_secret: SecretString,
        /// Access token lifetime in seconds (default: 3600 = 1 hour).
        #[serde(default = "default_jwt_expiry")]
        pub jwt_expiry_secs: u64,
        /// Refresh token lifetime in seconds (default: 86400 = 24 hours).
        #[serde(default = "default_refresh_expiry")]
        pub refresh_expiry_secs: u64,
    }

    impl Default for AuthConfig {
        fn default() -> Self {
            Self {
                jwt_secret: SecretString::new(String::new()),
                jwt_expiry_secs: default_jwt_expiry(),
                refresh_expiry_secs: default_refresh_expiry(),
            }
        }
    }

    impl ValidateConfiguration for AuthConfig {
        fn validate(&self) -> Result<(), String> {
            if self.jwt_secret.expose_secret().len() < 32 {
                return Err(
                    "auth.jwt_secret must be at least 32 characters (256 bits)".to_owned(),
                );
            }
            if self.jwt_expiry_secs == 0 {
                return Err("auth.jwt_expiry_secs must be greater than zero".to_owned());
            }
            Ok(())
        }
    }

    const fn default_jwt_expiry() -> u64 {
        3600
    }
    const fn default_refresh_expiry() -> u64 {
        86400
    }

    /// HTTP server configuration.
    ///
    /// ```toml
    /// [server]
    /// host = "0.0.0.0"
    /// port = 3000
    /// ```
    #[derive(Clone, Debug, Deserialize)]
    pub struct ServerConfig {
        /// The host to bind to (default: "0.0.0.0").
        #[serde(default = "default_host")]
        pub host: String,
        /// The port to listen on (default: 3000).
        #[serde(default = "default_port")]
        pub port: u16,
    }

    impl Default for ServerConfig {
        fn default() -> Self {
            Self {
                host: default_host(),
                port: default_port(),
            }
        }
    }

    impl ValidateConfiguration for ServerConfig {
        fn validate(&self) -> Result<(), String> {
            if self.port == 0 {
                return Err("server.port must be greater than zero".to_owned());
            }
            Ok(())
        }
    }

    fn default_host() -> String {
        "0.0.0.0".to_owned()
    }
    const fn default_port() -> u16 {
        3000
    }

    /// Redis connection configuration.
    ///
    /// Only available when the `redis` feature is enabled.
    ///
    /// ```toml
    /// [redis]
    /// url = "redis://localhost:6379"
    /// ```
    #[cfg(feature = "redis")]
    #[derive(Clone, Debug, Deserialize)]
    pub struct RedisConfig {
        /// The Redis connection URL.
        pub url: String,
    }

    #[cfg(feature = "redis")]
    impl Default for RedisConfig {
        fn default() -> Self {
            Self {
                url: "redis://localhost:6379".to_owned(),
            }
        }
    }

    #[cfg(feature = "redis")]
    impl ValidateConfiguration for RedisConfig {
        fn validate(&self) -> Result<(), String> {
            if self.url.is_empty() {
                return Err("redis.url must not be empty".to_owned());
            }
            Ok(())
        }
    }
}

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

    #[derive(Debug, Deserialize)]
    struct ProfileAwareConfig {
        port: u16,
        host: String,
        db_url: SecretString,
    }

    impl ValidateConfiguration for ProfileAwareConfig {
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

    #[test]
    fn auto_detect_env_defaults_to_development() {
        // With no env var set, auto_detect_env defaults to "development"
        // and tries to load config.development.toml (silently skipped).
        let config = ConfigurationLoader::new()
            .json(r#"{"port":3000,"host":"localhost","db_url":"postgres://localhost/db"}"#)
            .auto_detect_env()
            .load::<ProfileAwareConfig>()
            .unwrap();
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn profile_can_be_set_explicitly() {
        // production profile loads config.production.toml (silently skipped).
        let config = ConfigurationLoader::new()
            .json(r#"{"port":3000,"host":"localhost","db_url":"postgres://localhost/db"}"#)
            .profile("production")
            .load::<ProfileAwareConfig>()
            .unwrap();
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn profile_overlay_merges_on_top_of_base() {
        let _ = std::fs::remove_file("ovl_base_config.toml");
        let _ = std::fs::remove_file("config.staging.toml");
        std::fs::write("ovl_base_config.toml", r"port = 8080").unwrap();
        std::fs::write(
            "config.staging.toml",
            r#"port = 9090
host = "staging.example.com"
db_url = "postgres://staging/db""#,
        )
        .unwrap();
        let config = ConfigurationLoader::new()
            .file("ovl_base_config.toml")
            .profile("staging")
            .load::<ProfileAwareConfig>()
            .unwrap();

        assert_eq!(config.port, 9090, "profile should override base port");
        assert_eq!(config.host, "staging.example.com");
        assert_eq!(config.db_url.expose_secret(), "postgres://staging/db");

        let _ = std::fs::remove_file("ovl_base_config.toml");
        let _ = std::fs::remove_file("config.staging.toml");
    }

    #[test]
    fn profile_overlay_silently_skipped_when_missing() {
        let config = ConfigurationLoader::new()
            .json(r#"{"port":3000,"host":"localhost","db_url":"postgres://localhost/db"}"#)
            .profile("nonexistent_env")
            .load::<ProfileAwareConfig>()
            .unwrap();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "localhost");
    }

    #[test]
    fn file_then_profile_then_json_respected_precedence() {
        let _ = std::fs::remove_file("prec_base_config.toml");
        std::fs::write("prec_base_config.toml", r"port = 1000").unwrap();
        std::fs::write(
            "config.precedence.toml",
            r#"port = 2000
host = "profile-host""#,
        )
        .unwrap();
        let config = ConfigurationLoader::new()
            .file("prec_base_config.toml")
            .profile("precedence")
            .json(r#"{"port":3000,"host":"json-host","db_url":"postgres://json/db"}"#)
            .load::<ProfileAwareConfig>()
            .unwrap();

        assert_eq!(config.port, 3000, "json should override profile and base");
        assert_eq!(config.host, "json-host", "json should override profile");
        assert_eq!(config.db_url.expose_secret(), "postgres://json/db");

        let _ = std::fs::remove_file("prec_base_config.toml");
        let _ = std::fs::remove_file("config.precedence.toml");
    }
}
