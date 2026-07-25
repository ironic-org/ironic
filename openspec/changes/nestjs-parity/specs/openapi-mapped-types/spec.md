## ADDED Requirements

### Requirement: OpenAPI mapped type derives
The framework SHALL provide derive macros for common OpenAPI type transformations: `PartialType`, `PickType`, `OmitType`.

#### Scenario: Create partial type
- **WHEN** `#[derive(PartialType)] struct UpdateUserDto { #[partial(base = "CreateUserDto")] }`
- **THEN** `UpdateUserDto` has all fields of `CreateUserDto` as optional

#### Scenario: Create pick type
- **WHEN** `#[derive(PickType)] struct UserResponse { #[pick(base = "User", fields = [id, name, email])] }`
- **THEN** `UserResponse` only contains the specified fields from `User`

#### Scenario: Create omit type
- **WHEN** `#[derive(OmitType)] struct SafeUser { #[omit(base = "User", fields = [password_hash])] }`
- **THEN** `SafeUser` contains all fields of `User` except `password_hash`
