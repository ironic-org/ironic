## ADDED Requirements

### Requirement: Test module SHALL support provider overrides
The test module builder SHALL allow overriding registered providers with mock/spy implementations without global state.

#### Scenario: Provider override in test module
- **WHEN** a test creates a `TestModule` and overrides a provider with a mock
- **THEN** resolving that provider SHALL return the mock implementation

### Requirement: Test application SHALL run without binding a network port
The test application SHALL support in-process HTTP request testing without opening a real network socket.

#### Scenario: In-process request test
- **WHEN** a `TestApplication` receives a request via its fluent client
- **THEN** the request SHALL be processed through the full pipeline without binding a port

### Requirement: Test client SHALL provide fluent request builder
The test client SHALL provide a builder API for constructing requests with method, path, JSON body, and headers.

#### Scenario: Fluent GET request
- **WHEN** `app.get("/users").send().await` is called
- **THEN** a GET request to `/users` SHALL be processed

### Requirement: Test assertions SHALL support status, header, JSON, and error response checks
The test client response SHALL provide methods to assert on status code, headers, JSON body, and error responses.

#### Scenario: Status code assertion
- **WHEN** asserting `response.assert_status(200)`
- **THEN** the assertion SHALL pass only if the status is 200

### Requirement: Test application lifecycle SHALL run cleanup
When a test application is dropped or explicitly shut down, lifecycle cleanup hooks SHALL run.

#### Scenario: Drop triggers cleanup
- **WHEN** a test application is dropped
- **THEN** shutdown hooks SHALL execute
