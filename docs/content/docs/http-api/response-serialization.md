---
title: Response Serialization
description: Control which fields appear in JSON responses — hide internal fields, expose admin-only data based on user roles.
---

# Response Serialization

## What you'll learn

- Mark fields to exclude from JSON responses
- Expose fields only to users with specific roles
- Apply serialization rules per endpoint

Enable in `Cargo.toml`:

```toml
ironic = { features = ["serialization"] }
```

---

## Step 1: Mark your view struct

```rust
use ironic::Serializable;

#[derive(Clone, Debug, serde::Serialize, Serializable)]
struct UserView {
    id: u64,
    name: String,
    email: String,

    #[exclude]                       // ← Never sent to clients
    password_hash: String,

    #[expose(role = "admin")]        // ← Only sent to admins
    internal_notes: String,
}
```

## Step 2: Define field rules

```rust
use ironic::{FieldRules, SerializeInterceptor};
use std::sync::Arc;

let rules = FieldRules::new()
    .exclude("password_hash")              // Hide from everyone
    .expose("internal_notes", "admin");    // Show only to admins
```

## Step 3: Apply the interceptor

```rust
#[controller("/users")]
#[interceptor(SerializeInterceptor::new(rules))]
#[derive(Injectable)]
struct UserController;
```

## How it works

```
DB record: { id: 1, name: "Alice", password_hash: "abc123", internal_notes: "VIP" }
                │
                ▼ SerializeInterceptor + FieldRules
                │
API response (normal user):  { id: 1, name: "Alice" }
API response (admin):        { id: 1, name: "Alice", internal_notes: "VIP" }
```

> **Key insight:** `password_hash` is never sent, regardless of role. `internal_notes` is sent only to admins.

## Setting roles per request

```rust
use ironic::set_current_roles;

// In middleware or guard:
set_current_roles(&["admin", "editor"]);
```

The current roles determine which fields are visible.

## What you learned

- [x] `#[exclude]` hides fields from all responses
- [x] `#[expose(role = "admin")]` shows fields only to specific roles
- [x] `SerializeInterceptor` applies rules at the controller level
- [x] `set_current_roles()` controls visibility per request
