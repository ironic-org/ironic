## ADDED Requirements

### Requirement: Global path prefix
The framework SHALL support a global path prefix applied to all routes in the application.

#### Scenario: Set global prefix
- **WHEN** `Application::builder().api_prefix("/api/v1")` is set
- **THEN** all routes are mounted under `/api/v1` (e.g., `/api/v1/users`, `/api/v1/products`)
