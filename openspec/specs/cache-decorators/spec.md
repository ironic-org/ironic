## ADDED Requirements

### Requirement: Framework SHALL provide `#[cache_key]` parameter decorator
The framework SHALL provide a `#[cache_key]` attribute macro that marks a route parameter as a component of the cache key.

#### Scenario: CacheKey parameter is included in cache key
- **WHEN** a route handler has parameters annotated with `#[cache_key]`
- **AND** a request arrives
- **THEN** the values of those parameters SHALL be concatenated into the cache key

### Requirement: Framework SHALL provide `#[cache_ttl]` parameter decorator
The framework SHALL provide a `#[cache_ttl]` attribute macro that dynamically overrides the TTL for a cached response on a per-request basis.

#### Scenario: CacheTTL parameter overrides route-level TTL
- **WHEN** a route handler has a `#[cache_ttl]` parameter of type `Duration`
- **AND** the request provides a value for that parameter
- **THEN** the TTL from the parameter SHALL override `#[cache(ttl_secs = N)]` for that request

### Requirement: Framework SHALL provide a `CacheInterceptor`
The framework SHALL provide a `CacheInterceptor` that integrates with the request pipeline to check and store cached responses.

#### Scenario: CacheInterceptor is registered as global interceptor
- **WHEN** `CacheInterceptor` is registered as a global interceptor
- **AND** a route has cache metadata
- **THEN** the interceptor SHALL check the cache before handler execution and write to cache after

### Requirement: Framework SHALL provide a completed Redis cache backend
The existing `RedisCache` stub SHALL be completed with functional `get`, `set`, `remove`, `clear`, and `remove_by_prefix` methods using Redis commands.

#### Scenario: RedisCache get returns stored value
- **WHEN** a value is cached via `cache.set("key", value, ttl).await`
- **AND** `cache.get("key").await` is called before TTL expiry
- **THEN** the value SHALL be returned

#### Scenario: RedisCache set with TTL expires correctly
- **WHEN** a value is cached with a 1-second TTL
- **AND** `cache.get("key").await` is called after 2 seconds
- **THEN** `None` SHALL be returned

#### Scenario: RedisCache remove deletes key
- **WHEN** a value is cached
- **AND** `cache.remove("key").await` is called
- **THEN** subsequent `cache.get("key").await` SHALL return `None`

#### Scenario: RedisCache remove_by_prefix uses SCAN
- **WHEN** multiple values are cached with keys matching a prefix
- **AND** `cache.remove_by_prefix(prefix).await` is called
- **THEN** all matching keys SHALL be deleted
- **AND** the operation SHALL use `SCAN` (not `KEYS`) to find matching keys
