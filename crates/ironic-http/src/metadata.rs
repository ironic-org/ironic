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
