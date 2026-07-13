## ADDED Requirements

### Requirement: Framework SHALL provide `@CacheKey` and `@CacheTTL` parameter decorators
The framework SHALL provide attribute macros that mark route parameters as cache key components and specify TTL for cached responses.

#### Scenario: Cached route returns cached response
- **WHEN** a route is annotated with `#[cache(ttl_secs = 60)]`
- **AND** a request arrives with the same cache key as a previous request
- **AND** the cache entry has not expired
- **THEN** the cached response SHALL be returned without invoking the handler

#### Scenario: Expired cache invokes handler
- **WHEN** a route is annotated with `#[cache(ttl_secs = 60)]`
- **AND** a request arrives after the cache entry has expired
- **THEN** the handler SHALL be invoked and the response SHALL be re-cached

### Requirement: Framework SHALL provide a `CacheInterceptor`
The framework SHALL provide a `CacheInterceptor` that integrates with the request pipeline to check and store cached responses.

#### Scenario: CacheInterceptor is registered as global interceptor
- **WHEN** `CacheInterceptor` is registered as a global interceptor
- **AND** a route has cache metadata
- **THEN** the interceptor SHALL check the cache before handler execution and write to cache after

### Requirement: Framework SHALL provide a Redis cache backend
The framework SHALL implement the existing `Cache` trait over Redis.

#### Scenario: Redis cache stores and retrieves values
- **WHEN** a `RedisCache` is configured
- **AND** a value is cached
- **THEN** the value SHALL be retrievable from Redis until it expires
