## ADDED Requirements

### Requirement: Framework SHALL support URI prefix versioning
Controllers SHALL be assignable to a version prefix (e.g., `/v1/users`, `/v2/users`).

#### Scenario: Versioned controller routes are prefixed
- **WHEN** a controller is annotated with `#[controller(version = "1")]`
- **THEN** all its routes SHALL be mounted under `/v1/<path>`

#### Scenario: Multiple versions coexist
- **WHEN** `V1UsersController` is annotated with `version = "1"`
- **AND** `V2UsersController` is annotated with `version = "2"`
- **THEN** both `/v1/users` and `/v2/users` SHALL be routable

### Requirement: Framework SHALL support header-based versioning
Controllers SHALL be versionable via the `Accept-Version` request header.

#### Scenario: Header version matches controller
- **WHEN** a controller is annotated with `#[controller(version = "1", strategy = "header")]`
- **AND** a request includes `Accept-Version: 1`
- **THEN** the request SHALL be routed to that controller

#### Scenario: Unmatched header version returns 404
- **WHEN** a controller is annotated with `version = "1", strategy = "header"`
- **AND** a request includes `Accept-Version: 3`
- **THEN** a 404 Not Found SHALL be returned

### Requirement: Framework SHALL support media type versioning
Controllers SHALL be versionable via the `Accept` header media type (e.g., `application/vnd.api.v1+json`).

#### Scenario: Media type version matches controller
- **WHEN** a controller is annotated with `#[controller(version = "1", strategy = "media-type")]`
- **AND** a request includes `Accept: application/vnd.api.v1+json`
- **THEN** the request SHALL be routed to that controller
