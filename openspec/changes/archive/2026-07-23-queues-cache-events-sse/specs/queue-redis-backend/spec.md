## ADDED Requirements

### Requirement: Framework SHALL provide a Redis queue backend implementing the `Queue` trait
The framework SHALL provide a `RedisQueue` struct that implements the existing `Queue` trait using Redis operations, configurable with connection manager and queue name.

#### Scenario: RedisQueue enqueues and dequeues messages
- **WHEN** a `RedisQueue` is configured with a Redis connection manager
- **AND** a `QueueMessage` is enqueued via `queue.enqueue(msg)`
- **AND** `queue.dequeue()` is called
- **THEN** the message SHALL be returned with matching `id`, `headers`, and `payload`

#### Scenario: RedisQueue acknowledge removes from processing set
- **WHEN** a message is dequeued
- **AND** `queue.acknowledge(msg_id)` is called
- **THEN** the message SHALL be removed from the in-flight processing set

#### Scenario: RedisQueue reject requeues message
- **WHEN** a message is dequeued
- **AND** `queue.reject(msg, requeue: true)` is called
- **THEN** the message SHALL be re-enqueued for redelivery

#### Scenario: RedisQueue reject with requeue=false moves to dead letter
- **WHEN** a message is dequeued
- **AND** `queue.reject(msg, requeue: false)` is called
- **THEN** the message SHALL be moved to the dead-letter list

### Requirement: Framework SHALL support message priority in `RedisQueue`
The framework SHALL allow enqueuing messages with a priority score that determines dequeue ordering when priority-sorted messages are available.

#### Scenario: Higher priority messages are dequeued first
- **WHEN** a low-priority message and a high-priority message are enqueued
- **AND** both are pending
- **THEN** the high-priority message SHALL be returned first by `dequeue()`

### Requirement: Framework SHALL support message TTL and retry count in `RedisQueue`
The framework SHALL track retry counts per message and discard messages that exceed their maximum retry limit, moving them to the dead-letter queue.

#### Scenario: Message exceeding max retries goes to dead letter
- **WHEN** a message has a `max_retries` of 3
- **AND** it is rejected with requeue 4 times
- **THEN** the message SHALL be moved to the dead-letter queue on the 4th rejection

#### Scenario: Expired message is not returned by dequeue
- **WHEN** a message is enqueued with a TTL of 1 second
- **AND** `dequeue()` is called after 2 seconds
- **THEN** the message SHALL NOT be returned

### Requirement: Framework SHALL provide `QueueConfig` for `RedisQueue`
The framework SHALL provide a configuration struct for RedisQueue with queue name, visibility timeout, max retries, and base key prefix.

#### Scenario: QueueConfig applies prefix to all Redis keys
- **WHEN** a `QueueConfig` is created with `prefix = "myapp"` and `name = "jobs"`
- **AND** a message is enqueued
- **THEN** the Redis key SHALL be `myapp:jobs:messages`

### Requirement: Framework SHALL gate `RedisQueue` behind `queues` + `redis` features
The `RedisQueue` SHALL only be compiled when both `features = ["queues", "redis"]` are enabled.

#### Scenario: RedisQueue not compiled without redis feature
- **WHEN** only `features = ["queues"]` is enabled
- **THEN** the `RedisQueue` type SHALL NOT be present in the compiled binary
