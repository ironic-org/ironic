---
title: Cloud Storage Uploads
description: Upload files to S3, DigitalOcean Spaces, MinIO, and other S3-compatible object storage from multipart endpoints.
---

# Cloud Storage Uploads

## What you'll learn

- Upload files to S3-compatible object storage (AWS S3, DigitalOcean Spaces, MinIO, Backblaze B2)
- Combine with `MultipartForm<T>` for typed file uploads
- Set public URLs, content types, and ACLs on uploaded objects
- Test locally with MinIO

---

## Enabling features

```toml
ironic = { features = ["multipart"] }
aws-sdk-s3 = { version = "1", features = ["rt-tokio"] }
```

Or for a lighter alternative:

```toml
rust-s3 = "0.36"
```

This guide uses `aws-sdk-s3`.

## Quick start: single file to S3

```rust
use aws_sdk_s3::{Client as S3Client, primitives::ByteStream};
use ironic::{MultipartForm, UploadedFile, HttpError};
use serde::Deserialize;

#[derive(Deserialize)]
struct ProfileUpload {
    user_id: String,
    avatar: UploadedFile,
}

#[post("/upload-avatar")]
async fn upload_avatar(
    s3: S3Client,
    form: MultipartForm<ProfileUpload>,
) -> Result<String, HttpError> {
    let file = &form.data.avatar;
    let user_id = &form.data.user_id;

    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "avatars".into());
    let key = format!("users/{user_id}/avatar.jpg");

    s3.put_object()
        .bucket(&bucket)
        .key(&key)
        .body(ByteStream::from(file.data.clone()))
        .content_type(file.content_type.as_deref().unwrap_or("application/octet-stream"))
        .acl("public-read")
        .send()
        .await
        .map_err(|e| HttpError::internal(format!("S3 upload failed: {e}")))?;

    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".into());
    let url = format!("https://{bucket}.s3.{region}.amazonaws.com/{key}");

    Ok(url)
}
```

## DigitalOcean Spaces (S3-compatible)

DigitalOcean Spaces uses the same S3 API with a custom endpoint:

```rust
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::{Client as S3Client, primitives::ByteStream};

fn spaces_client() -> S3Client {
    let creds = Credentials::new(
        std::env::var("DO_SPACES_KEY").unwrap(),
        std::env::var("DO_SPACES_SECRET").unwrap(),
        None, None, None,
    );

    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(std::env::var("DO_SPACES_REGION").unwrap_or_else(|_| "nyc3".into())))
        .endpoint_url(std::env::var("DO_SPACES_ENDPOINT").unwrap_or_else(|_| "https://nyc3.digitaloceanspaces.com".into()))
        .credentials_provider(creds)
        .build();

    S3Client::from_conf(config)
}
```

Usage:

```rust
#[post("/upload-to-spaces")]
async fn upload_to_spaces(form: MultipartForm<ProfileUpload>) -> Result<String, HttpError> {
    let client = spaces_client();

    let key = format!("uploads/{}", form.data.avatar.file_name.as_deref().unwrap_or("file"));
    let bucket = std::env::var("DO_SPACES_BUCKET").unwrap_or_else(|_| "my-bucket".into());

    client.put_object()
        .bucket(&bucket)
        .key(&key)
        .body(ByteStream::from(form.data.avatar.data.clone()))
        .content_type(form.data.avatar.content_type.as_deref().unwrap_or("application/octet-stream"))
        .acl("public-read")
        .send()
        .await
        .map_err(|e| HttpError::internal(format!("Spaces upload failed: {e}")))?;

    let url = format!("https://{bucket}.{region}.digitaloceanspaces.com/{key}");
    Ok(url)
}
```

## MinIO (local testing)

```rust
fn minio_client() -> S3Client {
    let creds = Credentials::new("minioadmin", "minioadmin", None, None, None);

    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url("http://localhost:9000")
        .credentials_provider(creds)
        .force_path_style(true)  // MinIO requires path-style
        .build();

    S3Client::from_conf(config)
}
```

Start MinIO locally:

```bash
docker run -p 9000:9000 -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"
```

## Full controller example

```rust
use aws_sdk_s3::primitives::ByteStream;
use ironic::{HttpError, MultipartForm, UploadedFile};
use serde::Deserialize;

#[derive(Deserialize)]
struct GalleryUpload {
    title: String,
    images: Vec<UploadedFile>,  // multiple files under same field
}

#[controller("/gallery")]
struct GalleryController {
    s3: S3Client,
    bucket: String,
}

#[post]
async fn upload(
    &self,
    form: MultipartForm<GalleryUpload>,
) -> Result<Vec<String>, HttpError> {
    let mut urls = Vec::new();

    for (i, file) in form.data.images.iter().enumerate() {
        let key = format!(
            "gallery/{}/{}_{}",
            form.data.title,
            i,
            file.file_name.as_deref().unwrap_or("file")
        );

        self.s3
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(file.data.clone()))
            .content_type(file.content_type.as_deref().unwrap_or("application/octet-stream"))
            .acl("public-read")
            .send()
            .await
            .map_err(|e| HttpError::internal(format!("S3 upload failed: {e}")))?;

        urls.push(format!("/{bucket}/{key}", bucket = self.bucket));
    }

    Ok(urls)
}
```

## Error handling

| Failure mode | Response |
|---|---|
| Invalid credentials | 500 — check `S3_ACCESS_KEY` / `S3_SECRET_KEY` env vars |
| Bucket not found | 500 — verify bucket name and region |
| File too large (multipart limit) | 413 — `PAYLOAD_TOO_LARGE` |
| Network timeout | 500 — retry with exponential backoff |
| Permission denied | 500 — verify IAM policy or Spaces CORS config |

```rust
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::put_object::PutObjectError;

fn map_s3_error(e: SdkError<PutObjectError>) -> HttpError {
    let msg = format!("S3 upload failed: {e}");
    tracing::error!(msg);
    HttpError::internal(msg)
}
```

## Configuration

| Env variable | Default | Description |
|---|---|---|
| `S3_ENDPOINT` | — | Custom endpoint (required for Spaces/MinIO) |
| `S3_REGION` | `us-east-1` | AWS region |
| `S3_BUCKET` | — | Bucket name |
| `S3_ACCESS_KEY` | — | Access key ID |
| `S3_SECRET_KEY` | — | Secret access key |
| `S3_FORCE_PATH_STYLE` | `false` | Use path-style URLs (required for MinIO) |

## Testing with MinIO

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn test_client() -> S3Client {
        let creds = Credentials::new("minioadmin", "minioadmin", None, None, None);
        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .endpoint_url("http://localhost:9000")
            .credentials_provider(creds)
            .force_path_style(true)
            .build();
        S3Client::from_conf(config)
    }

    #[tokio::test]
    async fn test_s3_upload() {
        // This test requires MinIO running on localhost:9000
        let client = test_client().await;
        client.create_bucket().bucket("test-bucket").send().await.unwrap();

        let boundary = "test-boundary";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"title\"\r\n\r\n\
             My Gallery\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"images\"; filename=\"photo.jpg\"\r\n\
             Content-Type: image/jpeg\r\n\r\n\
             fake-image-data\r\n\
             --{boundary}--\r\n"
        );

        let app = build_app(client);
        let response = app
            .oneshot(
                Request::post("/gallery")
                    .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

## Common mistakes

| Mistake | Fix |
|---|---|
| MinIO returns `AccessDenied` | Use `force_path_style(true)` in config |
| DigitalOcean Spaces returns 404 | Endpoint must match region: `https://nyc3.digitaloceanspaces.com` |
| Upload succeeds but file is empty | Check `ByteStream` — ensure `file.data` is not consumed before use |
| Timeout on large uploads | Increase `request_body_limit()` on the adapter or stream directly |
| `SdkError` with `dispatch_failure` | Verify network access to the S3 endpoint |

## What you learned

- [x] Upload files from `MultipartForm<T>` to S3, DigitalOcean Spaces, and MinIO
- [x] Configure custom endpoints, credentials, and regions for S3-compatible storage
- [x] Set content type and ACL on uploaded objects
- [x] Test uploads locally with MinIO in Docker
- [x] Handle common S3 error modes with proper HTTP responses
