---
title: Response serialization
description: Conditionally include or exclude fields based on user roles with derive-macro field-level rules.
---

# Response serialization

Enable `serialization` to control which fields appear in JSON responses based on the current
user's roles. Define field-level rules with derive macros and apply the transformation via a
pipeline interceptor.

```toml
ironic = { features = ["serialization"] }
```

## Deriving field rules

```rust
use ironic::{Serializable, SerializeInterceptor, FieldRules};

#[derive(Serializable)]
struct UserProfile {
    #[expose(role = "admin")]
    email: String,
    name: String,
    #[exclude]
    internal_id: String,
    #[expose(role = "owner")]
    phone: String,
}
```

- `#[exclude]` drops the field from every response.
- `#[expose(role = "...")]` includes the field only for the matching role.
- Fields without an attribute are always included.

## Building the field rules set

```rust
let rules = FieldRules::new()
    .exclude("email")
    .expose("phone", "owner");
```

## Registering the interceptor

Set the active roles on the request context and register the interceptor globally:

```rust
use ironic::{SerializeInterceptor, set_current_roles};

// In a guard or middleware, populate user roles:
set_current_roles(context, vec!["admin".to_string()]);

// Register the interceptor on the compiled application:
CompiledHttpApplication::new(container, routes)
    .interceptor(SerializeInterceptor::new(rules));
```

The interceptor inspects each response, walks the JSON structure, and drops excluded or
role-restricted fields before the response reaches the client.

## Conditional exposure

The interceptor reads roles from the request context extensions. When no roles are set, all
`#[expose(role = "...")]` fields are omitted. Set roles from an authentication guard or
middleware before the response interceptor executes.
