//! Typed metadata types inserted into [`RouteMetadata`] for route and
//! controller-level capabilities.

/// Strategy for API versioning.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VersioningStrategy {
    /// Version prefix in the URI path (e.g., `/v1/users`).
    Uri,
    /// Version specified by the `Accept-Version` header.
    Header,
    /// Version specified by a media type parameter in `Accept`.
    MediaType,
}

/// Version metadata attached to a controller definition.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VersionMetadata {
    /// The version identifier (e.g., "1", "2024-01-01").
    pub version: String,
    /// The strategy used to match this version.
    pub strategy: VersioningStrategy,
}

impl VersionMetadata {
    /// Creates new version metadata.
    #[must_use]
    pub fn new(version: impl Into<String>, strategy: VersioningStrategy) -> Self {
        Self {
            version: version.into(),
            strategy,
        }
    }

    /// Returns the URI prefix for URI-based versioning (e.g., `/v1`).
    #[must_use]
    pub fn uri_prefix(&self) -> String {
        format!("/v{}", self.version)
    }
}

/// Cache configuration attached to a route definition.
#[derive(Clone, Debug)]
pub struct CacheMetadata {
    /// Time-to-live in seconds for the cached response.
    pub ttl_secs: u64,
}

impl CacheMetadata {
    /// Creates new cache metadata with the given TTL.
    #[must_use]
    pub const fn new(ttl_secs: u64) -> Self {
        Self { ttl_secs }
    }
}

/// Metadata attached when a route parameter is annotated with `#[cache_key]`.
#[derive(Clone, Debug)]
pub struct CacheKeyMetadata {
    /// The name of the parameter that forms part of the cache key.
    pub param_name: String,
}

impl CacheKeyMetadata {
    /// Creates new cache-key metadata for a named parameter.
    #[must_use]
    pub fn new(param_name: impl Into<String>) -> Self {
        Self {
            param_name: param_name.into(),
        }
    }
}

/// Metadata attached when a route parameter is annotated with `#[cache_ttl]`.
#[derive(Clone, Debug)]
pub struct CacheTtlMetadata {
    /// The name of the parameter that provides the TTL override.
    pub param_name: String,
}

impl CacheTtlMetadata {
    /// Creates new cache-ttl metadata for a named parameter.
    #[must_use]
    pub fn new(param_name: impl Into<String>) -> Self {
        Self {
            param_name: param_name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_metadata_new_creates_correct_values() {
        let vm = VersionMetadata::new("1", VersioningStrategy::Uri);
        assert_eq!(vm.version, "1");
        assert_eq!(vm.strategy, VersioningStrategy::Uri);
    }

    #[test]
    fn version_metadata_uri_prefix_formats_correctly() {
        let vm = VersionMetadata::new("2", VersioningStrategy::Uri);
        assert_eq!(vm.uri_prefix(), "/v2");
    }

    #[test]
    fn version_metadata_multi_digit_version_prefix() {
        let vm = VersionMetadata::new("2024", VersioningStrategy::Uri);
        assert_eq!(vm.uri_prefix(), "/v2024");
    }

    #[test]
    fn version_metadata_equality() {
        let a = VersionMetadata::new("1", VersioningStrategy::Uri);
        let b = VersionMetadata::new("1", VersioningStrategy::Uri);
        assert_eq!(a, b);
    }

    #[test]
    fn version_metadata_different_version_not_equal() {
        let a = VersionMetadata::new("1", VersioningStrategy::Uri);
        let b = VersionMetadata::new("2", VersioningStrategy::Uri);
        assert_ne!(a, b);
    }

    #[test]
    fn version_metadata_different_strategy_not_equal() {
        let a = VersionMetadata::new("1", VersioningStrategy::Uri);
        let b = VersionMetadata::new("1", VersioningStrategy::Header);
        assert_ne!(a, b);
    }

    #[test]
    fn versioning_strategy_debug_and_clone() {
        let strategies = [
            VersioningStrategy::Uri,
            VersioningStrategy::Header,
            VersioningStrategy::MediaType,
        ];
        for strategy in &strategies {
            let cloned = strategy.clone();
            assert_eq!(format!("{strategy:?}"), format!("{cloned:?}"));
        }
    }

    #[test]
    fn cache_metadata_new_sets_ttl() {
        let cm = CacheMetadata::new(3600);
        assert_eq!(cm.ttl_secs, 3600);
    }

    #[test]
    fn cache_metadata_zero_ttl() {
        let cm = CacheMetadata::new(0);
        assert_eq!(cm.ttl_secs, 0);
    }

    #[test]
    fn cache_metadata_max_ttl() {
        let cm = CacheMetadata::new(u64::MAX);
        assert_eq!(cm.ttl_secs, u64::MAX);
    }
}
