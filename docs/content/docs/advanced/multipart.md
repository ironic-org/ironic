---
title: Multipart Uploads
description: Accept file uploads, parse multipart forms, and control size limits.
---

# Multipart Uploads

## What you'll learn

- Accept file uploads with the `MultipartForm<T>` extractor
- Control per-file and total body size limits
- Handle upload errors with proper HTTP status codes
- Real-world examples: avatar upload, gallery, CSV import
- How to test multipart endpoints

---

## Enabling multipart

```toml
ironic = { features = ["multipart"] }
```

## Quick start: single file upload

```rust
use ironic::{MultipartForm, UploadedFile, HttpError};
use serde::Deserialize;

#[derive(Deserialize)]
struct AvatarUpload {
    name: String,
    avatar: UploadedFile,
}

#[post("/upload-avatar")]
async fn upload_avatar(form: MultipartForm<AvatarUpload>) -> Result<String, HttpError> {
    let file = &form.data.avatar;

    // Validate content type
    let is_image = file
        .content_type
        .as_deref()
        .is_some_and(|t| t.starts_with("image/"));
    if !is_image {
        return Err(HttpError::bad_request("INVALID_FILE_TYPE", "Only images are accepted"));
    }

    // Validate file size
    if file.size > 5 * 1024 * 1024 {
        return Err(HttpError::bad_request("FILE_TOO_LARGE", "Avatar must be under 5 MiB"));
    }

    // Save to disk
    let filename = format!("{}_{}", form.data.name, file.file_name.as_deref().unwrap_or("unknown"));
    tokio::fs::write(&format!("uploads/{filename}"), &file.data)
        .await
        .map_err(|e| HttpError::internal(format!("Failed to save file: {e}")))?;

    Ok(format!("Avatar saved as {filename}"))
}
```

## Multiple file upload (gallery)

```rust
use ironic::{MultipartFormData, UploadedFile};
use std::collections::HashMap;

#[post("/upload-gallery")]
async fn upload_gallery(form: MultipartFormData) -> Result<String, HttpError> {
    let images: Vec<&UploadedFile> = form
        .files
        .iter()
        .filter(|f| f.content_type.as_deref().is_some_and(|t| t.starts_with("image/")))
        .collect();

    if images.is_empty() {
        return Err(HttpError::bad_request("NO_IMAGES", "At least one image is required"));
    }
    if images.len() > 10 {
        return Err(HttpError::bad_request("TOO_MANY_FILES", "Maximum 10 images per upload"));
    }

    let total_size: u64 = images.iter().map(|f| f.size).sum();
    if total_size > 50 * 1024 * 1024 {
        return Err(HttpError::bad_request("TOTAL_TOO_LARGE", "Total image size must be under 50 MiB"));
    }

    Ok(format!("{} images accepted ({} bytes)", images.len(), total_size))
}
```

## CSV import with typed fields

```rust
#[derive(Deserialize)]
struct CsvImport {
    dataset: String,           // text field
    file: UploadedFile,        // the CSV file
    delimiter: Option<String>, // optional field, defaults to comma
}

#[post("/import-csv")]
async fn import_csv(form: MultipartForm<CsvImport>) -> Result<String, HttpError> {
    let file = &form.data.file;

    if file.content_type.as_deref() != Some("text/csv") {
        return Err(HttpError::bad_request("INVALID_FORMAT", "Only CSV files are accepted"));
    }

    let delimiter = form.data.delimiter.as_deref().unwrap_or(",");
    let content = std::str::from_utf8(&file.data)
        .map_err(|_| HttpError::bad_request("INVALID_ENCODING", "File must be valid UTF-8"))?;

    let record_count = content.lines().count() - 1; // skip header
    tracing::info!(
        dataset = %form.data.dataset,
        records = record_count,
        file = %file.file_name.as_deref().unwrap_or("unknown"),
        "CSV import started"
    );

    Ok(format!("Importing {record_count} records into {}", form.data.dataset))
}
```

## MultipartFormData (raw access)

When you don't know the field names at compile time, use `MultipartFormData`:

```rust
use ironic::MultipartFormData;

#[post("/upload-raw")]
async fn upload_raw(form: MultipartFormData) -> Result<String, HttpError> {
    // Group files by field name
    let mut groups: std::collections::BTreeMap<&str, Vec<&UploadedFile>> =
        std::collections::BTreeMap::new();
    for file in &form.files {
        groups.entry(file.field_name.as_str()).or_default().push(file);
    }

    for (field, files) in &groups {
        let total: u64 = files.iter().map(|f| f.size).sum();
        tracing::info!("field={field}, files={}, total_size={total}", files.len());
    }

    Ok(format!("Received {} files across {} fields", form.files.len(), groups.len()))
}
```

## UploadedFile reference

| Field | Type | Description |
|-------|------|-------------|
| `field_name` | `String` | Form field name from the multipart part |
| `file_name` | `Option<String>` | Original filename from `Content-Disposition` header |
| `content_type` | `Option<String>` | MIME type from `Content-Type` header |
| `size` | `u64` | File size in bytes |
| `data` | `Vec<u8>` | Raw file bytes (entire file in memory) |

> **Memory warning:** The entire file is buffered in memory.  For very large
> files (100 MiB+), consider streaming the multipart body directly instead of
> using this extractor.

## Configuration

```rust
use ironic::MultipartConfig;

// Per-endpoint configuration
let config = MultipartConfig {
    max_file_size: 10 * 1024 * 1024,  // 10 MiB per file
    max_field_size: 512 * 1024,       // 512 KiB per text field
};
```

| Field | Default | Description |
|-------|---------|-------------|
| `max_file_size` | `5 * 1024 * 1024` (5 MiB) | Maximum size per uploaded file |
| `max_field_size` | `256 * 1024` (256 KiB) | Maximum size per text field |

### Using configuration

```rust
// As a route parameter
#[post("/upload")]
async fn upload(
    MultipartForm::<AvatarUpload>::with_config(MultipartConfig {
        max_file_size: 1 * 1024 * 1024,  // 1 MiB for avatars
        ..MultipartConfig::default()
    }): MultipartForm<AvatarUpload>,
) -> Result<String, HttpError> {
    // ...
}
```

### Global body limit

`AxumAdapter::request_body_limit()` sets the total body limit for all requests,
including multipart.  Make sure it is at least as large as your total expected
upload size:

```rust
AxumAdapter::new()
    .request_body_limit(100 * 1024 * 1024)  // 100 MiB total
```

## Error handling

| Condition | HTTP Status | Error code |
|-----------|-------------|------------|
| File exceeds `max_file_size` | 413 Payload Too Large | `PAYLOAD_TOO_LARGE` |
| Field exceeds `max_field_size` | 413 Payload Too Large | `PAYLOAD_TOO_LARGE` |
| Malformed multipart boundary | 400 Bad Request | `BAD_REQUEST` |
| Missing required field | 400 Bad Request | `BAD_REQUEST` |
| Total body exceeds limit | 413 Payload Too Large | `PAYLOAD_TOO_LARGE` |

```rust
// Catching multipart errors
use ironic::HttpError;

fn handle_upload_error(e: HttpError) -> HttpError {
    if e.code() == "PAYLOAD_TOO_LARGE" {
        tracing::warn!("Upload rejected: file too large");
    }
    e
}
```

## Testing multipart endpoints

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_avatar_upload() {
        let app = build_app();

        // Build a multipart request manually
        let boundary = "test-boundary";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"name\"\r\n\r\n\
             alice\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"avatar\"; filename=\"photo.jpg\"\r\n\
             Content-Type: image/jpeg\r\n\r\n\
             fake-image-bytes\r\n\
             --{boundary}--\r\n"
        );

        let response = app
            .oneshot(
                Request::post("/upload-avatar")
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_multipart_config_defaults() {
        let config = MultipartConfig::default();
        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert_eq!(config.max_field_size, 256 * 1024);
    }
}
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Missing `multipart` feature | Add `ironic = { features = ["multipart"] }` to `Cargo.toml` |
| Body limit too low for uploads | Increase `request_body_limit()` on the adapter |
| Text field exceeds 256 KiB | Raise `max_field_size` in `MultipartConfig` |
| `MultipartFormData` used for known schema | Prefer `MultipartForm<T>` for compile-time field validation |
| File data consumes too much memory | For very large files, stream the multipart body directly |

## What you learned

- [x] `MultipartForm<T>` deserializes typed text fields with file uploads in one extractor
- [x] `MultipartFormData` gives raw access to all fields and files
- [x] `MultipartConfig` controls per-file and per-field size limits independently
- [x] Exceeding any limit returns 413 Payload Too Large
- [x] Memory: entire file buffered in `Vec<u8>` — plan for large uploads
- [x] Test multipart endpoints by sending manually constructed multipart bodies
