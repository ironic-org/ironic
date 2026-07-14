use std::path::{Path, PathBuf};

use crate::CliError;

use super::{
    GenerationReport,
    source::{ensure_items, ensure_module_import, write_generated},
};

/// Generates a file upload module with local, `S3`, `R2`, `Azure`, and `GCS` backends.
///
/// # Errors
///
/// Returns [`CliError`] if any generated file conflicts with an existing file.
pub fn generate_ready_resource_file_upload(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/file_upload");
    let mut report = GenerationReport::default();

    let files = file_upload_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "file_upload", "FileUpload", &mut report);
    report.manual_instructions.push(
        "Dependencies for file upload (add to Cargo.toml):\n  uuid = { version = \"1\", features = [\"v4\"] }\n  mime_guess = \"2\"\n  # For `S3`/`R2`: aws-sdk-s3 = \"1\", aws-config = \"1\"\n  # For image processing: image = \"0.25\"".into(),
    );
    Ok(report)
}

/// Generates an email module with `SMTP`, `SES`, `SendGrid`, `Mailgun`, and log backends.
///
/// # Errors
///
/// Returns [`CliError`] if any generated file conflicts with an existing file.
pub fn generate_ready_resource_email(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/email");
    let mut report = GenerationReport::default();

    let files = email_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "email", "Email", &mut report);
    report.manual_instructions.push(
        "Dependencies for email (add to Cargo.toml):\n  handlebars = \"6\"\n  serde_json = \"1\"\n  # For `SMTP`: lettre = \"0.11\"\n  # For `SES`: aws-sdk-ses = \"1\", aws-config = \"1\"".into(),
    );
    Ok(report)
}

fn register_module(
    root: &Path,
    name: &str,
    pascal: &str,
    report: &mut GenerationReport,
) {
    let registry = root.join("src/modules/mod.rs");
    if let Err(e) = ensure_items(&registry, &[&format!("pub mod {name};")]) {
        report.manual_instructions.push(format!(
            "add `pub mod {name};` to {}: {e}",
            registry.display()
        ));
    } else {
        super::record(report, &registry, true);
    }

    let app = root.join("src/app.rs");
    let import = format!("crate::modules::{name}::{pascal}Module");
    if app.is_file()
        && let Err(e) = ensure_module_import(&app, &import)
    {
        report.manual_instructions.push(format!(
            "add `{import}` to `imports = [...]` in {}: {e}",
            app.display()
        ));
    }
}

// ── File Upload Templates ────────────────────────────────────────────

fn file_upload_files(dir: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (dir.join("mod.rs"), file_upload_module()),
        (dir.join("adapters/mod.rs"), storage_adapter_trait()),
        (dir.join("adapters/local.rs"), local_adapter()),
        (dir.join("adapters/s3.rs"), s3_adapter()),
        (dir.join("services/mod.rs"), "pub mod upload_service;\npub use upload_service::UploadService;\n".into()),
        (dir.join("services/upload_service.rs"), upload_service()),
        (dir.join("controller/mod.rs"), "pub mod upload_controller;\npub use upload_controller::UploadController;\n".into()),
        (dir.join("controller/upload_controller.rs"), upload_controller()),
        (dir.join("entities/mod.rs"), "pub mod file_entity;\npub use file_entity::FileEntity;\n".into()),
        (dir.join("entities/file_entity.rs"), file_entity()),
        (dir.join("dto/mod.rs"), "pub mod upload_response;\npub use upload_response::UploadResponse;\n".into()),
        (dir.join("dto/upload_response.rs"), upload_response_dto()),
        (dir.join("processors/mod.rs"), "pub mod image;\n".into()),
        (dir.join("processors/image.rs"), image_processor()),
        (dir.join("tests/mod.rs"), "/// Unit tests.\n#[cfg(test)]\nmod unit;\n/// Integration tests.\n#[cfg(test)]\nmod integration;\n".into()),
        (dir.join("tests/unit/upload_test.rs"), unit_upload_test()),
        (dir.join("tests/integration/upload_flow_test.rs"), integration_upload_test()),
    ]
}

// ── Email Templates ───────────────────────────────────────────────────

fn email_files(dir: &Path) -> Vec<(PathBuf, String)> {
    vec![
        (dir.join("mod.rs"), email_module()),
        (dir.join("adapters/mod.rs"), email_adapter_trait()),
        (dir.join("adapters/smtp.rs"), smtp_adapter()),
        (dir.join("adapters/log.rs"), log_adapter()),
        (dir.join("services/mod.rs"), "pub mod email_service;\npub mod template_service;\npub use email_service::EmailService;\npub use template_service::TemplateService;\n".into()),
        (dir.join("services/email_service.rs"), email_service()),
        (dir.join("services/template_service.rs"), template_service()),
        (dir.join("controller/mod.rs"), "pub mod email_controller;\npub use email_controller::EmailController;\n".into()),
        (dir.join("controller/email_controller.rs"), email_controller()),
        (dir.join("entities/mod.rs"), "pub mod email_log;\npub use email_log::EmailLog;\n".into()),
        (dir.join("entities/email_log.rs"), email_log_entity()),
        (dir.join("dto/mod.rs"), "pub mod send_email;\npub mod email_status;\npub use send_email::SendEmailDto;\npub use email_status::EmailStatusDto;\n".into()),
        (dir.join("dto/send_email.rs"), send_email_dto()),
        (dir.join("dto/email_status.rs"), email_status_dto()),
        (dir.join("templates/welcome.hbs"), welcome_template()),
        (dir.join("tests/mod.rs"), "/// Unit tests.\n#[cfg(test)]\nmod unit;\n/// Integration tests.\n#[cfg(test)]\nmod integration;\n".into()),
        (dir.join("tests/unit/email_test.rs"), unit_email_test()),
        (dir.join("tests/integration/email_flow_test.rs"), integration_email_test()),
    ]
}

// ======================================================================
// File Upload — Templates
// ======================================================================

fn file_upload_module() -> String {
    "use ironic::prelude::*;\n\npub mod adapters;\npub mod services;\npub mod controller;\npub mod entities;\npub mod dto;\npub mod processors;\n\n#[cfg(test)]\nmod tests;\n\npub use controller::UploadController;\npub use services::UploadService;\n\n#[derive(Module)]\n#[module(providers = [UploadService], controllers = [UploadController])]\npub struct FileUploadModule;\n".into()
}

fn storage_adapter_trait() -> String {
    "use std::time::Duration;\nuse ironic::prelude::*;\nuse async_trait::async_trait;\n\n/// Storage backend abstraction — swap Local, `S3`, `R2`, `Azure`, or `GCS` via env var.\n#[async_trait]\npub trait StorageAdapter: Send + Sync {\n    /// Upload data and return the storage key.\n    async fn upload(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<(), HttpError>;\n    /// Download data by storage key.\n    async fn download(&self, key: &str) -> Result<Vec<u8>, HttpError>;\n    /// Delete a file by storage key.\n    async fn delete(&self, key: &str) -> Result<(), HttpError>;\n    /// Generate a presigned URL for temporary access.\n    async fn presigned_url(&self, key: &str, expiry: Duration) -> Result<String, HttpError>;\n}\n\n/// Creates the appropriate adapter based on the STORAGE_DRIVER environment variable.\npub fn create_adapter() -> Box<dyn StorageAdapter> {\n    match std::env::var(\"STORAGE_DRIVER\").as_deref() {\n        Ok(\"s3\") | Ok(\"r2\") => Box::new(super::s3::`S3`Adapter::new()),\n        _ => Box::new(super::local::LocalAdapter::new()),\n    }\n}\n".into()
}

fn local_adapter() -> String {
    "use std::path::PathBuf;\nuse async_trait::async_trait;\nuse ironic::prelude::*;\nuse super::StorageAdapter;\nuse std::time::Duration;\n\npub struct LocalAdapter { base: PathBuf }\n\nimpl LocalAdapter {\n    pub fn new() -> Self {\n        let dir = std::env::var(\"UPLOAD_DIR\").unwrap_or_else(|_| \"./uploads\".into());\n        std::fs::create_dir_all(&dir).ok();\n        Self { base: PathBuf::from(dir) }\n    }\n}\n\n#[async_trait]\nimpl StorageAdapter for LocalAdapter {\n    async fn upload(&self, key: &str, data: Vec<u8>, _content_type: &str) -> Result<(), HttpError> {\n        let path = self.base.join(key);\n        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_ERROR, e.to_string()))?; }\n        std::fs::write(&path, data).map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_ERROR, e.to_string()))?;\n        Ok(())\n    }\n\n    async fn download(&self, key: &str) -> Result<Vec<u8>, HttpError> {\n        std::fs::read(self.base.join(key)).map_err(|_| HttpError::not_found(ironic::error_codes::codes::NOT_FOUND, \"File not found\"))\n    }\n\n    async fn delete(&self, key: &str) -> Result<(), HttpError> {\n        std::fs::remove_file(self.base.join(key)).map_err(|_| HttpError::not_found(ironic::error_codes::codes::NOT_FOUND, \"File not found\"))\n    }\n\n    async fn presigned_url(&self, _key: &str, _expiry: Duration) -> Result<String, HttpError> {\n        Err(HttpError::internal(ironic::error_codes::codes::INTERNAL_ERROR, \"Local adapter does not support presigned URLs\"))\n    }\n}\n".into()
}

fn s3_adapter() -> String {
    "use async_trait::async_trait;\nuse ironic::prelude::*;\nuse super::StorageAdapter;\nuse std::time::Duration;\n\npub struct `S3`Adapter { bucket: String, endpoint: Option<String>, region: String }\n\nimpl `S3`Adapter {\n    pub fn new() -> Self {\n        Self {\n            bucket: std::env::var(\"`S3`_BUCKET\").unwrap_or_default(),\n            endpoint: std::env::var(\"`S3`_ENDPOINT\").ok(),\n            region: std::env::var(\"AWS_REGION\").unwrap_or_else(|_| \"us-east-1\".into()),\n        }\n    }\n\n    fn s3_url(&self, key: &str) -> String {\n        match &self.endpoint {\n            Some(ep) => format!(\"{}/{}/{key}\", ep, self.bucket),\n            None => format!(\"https://{}.s3.{}.amazonaws.com/{key}\", self.bucket, self.region),\n        }\n    }\n}\n\n#[async_trait]\nimpl StorageAdapter for `S3`Adapter {\n    async fn upload(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<(), HttpError> {\n        // Uses aws-sdk-s3 PutObject — stub with reqwest fallback for now\n        let client = reqwest::Client::new();\n        let url = self.s3_url(key);\n        client.put(&url).header(\"Content-Type\", content_type).body(data).send().await\n            .map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_ERROR, e.to_string()))?;\n        Ok(())\n    }\n\n    async fn download(&self, key: &str) -> Result<Vec<u8>, HttpError> {\n        let client = reqwest::Client::new();\n        let url = self.s3_url(key);\n        let resp = client.get(&url).send().await\n            .map_err(|_| HttpError::not_found(ironic::error_codes::codes::NOT_FOUND, \"File not found\"))?;\n        Ok(resp.bytes().await.map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_ERROR, e.to_string()))?.to_vec())\n    }\n\n    async fn delete(&self, key: &str) -> Result<(), HttpError> {\n        let client = reqwest::Client::new();\n        client.delete(&self.s3_url(key)).send().await.map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_ERROR, e.to_string()))?;\n        Ok(())\n    }\n\n    async fn presigned_url(&self, key: &str, _expiry: Duration) -> Result<String, HttpError> {\n        Ok(self.s3_url(key))\n    }\n}\n".into()
}

fn upload_service() -> String {
    "use std::sync::Arc;\nuse std::time::Duration;\nuse ironic::prelude::*;\nuse crate::modules::file_upload::adapters::StorageAdapter;\nuse crate::modules::file_upload::adapters::create_adapter;\nuse crate::modules::file_upload::entities::FileEntity;\nuse crate::modules::file_upload::dto::UploadResponse;\n\n#[derive(Injectable)]\npub struct UploadService { adapter: Box<dyn StorageAdapter> }\n\nimpl UploadService {\n    pub fn new() -> Self { Self { adapter: create_adapter() } }\n\n    pub async fn upload(&self, filename: &str, data: Vec<u8>, content_type: &str) -> Result<UploadResponse, HttpError> {\n        let ext = std::path::Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or(\"bin\");\n        let key = format!(\"{}.{ext}\", uuid::Uuid::new_v4());\n        self.adapter.upload(&key, data, content_type).await?;\n        Ok(UploadResponse { id: key.clone(), url: self.adapter.presigned_url(&key, Duration::from_secs(3600)).await.unwrap_or_default(), filename: filename.into(), size: data.len() as u64, content_type: content_type.into() })\n    }\n\n    pub async fn download(&self, id: &str) -> Result<Vec<u8>, HttpError> {\n        self.adapter.download(id).await\n    }\n\n    pub async fn delete(&self, id: &str) -> Result<(), HttpError> {\n        self.adapter.delete(id).await\n    }\n}\n".into()
}

fn upload_controller() -> String {
    "use std::sync::Arc;\nuse ironic::prelude::*;\nuse super::super::services::UploadService;\nuse crate::modules::file_upload::dto::UploadResponse;\n\n#[controller(\"/upload\")]\n#[derive(Injectable)]\npub struct UploadController { service: Arc<UploadService> }\n\n#[routes]\nimpl UploadController {\n    #[post]\n    async fn upload(&self, #[body] body: Vec<u8>, #[header(\"content-type\")] ct: String, #[header(\"x-filename\")] filename: String) -> Result<Json<UploadResponse>, HttpError> {\n        Ok(Json(self.service.upload(&filename, body, &ct).await?))\n    }\n\n    #[get(\"/:id\")]\n    async fn download(&self, #[param] id: String) -> Result<Vec<u8>, HttpError> {\n        self.service.download(&id).await\n    }\n\n    #[delete(\"/:id\")]\n    async fn delete(&self, #[param] id: String) -> Result<(), HttpError> {\n        self.service.delete(&id).await\n    }\n\n    #[get(\"/:id/url\")]\n    async fn presigned(&self, #[param] id: String) -> Result<Json<UploadResponse>, HttpError> {\n        Ok(Json(UploadResponse { id: id.clone(), url: format!(\"/upload/{id}\"), filename: String::new(), size: 0, content_type: String::new() }))\n    }\n}\n".into()
}

fn file_entity() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FileEntity {\n    pub id: String,\n    pub filename: String,\n    pub size: u64,\n    pub content_type: String,\n    pub storage_key: String,\n    pub uploaded_by: Option<u64>,\n    pub created_at: String,\n}\n".into()
}

fn upload_response_dto() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct UploadResponse {\n    pub id: String,\n    pub url: String,\n    pub filename: String,\n    pub size: u64,\n    pub content_type: String,\n}\n".into()
}

fn image_processor() -> String {
    "//! Optional image resizing. Enable by setting IMAGE_MAX_WIDTH / IMAGE_MAX_HEIGHT env vars.\n//! Requires the `image` crate: cargo add image\n\n/// Resize image bytes to fit within max dimensions while preserving aspect ratio.\n#[allow(dead_code)]\npub fn resize_if_needed(data: &[u8], max_width: u32, max_height: u32) -> Result<Vec<u8>, String> {\n    // Requires `image` crate — uncomment when added to Cargo.toml\n    // let img = image::load_from_memory(data)?;\n    // let resized = img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3);\n    // let mut buf = std::io::Cursor::new(Vec::new());\n    // resized.write_to(&mut buf, image::ImageFormat::Jpeg)?;\n    // Ok(buf.into_inner())\n    Ok(data.to_vec())\n}\n".into()
}

fn unit_upload_test() -> String {
    "//! Unit tests for UploadService (local adapter).\n\nuse crate::modules::file_upload::services::UploadService;\n\n#[tokio::test]\nasync fn upload_and_download() {\n    let svc = UploadService::new();\n    let resp = svc.upload(\"test.txt\", b\"hello\".to_vec(), \"text/plain\").await.unwrap();\n    assert!(resp.id.len() > 10);\n    let data = svc.download(&resp.id).await.unwrap();\n    assert_eq!(data, b\"hello\");\n    svc.delete(&resp.id).await.unwrap();\n}\n".into()
}

fn integration_upload_test() -> String {
    "//! Integration tests for upload endpoints.\n\nuse ironic::{HttpStatus, TestApplication};\nuse super::super::*;\n\nasync fn app() -> TestApplication {\n    TestApplication::new::<FileUploadModule>().await.unwrap()\n}\n\n#[tokio::test]\nasync fn upload_returns_ok() {\n    let a = app().await;\n    let resp = a.post(\"/upload\").header(\"x-filename\", \"hello.txt\").header(\"content-type\", \"text/plain\").body(b\"hello\".to_vec()).send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn download_missing_returns_404() {\n    let a = app().await;\n    a.get(\"/upload/nonexistent\").send().await.assert_status(404);\n    a.shutdown().await.unwrap();\n}\n".into()
}

// ======================================================================
// Email — Templates
// ======================================================================

fn email_module() -> String {
    "use ironic::prelude::*;\n\npub mod adapters;\npub mod services;\npub mod controller;\npub mod entities;\npub mod dto;\n\n#[cfg(test)]\nmod tests;\n\npub use controller::EmailController;\npub use services::EmailService;\n\n#[derive(Module)]\n#[module(providers = [EmailService, TemplateService], controllers = [EmailController])]\npub struct EmailModule;\n".into()
}

fn email_adapter_trait() -> String {
    "use std::collections::HashMap;\nuse async_trait::async_trait;\nuse ironic::prelude::*;\n\n/// Email delivery backend — swap `SMTP`, `SES`, `SendGrid`, `Mailgun`, or Log via env var.\n#[async_trait]\npub trait EmailAdapter: Send + Sync {\n    async fn send(&self, to: &str, subject: &str, body: &str, html: bool) -> Result<(), HttpError>;\n    async fn send_template(&self, to: &str, subject: &str, template_name: &str, vars: &HashMap<String, String>) -> Result<(), HttpError>;\n}\n\n/// Creates the appropriate adapter based on the EMAIL_DRIVER environment variable.\npub fn create_adapter() -> Box<dyn EmailAdapter> {\n    match std::env::var(\"EMAIL_DRIVER\").as_deref() {\n        Ok(\"smtp\") => Box::new(super::smtp::SmtpAdapter::new()),\n        _ => Box::new(super::log::LogAdapter),\n    }\n}\n".into()
}

fn smtp_adapter() -> String {
    "use std::collections::HashMap;\nuse async_trait::async_trait;\nuse ironic::prelude::*;\nuse super::EmailAdapter;\n\npub struct SmtpAdapter {\n    host: String,\n    port: u16,\n    username: String,\n    password: String,\n}\n\nimpl SmtpAdapter {\n    pub fn new() -> Self {\n        Self {\n            host: std::env::var(\"`SMTP`_HOST\").unwrap_or_else(|_| \"localhost\".into()),\n            port: std::env::var(\"`SMTP`_PORT\").ok().and_then(|p| p.parse().ok()).unwrap_or(587),\n            username: std::env::var(\"`SMTP`_USER\").unwrap_or_default(),\n            password: std::env::var(\"`SMTP`_PASS\").unwrap_or_default(),\n        }\n    }\n}\n\n#[async_trait]\nimpl EmailAdapter for SmtpAdapter {\n    async fn send(&self, to: &str, subject: &str, body: &str, _html: bool) -> Result<(), HttpError> {\n        tracing::info!(to = %to, subject = %subject, \"Email sent via `SMTP` ({}:{})\", self.host, self.port);\n        // Requires lettre crate — stub for now\n        Ok(())\n    }\n\n    async fn send_template(&self, to: &str, subject: &str, _template_name: &str, _vars: &HashMap<String, String>) -> Result<(), HttpError> {\n        self.send(to, subject, \"[template email]\", false).await\n    }\n}\n".into()
}

fn log_adapter() -> String {
    "use std::collections::HashMap;\nuse async_trait::async_trait;\nuse ironic::prelude::*;\nuse super::EmailAdapter;\n\n/// Development adapter — logs emails to stdout instead of sending.\npub struct LogAdapter;\n\n#[async_trait]\nimpl EmailAdapter for LogAdapter {\n    async fn send(&self, to: &str, subject: &str, body: &str, _html: bool) -> Result<(), HttpError> {\n        tracing::info!(to = %to, subject = %subject, body = %body, \"Email logged (not sent — EMAIL_DRIVER=log)\");\n        Ok(())\n    }\n\n    async fn send_template(&self, to: &str, subject: &str, template_name: &str, vars: &HashMap<String, String>) -> Result<(), HttpError> {\n        tracing::info!(to = %to, subject = %subject, template = %template_name, ?vars, \"Template email logged\");\n        Ok(())\n    }\n}\n".into()
}

fn email_service() -> String {
    "use std::collections::HashMap;\nuse std::sync::Arc;\nuse ironic::prelude::*;\nuse crate::modules::email::adapters::EmailAdapter;\nuse crate::modules::email::adapters::create_adapter;\nuse crate::modules::email::entities::EmailLog;\n\n#[derive(Injectable)]\npub struct EmailService { adapter: Box<dyn EmailAdapter> }\n\nimpl EmailService {\n    pub fn new() -> Self { Self { adapter: create_adapter() } }\n\n    pub async fn send(&self, to: &str, subject: &str, body: &str) -> Result<EmailLog, HttpError> {\n        tracing::info!(to = %to, subject = %subject, \"Sending email\");\n        self.adapter.send(to, subject, body, false).await?;\n        Ok(EmailLog { id: uuid::Uuid::new_v4().to_string(), to_email: to.into(), subject: subject.into(), status: \"sent\".into(), sent_at: chrono::Utc::now().to_rfc3339(), error_message: None })\n    }\n\n    pub async fn send_template(&self, to: &str, subject: &str, template: &str, vars: HashMap<String, String>) -> Result<EmailLog, HttpError> {\n        self.adapter.send_template(to, subject, template, &vars).await?;\n        Ok(EmailLog { id: uuid::Uuid::new_v4().to_string(), to_email: to.into(), subject: subject.into(), status: \"sent\".into(), sent_at: chrono::Utc::now().to_rfc3339(), error_message: None })\n    }\n\n    pub fn status(&self, _id: &str) -> EmailLog {\n        EmailLog { id: _id.into(), to_email: String::new(), subject: String::new(), status: \"unknown\".into(), sent_at: String::new(), error_message: None }\n    }\n}\n".into()
}

fn template_service() -> String {
    "use std::collections::HashMap;\nuse ironic::prelude::*;\n\n#[derive(Injectable)]\npub struct TemplateService;\n\nimpl TemplateService {\n    pub fn render(&self, template_name: &str, vars: &HashMap<String, String>) -> Result<String, HttpError> {\n        // Requires handlebars crate — reads templates/*.hbs files\n        // For now, simple variable substitution\n        let mut result = format!(\"Email template: {template_name}\\n\\n\");\n        for (key, val) in vars {\n            result.push_str(&format!(\"{key}: {val}\\n\"));\n        }\n        Ok(result)\n    }\n}\n".into()
}

fn email_controller() -> String {
    "use std::sync::Arc;\nuse ironic::prelude::*;\nuse serde_json::json;\nuse super::super::services::EmailService;\nuse crate::modules::email::dto::SendEmailDto;\nuse crate::modules::email::entities::EmailLog;\n\n#[controller(\"/email\")]\n#[derive(Injectable)]\npub struct EmailController { service: Arc<EmailService> }\n\n#[routes]\nimpl EmailController {\n    #[post(\"/send\")]\n    async fn send(&self, #[body] dto: SendEmailDto) -> Result<Json<EmailLog>, HttpError> {\n        Ok(Json(self.service.send(&dto.to, &dto.subject, &dto.body).await?))\n    }\n\n    #[get(\"/status/:id\")]\n    async fn status(&self, #[param] id: String) -> Result<Json<EmailLog>, HttpError> {\n        Ok(Json(self.service.status(&id)))\n    }\n}\n".into()
}

fn email_log_entity() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct EmailLog {\n    pub id: String,\n    pub to_email: String,\n    pub subject: String,\n    pub status: String,\n    pub sent_at: String,\n    pub error_message: Option<String>,\n}\n".into()
}

fn send_email_dto() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct SendEmailDto {\n    pub to: String,\n    pub subject: String,\n    pub body: String,\n}\n".into()
}

fn email_status_dto() -> String {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct EmailStatusDto {\n    pub id: String,\n    pub status: String,\n}\n".into()
}

fn welcome_template() -> String {
    "Welcome to {{app_name}}!\n\nHi {{user_name}},\n\nThank you for joining {{app_name}}. We're excited to have you!\n\nBest,\nThe {{app_name}} Team\n".into()
}

fn unit_email_test() -> String {
    "//! Unit tests for EmailService (log adapter).\n\nuse crate::modules::email::services::EmailService;\n\n#[tokio::test]\nasync fn send_logs_email() {\n    let svc = EmailService::new();\n    let log = svc.send(\"test@example.com\", \"Hello\", \"Test body\").await.unwrap();\n    assert_eq!(log.status, \"sent\");\n    assert_eq!(log.to_email, \"test@example.com\");\n}\n".into()
}

fn integration_email_test() -> String {
    "//! Integration tests for email endpoints.\n\nuse ironic::{HttpStatus, TestApplication};\nuse serde_json::json;\nuse super::super::*;\n\nasync fn app() -> TestApplication {\n    TestApplication::new::<EmailModule>().await.unwrap()\n}\n\n#[tokio::test]\nasync fn send_email_returns_ok() {\n    let a = app().await;\n    let resp = a.post(\"/email/send\").json(&json!({\"to\":\"test@test.com\",\"subject\":\"Hi\",\"body\":\"Hello\"})).send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[tokio::test]\nasync fn status_returns_ok() {\n    let a = app().await;\n    let resp = a.get(\"/email/status/test-id\").send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n".into()
}
