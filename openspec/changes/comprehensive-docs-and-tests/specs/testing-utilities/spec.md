## ADDED Requirements

### Requirement: Every public function SHALL have at least one test
Every public function across all crates SHALL have at least one test case covering its primary use case. Tests SHALL be placed in a `#[cfg(test)] mod tests { ... }` block at the end of each source file.

#### Scenario: Public function has test coverage
- **WHEN** a crate is tested via `cargo test`
- **THEN** every public function SHALL be exercised by at least one test

### Requirement: Test assertions on doc-comment examples
Doc comments containing code examples SHALL be tested as doctests to ensure they remain accurate.

#### Scenario: Doc-comment example compiles and passes
- **WHEN** `cargo test --doc` is run
- **THEN** all doc-comment code examples SHALL compile and pass

## MODIFIED Requirements

### Requirement: Test module SHALL support provider overrides

The test module builder SHALL allow overriding registered providers with mock/spy implementations without global state. All public API items in the test module SHALL have full doc comments.

#### Scenario: Provider override in test module
- **WHEN** a test creates a `TestModule` and overrides a provider with a mock
- **THEN** resolving that provider SHALL return the mock implementation

### Requirement: Test application SHALL run without binding a network port

The test application SHALL support in-process HTTP request testing without opening a real network socket. All public API items in the test application SHALL have full doc comments.

#### Scenario: In-process request test
- **WHEN** a `TestApplication` receives a request via its fluent client
- **THEN** the request SHALL be processed through the full pipeline without binding a port

### Requirement: Test client SHALL provide fluent request builder

The test client SHALL provide a builder API for constructing requests with method, path, JSON body, and headers. All public API items in the test client SHALL have full doc comments.

#### Scenario: Fluent GET request
- **WHEN** `app.get("/users").send().await` is called
- **THEN** a GET request to `/users` SHALL be processed

### Requirement: Test assertions SHALL support status, header, JSON, and error response checks

The test client response SHALL provide methods to assert on status code, headers, JSON body, and error responses. All public API items in the test assertions SHALL have full doc comments.

#### Scenario: Status code assertion
- **WHEN** asserting `response.assert_status(200)`
- **THEN** the assertion SHALL pass only if the status is 200

### Requirement: Test application lifecycle SHALL run cleanup

When a test application is dropped or explicitly shut down, lifecycle cleanup hooks SHALL run. All public API items in the test application lifecycle SHALL have full doc comments.

#### Scenario: Drop triggers cleanup
- **WHEN** a test application is dropped
- **THEN** shutdown hooks SHALL execute
