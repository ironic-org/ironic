## ADDED Requirements

### Requirement: Documentation for #[cache_key] parameter decorator
The framework SHALL add documentation for the `#[cache_key]` parameter decorator showing how to include route parameters in the cache key.

#### Scenario: Cache_key usage example in docs
- **WHEN** a user reads the cache-decorators documentation page
- **THEN** they see examples of `#[cache_key]` annotated parameters and how they affect the cache key

### Requirement: Documentation for #[cache_ttl] parameter decorator
The framework SHALL add documentation for the `#[cache_ttl]` parameter decorator showing how to dynamically override TTL per request.

#### Scenario: Cache_ttl usage example in docs
- **WHEN** a user reads the cache-decorators documentation page
- **THEN** they see examples of `#[cache_ttl]` with `Duration` parameters and how they override the route-level TTL

### Requirement: Documentation for CacheInterceptor URI-based cache key
The framework SHALL document that the `CacheInterceptor` includes the full URI (path + query string) in the cache key.

#### Scenario: Full URI cache key documented
- **WHEN** a user reads the cache documentation
- **THEN** they understand that the cache key includes path and query parameters
