## ADDED Requirements

### Requirement: Request pipeline SHALL follow documented ordering
The request pipeline SHALL process requests in this order: middleware chain, guards, interceptor before, parameter extraction, validation, controller handler, interceptor after, error mapping, response.

#### Scenario: Pipeline ordering is deterministic
- **WHEN** a request arrives
- **THEN** middleware SHALL execute before guards, guards before interceptors, interceptors before the handler

### Requirement: Guards SHALL gate handler invocation
Guards SHALL evaluate whether a request may invoke a handler. If a guard denies access, the handler SHALL NOT be invoked.

#### Scenario: Guard denies request
- **WHEN** a guard returns `false`
- **THEN** the request SHALL be rejected with a forbidden response and the handler SHALL NOT be invoked

### Requirement: Interceptors SHALL wrap handler execution
Interceptors SHALL execute before and after the controller handler, with the before-interceptor running after guards and the after-interceptor running after the handler returns.

#### Scenario: Interceptor wraps handler
- **WHEN** an interceptor is registered on a route
- **THEN** its before logic SHALL execute before the handler and its after logic SHALL execute after the handler

### Requirement: Parameter transformation and validation SHALL occur before handler invocation
Parameters extracted from the request SHALL be validated before the handler receives them.

#### Scenario: Invalid parameter causes validation error
- **WHEN** a request parameter fails validation
- **THEN** the handler SHALL NOT be invoked and a validation error SHALL be returned

### Requirement: Pipeline errors SHALL propagate through error handling
Errors at any pipeline stage SHALL be caught and mapped to an appropriate response.

#### Scenario: Middleware error propagates
- **WHEN** middleware returns an error
- **THEN** the error SHALL be mapped to a response and the pipeline SHALL stop
