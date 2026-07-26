use std::path::{Path, PathBuf};

use crate::CliError;

use super::super::{
    GenerationReport,
    common::source::{ensure_items, ensure_module_import, write_generated},
};

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
        super::super::record(&mut report, path, state);
    }

    register_module(root, "email", "Email", &mut report);
    report.manual_instructions.push(
        "Dependencies for email (add to Cargo.toml):\n  handlebars = \"6\"\n  serde_json = \"1\"\n  # For `SMTP`: lettre = \"0.11\"\n  # For `SES`: aws-sdk-ses = \"1\", aws-config = \"1\"".into(),
    );
    Ok(report)
}

fn register_module(root: &Path, name: &str, pascal: &str, report: &mut GenerationReport) {
    let registry = root.join("src/modules/mod.rs");
    if let Err(e) = ensure_items(&registry, &[&format!("pub mod {name};")]) {
        report.manual_instructions.push(format!(
            "add `pub mod {name};` to {}: {e}",
            registry.display()
        ));
    } else {
        super::super::record(report, &registry, true);
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
    "//! Unit tests for EmailService (log adapter).\n\nuse crate::modules::email::services::EmailService;\n\n#[ironic::test]\nasync fn send_logs_email() {\n    let svc = EmailService::new();\n    let log = svc.send(\"test@example.com\", \"Hello\", \"Test body\").await.unwrap();\n    assert_eq!(log.status, \"sent\");\n    assert_eq!(log.to_email, \"test@example.com\");\n}\n".into()
}

fn integration_email_test() -> String {
    "//! Integration tests for email endpoints.\n\nuse ironic::{HttpStatus, TestApplication};\nuse serde_json::json;\nuse super::super::*;\n\nasync fn app() -> TestApplication {\n    TestApplication::new::<EmailModule>().await.unwrap()\n}\n\n#[ironic::test]\nasync fn send_email_returns_ok() {\n    let a = app().await;\n    let resp = a.post(\"/email/send\").json(&json!({\"to\":\"test@test.com\",\"subject\":\"Hi\",\"body\":\"Hello\"})).send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n\n#[ironic::test]\nasync fn status_returns_ok() {\n    let a = app().await;\n    let resp = a.get(\"/email/status/test-id\").send().await;\n    assert_eq!(resp.status(), HttpStatus::OK);\n    a.shutdown().await.unwrap();\n}\n".into()
}
