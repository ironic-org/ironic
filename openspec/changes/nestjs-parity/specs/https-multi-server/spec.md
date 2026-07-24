## ADDED Requirements

### Requirement: HTTPS/TLS support
The framework SHALL support configuring HTTPS with TLS certificates.

#### Scenario: Configure HTTPS
- **WHEN** `Application::builder().tls(cert_path, key_path)` is configured
- **THEN** the HTTP server listens on HTTPS with the provided certificate

### Requirement: Multiple server support
The framework SHALL support running multiple HTTP servers (e.g., HTTP + HTTPS, or different ports).

#### Scenario: Run HTTP and HTTPS simultaneously
- **WHEN** two server configs are provided (one HTTP, one HTTPS)
- **THEN** both servers start and handle requests concurrently
