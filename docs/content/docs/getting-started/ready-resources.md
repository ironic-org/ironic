---
title: Ready Resources
description: Production-grade pre-built modules — authentication, file upload, and email delivery. Scaffold complete features in one command.
---

# Ready Resources

Ready Resources are production-grade, fully-tested modules that you can scaffold in one command. Unlike `ironic generate resource` (which creates a basic CRUD skeleton), ready resources include complete business logic, multiple backend adapters, guards, decorators, and test suites.

## Available Modules

| Command | Module | Includes |
|---------|--------|----------|
| `ironic generate ready-resource auth` | Authentication | Passwords, JWT, OAuth, RBAC, guards, decorators |
| `ironic generate ready-resource file-upload` | File Upload | Local, S3, R2 backends + image processing |
| `ironic generate ready-resource email` | Email | SMTP, SES, SendGrid, Mailgun + templates |

---

## Auth Module

### Quick Start

```bash
ironic generate ready-resource auth
```

This creates `src/modules/auth/` with **25 files** covering everything you need.

### Generated Structure

```
src/modules/auth/
├── mod.rs                          ← AuthModule wiring
├── controller/
│   └── auth_controller.rs          ← All auth endpoints
├── services/
│   ├── password_service.rs         ← Argon2id hashing
│   └── auth_service.rs             ← Register, login, JWT, OAuth
├── guards/
│   ├── auth_guard.rs               ← JWT extraction + verification
│   └── role_guard.rs               ← Role-based access control
├── decorators/
│   ├── current_user.rs             ← Extract user from request
│   └── roles.rs                    ← Extract roles from request
├── entities/
│   ├── user.rs                     ← User + PublicUser
│   └── role.rs                     ← Admin | User | Moderator
├── dto/
│   ├── register_dto.rs
│   ├── login_dto.rs
│   ├── refresh_dto.rs
│   └── token_response.rs
└── tests/
    ├── unit/                       ← 3 unit test files
    └── integration/                ← Full auth flow tests
```

### API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/auth/register` | No | Create account with email + password |
| `POST` | `/auth/login` | No | Get JWT access + refresh tokens |
| `POST` | `/auth/refresh` | Refresh token | Get new access token |
| `GET` | `/auth/me` | JWT required | Get current user profile |
| `GET` | `/auth/oauth/:provider` | No | Start OAuth flow (google/github) |

### How to Use

**1. Add dependencies to `Cargo.toml`:**

```toml
jsonwebtoken = "9"
argon2 = "0.5"
oauth2 = "5.0"
getrandom = "0.4"
```

**2. Set environment variables:**

```bash
# .env
JWT_SECRET=your-256-bit-secret-change-me
OAUTH_CLIENT_ID=your-client-id
OAUTH_REDIRECT_URL=http://localhost:3000/auth/oauth/callback
```

**3. Protect routes with guards:**

```rust
#[get("/admin")]
#[guard(RoleGuard::new(&["admin"]))]
async fn admin_only(&self, #[decorator(current_user)] user_id: u64) -> Result<Json<String>, HttpError> {
    Ok(Json(format!("Welcome admin #{user_id}!")))
}
```

**4. Use the current_user decorator:**

```rust
#[get("/profile")]
async fn profile(&self, #[decorator(current_user)] user_id: u64) -> Result<Json<User>, HttpError> {
    self.service.find_by_id(user_id)
}
```

### Auth Variants

| Command | What's Generated |
|---------|-----------------|
| `ironic generate ready-resource auth` | Full: passwords + JWT + OAuth + RBAC |
| `ironic generate ready-resource auth-basic` | Passwords + sessions only |
| `ironic generate ready-resource auth-jwt` | JWT tokens only |
| `ironic generate ready-resource auth-oauth` | OAuth2 social login only |

---

## File Upload Module

### Quick Start

```bash
ironic generate ready-resource file-upload
```

### Generated Structure

```
src/modules/file_upload/
├── mod.rs                          ← FileUploadModule wiring
├── adapters/
│   ├── mod.rs                      ← StorageAdapter trait + factory
│   ├── local.rs                    ← Local filesystem (./uploads/)
│   └── s3.rs                       ← AWS S3 + CloudFlare R2
├── controller/
│   └── upload_controller.rs        ← Upload/download/delete endpoints
├── services/
│   └── upload_service.rs           ← Adapter selection + orchestration
├── processors/
│   └── image.rs                    ← Image resize/crop hook
├── entities/
│   └── file_entity.rs              ← File metadata
├── dto/
│   └── upload_response.rs          ← Upload response
└── tests/
    ├── unit/upload_test.rs
    └── integration/upload_flow_test.rs
```

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/upload` | Upload file (raw body + `x-filename` + `content-type` headers) |
| `GET` | `/upload/:id` | Download file by UUID |
| `DELETE` | `/upload/:id` | Delete file |
| `GET` | `/upload/:id/url` | Get presigned URL |

### How to Use

**1. Add dependencies:**

```toml
uuid = { version = "1", features = ["v4"] }
mime_guess = "2"
# For S3/R2:
aws-sdk-s3 = "1"
aws-config = "1"
# For image processing:
image = "0.25"
```

**2. Choose storage backend:**

```bash
# Local filesystem (default)
STORAGE_DRIVER=local
UPLOAD_DIR=./uploads

# AWS S3
STORAGE_DRIVER=s3
AWS_REGION=us-east-1
S3_BUCKET=my-bucket
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=...

# CloudFlare R2 (S3-compatible)
STORAGE_DRIVER=r2
S3_BUCKET=my-bucket
S3_ENDPOINT=https://<account>.r2.cloudflarestorage.com
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
```

**3. Upload a file:**

```bash
curl -X POST http://localhost:3000/upload \
  -H "x-filename: photo.jpg" \
  -H "content-type: image/jpeg" \
  --data-binary @photo.jpg
```

Response:
```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890.jpg",
  "url": "https://s3.amazonaws.com/my-bucket/a1b2c3d4...",
  "filename": "photo.jpg",
  "size": 245760,
  "content_type": "image/jpeg"
}
```

### StorageAdapter Trait

```rust
pub trait StorageAdapter {
    async fn upload(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<(), HttpError>;
    async fn download(&self, key: &str) -> Result<Vec<u8>, HttpError>;
    async fn delete(&self, key: &str) -> Result<(), HttpError>;
    async fn presigned_url(&self, key: &str, expiry: Duration) -> Result<String, HttpError>;
}
```

### Adding a Custom Adapter

Implement the `StorageAdapter` trait and register in `create_adapter()`:

```rust
use crate::modules::file_upload::adapters::StorageAdapter;

pub struct MyCustomAdapter;

impl StorageAdapter for MyCustomAdapter {
    // ... implement 4 methods
}

// In adapters/mod.rs:
pub fn create_adapter() -> Box<dyn StorageAdapter> {
    match std::env::var("STORAGE_DRIVER").as_deref() {
        Ok("s3") | Ok("r2") => Box::new(super::s3::S3Adapter::new()),
        Ok("custom") => Box::new(super::custom::MyCustomAdapter::new()),
        _ => Box::new(super::local::LocalAdapter::new()),
    }
}
```

---

## Email Module

### Quick Start

```bash
ironic generate ready-resource email
```

### Generated Structure

```
src/modules/email/
├── mod.rs                          ← EmailModule wiring
├── adapters/
│   ├── mod.rs                      ← EmailAdapter trait + factory
│   ├── smtp.rs                     ← SMTP (any server)
│   └── log.rs                      ← Development: logs to stdout
├── controller/
│   └── email_controller.rs         ← Send + status endpoints
├── services/
│   ├── email_service.rs            ← Adapter selection + send
│   └── template_service.rs         ← Handlebars rendering
├── entities/
│   └── email_log.rs                ← Sent email audit log
├── dto/
│   ├── send_email.rs
│   └── email_status.rs
├── templates/
│   └── welcome.hbs                 ← Handlebars template
└── tests/
    ├── unit/email_test.rs
    └── integration/email_flow_test.rs
```

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/email/send` | Send email (to, subject, body) |
| `GET` | `/email/status/:id` | Check delivery status |

### How to Use

**1. Add dependencies:**

```toml
handlebars = "6"
serde_json = "1"
# For SMTP:
lettre = "0.11"
# For SES:
aws-sdk-ses = "1"
aws-config = "1"
```

**2. Choose email driver:**

```bash
# Development (default) — logs to stdout, no real sending
EMAIL_DRIVER=log

# SMTP (any server: Gmail, Mailtrap, SendGrid SMTP, custom)
EMAIL_DRIVER=smtp
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=you@gmail.com
SMTP_PASS=your-app-password

# AWS SES
EMAIL_DRIVER=ses
AWS_REGION=us-east-1
```

**3. Send an email:**

```bash
curl -X POST http://localhost:3000/email/send \
  -H "Content-Type: application/json" \
  -d '{"to":"user@example.com","subject":"Hello","body":"Welcome to our platform!"}'
```

Response:
```json
{
  "id": "a1b2c3d4-...",
  "to_email": "user@example.com",
  "subject": "Hello",
  "status": "sent",
  "sent_at": "2026-07-14T12:00:00Z",
  "error_message": null
}
```

### EmailAdapter Trait

```rust
pub trait EmailAdapter {
    async fn send(&self, to: &str, subject: &str, body: &str, html: bool) -> Result<(), HttpError>;
    async fn send_template(&self, to: &str, subject: &str, template_name: &str, vars: &HashMap<String, String>) -> Result<(), HttpError>;
}
```

### Handlebars Templates

Create `.hbs` files in `src/modules/email/templates/`:

```handlebars
<!-- welcome.hbs -->
<h1>Welcome to {{app_name}}!</h1>
<p>Hi {{user_name}},</p>
<p>Thank you for joining. We're excited to have you!</p>
```

Render with variables:

```rust
let mut vars = HashMap::new();
vars.insert("app_name".into(), "My App".into());
vars.insert("user_name".into(), "Alice".into());
let html = template_service.render("welcome", &vars)?;
```

---

## Error Codes Reference

All ready-resource modules use standardized error codes from `ironic::error_codes::codes`:

| Code | HTTP | Meaning |
|------|------|---------|
| `AUTH_INVALID_CREDENTIALS` | 401 | Wrong email or password |
| `AUTH_INVALID_TOKEN` | 401 | JWT expired, malformed, or missing |
| `AUTH_EMAIL_EXISTS` | 409 | Email already registered |
| `NOT_FOUND` | 404 | Resource not found |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

---

## Adding Dependencies Manually

After generating any ready resource, the CLI prints the required dependencies. Add them to your `Cargo.toml`:

```bash
ironic generate ready-resource auth 2>&1 | grep "manual:"
# manual: Add these dependencies to Cargo.toml:
#   jsonwebtoken = "9"
#   argon2 = "0.5"
#   ...
```

Then run `ironic dev` to start with hot reload.
