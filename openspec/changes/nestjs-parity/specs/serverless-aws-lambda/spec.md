## ADDED Requirements

### Requirement: AWS Lambda serverless adapter
The framework SHALL provide an adapter that runs an Ironic application as an AWS Lambda function.

#### Scenario: Lambda handler creation
- **WHEN** `LambdaAdapter::new(app).handler()` is called
- **THEN** it returns a Lambda-compatible handler function that processes API Gateway events through the Ironic pipeline

#### Scenario: Request lifecycle preservation
- **WHEN** a Lambda request is processed
- **THEN** the full Ironic request pipeline runs (middleware, guards, interceptors, controllers, exception filters)
