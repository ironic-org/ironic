#![allow(clippy::needless_raw_string_hashes)]

use std::path::{Path, PathBuf};

use crate::CliError;

use super::{
    GenerationReport,
    source::{ensure_items, ensure_module_import, write_generated},
};

/// Generates a full authentication module with passwords, JWT, OAuth, sessions, and RBAC.
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource(root: &Path, name: &str) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules").join(name);
    let mut report = GenerationReport::default();

    let files = auth_full_files(&module_dir, name);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, name, &mut report);
    Ok(report)
}

/// Generates a basic auth module (passwords + sessions only).
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource_basic(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/auth");
    let mut report = GenerationReport::default();

    let files = auth_basic_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "auth", &mut report);
    Ok(report)
}

/// Generates a JWT-only auth module.
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource_jwt(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/auth");
    let mut report = GenerationReport::default();

    let files = auth_jwt_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "auth", &mut report);
    Ok(report)
}

/// Generates an OAuth-only auth module.
///
/// # Errors
///
/// Returns an error if file I/O fails during module generation or registration.
pub fn generate_ready_resource_oauth(root: &Path) -> Result<GenerationReport, CliError> {
    let module_dir = root.join("src/modules/auth");
    let mut report = GenerationReport::default();

    let files = auth_oauth_files(&module_dir);
    for (path, contents) in &files {
        let state = write_generated(path, contents)?;
        super::record(&mut report, path, state);
    }

    register_module(root, "auth", &mut report);
    Ok(report)
}

fn register_module(root: &Path, name: &str, report: &mut GenerationReport) {
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
    let pascal = "Auth";
    let import = format!("crate::modules::{name}::{pascal}Module");
    if app.is_file() {
        if let Err(e) = ensure_module_import(&app, &import) {
            report.manual_instructions.push(format!(
                "add `{import}` to `imports = [...]` in {}: {e}",
                app.display()
            ));
        } else {
            super::record(report, &app, true);
        }
    } else {
        report.manual_instructions.push(format!(
            "add `{import}` to your root module's `imports = [...]`"
        ));
    }

    // Auto-add required dependencies to Cargo.toml
    let manifest = root.join("Cargo.toml");
    if manifest.is_file() {
        let mut content = std::fs::read_to_string(&manifest).unwrap_or_default();
        let deps = [
            ("jsonwebtoken", "jsonwebtoken = \"9\""),
            ("argon2", "argon2 = \"0.5\""),
            ("oauth2", "oauth2 = \"5.0\""),
            ("getrandom", "getrandom = \"0.4\""),
        ];
        let mut added = false;
        for (name, dep) in &deps {
            if !content.contains(name) {
                content = content.replace("[dependencies]\n", &format!("[dependencies]\n{dep}\n"));
                added = true;
            }
        }
        if added {
            std::fs::write(&manifest, content).ok();
            report.manual_instructions.push(
                "Dependencies auto-added to Cargo.toml. Run `cargo build` to fetch them.".into(),
            );
        }
    }
}

// ── Auth helper source ───────────────────────────────────────────────

fn auth_module_preamble() -> &'static str {
    "use ironic::prelude::*;\n\npub mod controller;\npub mod services;\npub mod dto;\npub mod entities;\npub mod guards;\npub mod decorators;\n\n#[cfg(test)]\nmod tests;\n"
}

fn auth_full_files(module_dir: &Path, name: &str) -> Vec<(PathBuf, String)> {
    let d = module_dir;
    vec![
        (d.join("mod.rs"), format!("{}\n{}", auth_module_preamble(), auth_module_body(name))),
        (d.join("entities/mod.rs"), entity_mod().into()),
        (d.join("entities/user.rs"), user_entity().into()),
        (d.join("entities/role.rs"), role_enum().into()),
        (d.join("services/mod.rs"), services_mod().into()),
        (d.join("services/password_service.rs"), password_service().into()),
        (d.join("services/auth_service.rs"), auth_service_full().into()),
        (d.join("dto/mod.rs"), dto_mod_full().into()),
        (d.join("dto/register_dto.rs"), register_dto().into()),
        (d.join("dto/login_dto.rs"), login_dto().into()),
        (d.join("dto/refresh_dto.rs"), refresh_dto().into()),
        (d.join("dto/token_response.rs"), token_response().into()),
        (d.join("controller/mod.rs"), "pub mod auth_controller;\npub use auth_controller::AuthController;\n".into()),
        (d.join("controller/auth_controller.rs"), auth_controller_full().into()),
        (d.join("guards/mod.rs"), "pub mod auth_guard;\npub mod role_guard;\npub use auth_guard::AuthGuard;\npub use role_guard::RoleGuard;\n".into()),
        (d.join("guards/auth_guard.rs"), auth_guard().into()),
        (d.join("guards/role_guard.rs"), role_guard().into()),
        (d.join("decorators/mod.rs"), "pub mod current_user;\npub mod roles;\n".into()),
        (d.join("decorators/current_user.rs"), current_user_decorator().into()),
        (d.join("decorators/roles.rs"), roles_decorator().into()),
        (d.join("tests/mod.rs"), tests_mod().into()),
        (d.join("tests/unit/password_service_test.rs"), unit_password_test().into()),
        (d.join("tests/unit/auth_service_test.rs"), unit_auth_test().into()),
        (d.join("tests/unit/guard_test.rs"), unit_guard_test().into()),
        (d.join("tests/integration.rs"), integration_auth_test().into()),
    ]
}

fn auth_basic_files(module_dir: &Path) -> Vec<(PathBuf, String)> {
    let d = module_dir;
    vec![
        (
            d.join("mod.rs"),
            format!("{}\n{}", auth_module_preamble(), auth_module_body("auth")),
        ),
        (d.join("entities/mod.rs"), entity_mod().into()),
        (d.join("entities/user.rs"), user_entity().into()),
        (d.join("entities/role.rs"), role_enum().into()),
        (d.join("services/mod.rs"), services_mod().into()),
        (
            d.join("services/password_service.rs"),
            password_service().into(),
        ),
        (
            d.join("services/auth_service.rs"),
            auth_service_basic().into(),
        ),
        (d.join("dto/mod.rs"), dto_mod_basic().into()),
        (d.join("dto/register_dto.rs"), register_dto().into()),
        (d.join("dto/login_dto.rs"), login_dto().into()),
        (
            d.join("controller/mod.rs"),
            "pub mod auth_controller;\npub use auth_controller::AuthController;\n".into(),
        ),
        (
            d.join("controller/auth_controller.rs"),
            auth_controller_basic().into(),
        ),
        (
            d.join("guards/mod.rs"),
            "pub mod auth_guard;\npub use auth_guard::AuthGuard;\n".into(),
        ),
        (d.join("guards/auth_guard.rs"), auth_guard_basic().into()),
        (d.join("tests/mod.rs"), tests_mod().into()),
        (
            d.join("tests/unit/password_service_test.rs"),
            unit_password_test().into(),
        ),
        (
            d.join("tests/integration.rs"),
            integration_auth_basic_test().into(),
        ),
    ]
}

fn auth_jwt_files(module_dir: &Path) -> Vec<(PathBuf, String)> {
    let d = module_dir;
    vec![
        (
            d.join("mod.rs"),
            format!("{}\n{}", auth_module_preamble(), auth_module_body("auth")),
        ),
        (d.join("entities/mod.rs"), entity_mod().into()),
        (d.join("entities/user.rs"), user_entity().into()),
        (d.join("entities/role.rs"), role_enum().into()),
        (d.join("services/mod.rs"), services_mod().into()),
        (
            d.join("services/password_service.rs"),
            password_service().into(),
        ),
        (
            d.join("services/auth_service.rs"),
            auth_service_jwt().into(),
        ),
        (d.join("dto/mod.rs"), dto_mod_jwt().into()),
        (d.join("dto/register_dto.rs"), register_dto().into()),
        (d.join("dto/login_dto.rs"), login_dto().into()),
        (d.join("dto/refresh_dto.rs"), refresh_dto().into()),
        (d.join("dto/token_response.rs"), token_response().into()),
        (
            d.join("controller/mod.rs"),
            "pub mod auth_controller;\npub use auth_controller::AuthController;\n".into(),
        ),
        (
            d.join("controller/auth_controller.rs"),
            auth_controller_jwt().into(),
        ),
        (
            d.join("guards/mod.rs"),
            "pub mod auth_guard;\npub use auth_guard::AuthGuard;\n".into(),
        ),
        (d.join("guards/auth_guard.rs"), auth_guard().into()),
        (
            d.join("decorators/mod.rs"),
            "pub mod current_user;\n".into(),
        ),
        (
            d.join("decorators/current_user.rs"),
            current_user_decorator().into(),
        ),
        (d.join("tests/mod.rs"), tests_mod().into()),
        (
            d.join("tests/unit/auth_service_test.rs"),
            unit_auth_test().into(),
        ),
        (
            d.join("tests/integration.rs"),
            integration_auth_jwt_test().into(),
        ),
    ]
}

fn auth_oauth_files(module_dir: &Path) -> Vec<(PathBuf, String)> {
    let d = module_dir;
    vec![
        (
            d.join("mod.rs"),
            format!("{}\n{}", auth_module_preamble(), auth_module_body("auth")),
        ),
        (d.join("entities/mod.rs"), entity_mod().into()),
        (d.join("entities/user.rs"), user_entity().into()),
        (d.join("services/mod.rs"), services_mod().into()),
        (
            d.join("services/auth_service.rs"),
            auth_service_oauth().into(),
        ),
        (
            d.join("dto/mod.rs"),
            "pub mod token_response;\npub use token_response::TokenResponse;\n".into(),
        ),
        (d.join("dto/token_response.rs"), token_response().into()),
        (
            d.join("controller/mod.rs"),
            "pub mod auth_controller;\npub use auth_controller::AuthController;\n".into(),
        ),
        (
            d.join("controller/auth_controller.rs"),
            auth_controller_oauth().into(),
        ),
        (
            d.join("guards/mod.rs"),
            "pub mod auth_guard;\npub use auth_guard::AuthGuard;\n".into(),
        ),
        (d.join("guards/auth_guard.rs"), auth_guard_oauth().into()),
        (
            d.join("decorators/mod.rs"),
            "pub mod current_user;\n".into(),
        ),
        (
            d.join("decorators/current_user.rs"),
            current_user_decorator().into(),
        ),
        (d.join("tests/mod.rs"), tests_mod().into()),
        (
            d.join("tests/integration.rs"),
            integration_auth_oauth_test().into(),
        ),
    ]
}

// ── Module ────────────────────────────────────────────────────────────

fn auth_module_body(_name: &str) -> String {
    "pub use controller::AuthController;\npub use services::auth_service::AuthService;\npub use services::password_service::PasswordService;\npub use guards::AuthGuard;\n\n#[derive(Module)]\n#[module(\n    providers = [AuthService, PasswordService],\n    controllers = [AuthController],\n    exports = [AuthService],\n)]\npub struct AuthModule;\n".to_string()
}

// ── Entity ────────────────────────────────────────────────────────────

fn entity_mod() -> &'static str {
    "pub mod user;\npub mod role;\npub use user::User;\npub use role::Role;\n"
}

fn user_entity() -> &'static str {
    "use serde::{Deserialize, Serialize};\nuse super::role::Role;\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct User {\n    pub id: u64,\n    pub email: String,\n    #[serde(skip_serializing)]\n    pub password_hash: String,\n    pub name: String,\n    pub role: Role,\n    pub provider: String,\n    pub created_at: String,\n}\n\nimpl User {\n    /// Returns a safe view without the password hash.\n    #[must_use]\n    pub fn public_view(&self) -> PublicUser {\n        PublicUser {\n            id: self.id,\n            email: self.email.clone(),\n            name: self.name.clone(),\n            role: self.role,\n            provider: self.provider.clone(),\n        }\n    }\n}\n\n#[derive(Debug, Clone, Serialize)]\npub struct PublicUser {\n    pub id: u64,\n    pub email: String,\n    pub name: String,\n    pub role: Role,\n    pub provider: String,\n}\n"
}

fn role_enum() -> &'static str {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum Role {\n    Admin,\n    User,\n    Moderator,\n}\n\nimpl Role {\n    #[must_use]\n    pub fn as_str(&self) -> &'static str {\n        match self {\n            Role::Admin => \"admin\",\n            Role::User => \"user\",\n            Role::Moderator => \"moderator\",\n        }\n    }\n}\n"
}

// ── Services ──────────────────────────────────────────────────────────

fn services_mod() -> &'static str {
    "pub mod password_service;\npub mod auth_service;\npub use password_service::PasswordService;\npub use auth_service::AuthService;\n"
}

fn password_service() -> &'static str {
    "use argon2::{\n    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,\n    password_hash::{SaltString, rand_core::OsRng},\n};\nuse ironic::prelude::*;\n\n#[derive(Injectable)]\npub struct PasswordService;\n\nimpl PasswordService {\n    pub fn hash(&self, password: &str) -> Result<String, HttpError> {\n        let salt = SaltString::generate(&mut OsRng);\n        let hash = Argon2::default()\n            .hash_password(password.as_bytes(), &salt)\n            .map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_HASH_ERROR, e.to_string()))?;\n        Ok(hash.to_string())\n    }\n\n    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, HttpError> {\n        let parsed = PasswordHash::new(hash)\n            .map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_HASH_ERROR, e.to_string()))?;\n        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())\n    }\n}\n"
}

fn auth_service_full() -> &'static str {
    r#"use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use ironic::prelude::*;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use super::password_service::PasswordService;
use crate::modules::auth::dto::{LoginDto, RefreshDto, RegisterDto, TokenResponse};
use crate::modules::auth::entities::user::User;
use crate::modules::auth::entities::role::Role;

#[derive(Injectable)]
pub struct AuthService {
    password: Arc<PasswordService>,
}

static USERS: std::sync::LazyLock<Mutex<HashMap<u64, User>>> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims { pub sub: String, pub role: String, pub exp: usize, pub iat: usize }

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".into())
}

impl AuthService {
    pub fn register(&self, dto: RegisterDto) -> Result<User, HttpError> {
        let mut users = USERS.lock().unwrap();
        if users.values().any(|u| u.email == dto.email) {
            return Err(HttpError::bad_request(ironic::error_codes::codes::AUTH_EMAIL_EXISTS, "Email already registered"));
        }
        let hash = self.password.hash(&dto.password)?;
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let user = User { id, email: dto.email, password_hash: hash, name: dto.name, role: Role::User, provider: "email".into(), created_at: "now".into() };
        users.insert(id, user.clone());
        Ok(user)
    }

    pub fn login(&self, dto: LoginDto) -> Result<TokenResponse, HttpError> {
        let users = USERS.lock().unwrap();
        let user = users.values().find(|u| u.email == dto.email)
            .ok_or_else(|| HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_CREDENTIALS, "Invalid email or password"))?;
        if !self.password.verify(&dto.password, &user.password_hash)? {
            return Err(HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_CREDENTIALS, "Invalid email or password"));
        }
        self.issue_tokens(user)
    }

    pub fn refresh(&self, dto: RefreshDto) -> Result<TokenResponse, HttpError> {
        let token_data = decode::<Claims>(&dto.refresh_token, &DecodingKey::from_secret(jwt_secret().as_bytes()), &Validation::default())
            .map_err(|_| HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_TOKEN, "Invalid refresh token"))?;
        let users = USERS.lock().unwrap();
        let id: u64 = token_data.claims.sub.parse().unwrap_or(0);
        let user = users.get(&id).ok_or_else(|| HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_TOKEN, "User not found"))?;
        self.issue_tokens(user)
    }

    pub fn me(&self, user_id: u64) -> Result<User, HttpError> {
        USERS.lock().unwrap().get(&user_id).cloned()
            .ok_or_else(|| HttpError::not_found(ironic::error_codes::codes::NOT_FOUND_USER, "User not found"))
    }

    fn issue_tokens(&self, user: &User) -> Result<TokenResponse, HttpError> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize;
        let access = Claims { sub: user.id.to_string(), role: user.role.as_str().into(), exp: now + 900, iat: now };
        let refresh = Claims { sub: user.id.to_string(), role: user.role.as_str().into(), exp: now + 604800, iat: now };
        let at = encode(&Header::default(), &access, &EncodingKey::from_secret(jwt_secret().as_bytes()))
            .map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_JWT_ERROR, e.to_string()))?;
        let rt = encode(&Header::default(), &refresh, &EncodingKey::from_secret(jwt_secret().as_bytes()))
            .map_err(|e| HttpError::internal(ironic::error_codes::codes::INTERNAL_JWT_ERROR, e.to_string()))?;
        Ok(TokenResponse { access_token: at, refresh_token: rt, expires_in: 900 })
    }

    pub fn verify_token(token: &str) -> Result<Claims, HttpError> {
        decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret().as_bytes()), &Validation::default())
            .map(|d| d.claims)
            .map_err(|_| HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_TOKEN, "Invalid or expired token"))
    }
}
"#
}

fn auth_service_basic() -> &'static str {
    r#"use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use ironic::prelude::*;
use super::password_service::PasswordService;
use crate::modules::auth::dto::{LoginDto, RegisterDto};
use crate::modules::auth::entities::user::User;
use crate::modules::auth::entities::role::Role;

#[derive(Injectable)]
pub struct AuthService {
    password: Arc<PasswordService>,
}

static USERS: std::sync::LazyLock<Mutex<HashMap<u64, User>>> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl AuthService {
    pub fn register(&self, dto: RegisterDto) -> Result<User, HttpError> {
        let mut users = USERS.lock().unwrap();
        if users.values().any(|u| u.email == dto.email) {
            return Err(HttpError::bad_request(ironic::error_codes::codes::AUTH_EMAIL_EXISTS, "Email already registered"));
        }
        let hash = self.password.hash(&dto.password)?;
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let user = User { id, email: dto.email, password_hash: hash, name: dto.name, role: Role::User, provider: "email".into(), created_at: "now".into() };
        users.insert(id, user.clone());
        Ok(user)
    }

    pub fn login(&self, dto: LoginDto) -> Result<User, HttpError> {
        let users = USERS.lock().unwrap();
        let user = users.values().find(|u| u.email == dto.email)
            .ok_or_else(|| HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_CREDENTIALS, "Invalid email or password"))?;
        if !self.password.verify(&dto.password, &user.password_hash)? {
            return Err(HttpError::unauthorized(ironic::error_codes::codes::AUTH_INVALID_CREDENTIALS, "Invalid email or password"));
        }
        Ok(user.clone())
    }
}
"#
}

fn auth_service_jwt() -> &'static str {
    auth_service_full()
}
fn auth_service_oauth() -> &'static str {
    r#"use ironic::prelude::*;
use crate::modules::auth::dto::TokenResponse;

#[derive(Injectable)]
pub struct AuthService;

impl AuthService {
    pub fn oauth_url(&self, provider: &str) -> Result<String, HttpError> {
        let base = match provider {
            "google" => "https://accounts.google.com/o/oauth2/v2/auth",
            "github" => "https://github.com/login/oauth/authorize",
            _ => return Err(HttpError::bad_request("UNKNOWN_PROVIDER", format!("Unknown provider: {provider}"))),
        };
        let client_id = std::env::var("OAUTH_CLIENT_ID").unwrap_or_default();
        let redirect = std::env::var("OAUTH_REDIRECT_URL").unwrap_or_else(|_| "http://localhost:3000/auth/oauth/callback".into());
        Ok(format!("{base}?client_id={client_id}&redirect_uri={redirect}&response_type=code&scope=email+profile"))
    }

    pub fn exchange_code(&self, _code: &str) -> Result<TokenResponse, HttpError> {
        Ok(TokenResponse { access_token: "mock-oauth-token".into(), refresh_token: String::new(), expires_in: 3600 })
    }
}
"#
}

// ── DTOs ──────────────────────────────────────────────────────────────

fn dto_mod_full() -> &'static str {
    "pub mod register_dto;\npub mod login_dto;\npub mod refresh_dto;\npub mod token_response;\npub use register_dto::RegisterDto;\npub use login_dto::LoginDto;\npub use refresh_dto::RefreshDto;\npub use token_response::TokenResponse;\n"
}
fn dto_mod_basic() -> &'static str {
    "pub mod register_dto;\npub mod login_dto;\npub use register_dto::RegisterDto;\npub use login_dto::LoginDto;\n"
}
fn dto_mod_jwt() -> &'static str {
    dto_mod_full()
}

fn register_dto() -> &'static str {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RegisterDto {\n    pub email: String,\n    pub password: String,\n    pub name: String,\n}\n"
}
fn login_dto() -> &'static str {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LoginDto {\n    pub email: String,\n    pub password: String,\n}\n"
}
fn refresh_dto() -> &'static str {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RefreshDto {\n    pub refresh_token: String,\n}\n"
}
fn token_response() -> &'static str {
    "use serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TokenResponse {\n    pub access_token: String,\n    pub refresh_token: String,\n    pub expires_in: u64,\n}\n"
}

// ── Controller ────────────────────────────────────────────────────────

fn auth_controller_full() -> &'static str {
    r#"use std::sync::Arc;
use ironic::prelude::*;
use super::super::services::AuthService;
use crate::modules::auth::dto::{LoginDto, RefreshDto, RegisterDto, TokenResponse};
use crate::modules::auth::entities::user::PublicUser;

use super::super::guards::AuthGuard;
use super::super::decorators::current_user::current_user;

#[controller("/auth")]
#[derive(Injectable)]
pub struct AuthController { service: Arc<AuthService> }

#[routes]
impl AuthController {
    #[post("/register")]
    async fn register(&self, #[body] dto: RegisterDto) -> Result<Json<PublicUser>, HttpError> {
        Ok(Json(self.service.register(dto)?.public_view()))
    }

    #[post("/login")]
    async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
        Ok(Json(self.service.login(dto)?))
    }

    #[post("/refresh")]
    async fn refresh(&self, #[body] dto: RefreshDto) -> Result<Json<TokenResponse>, HttpError> {
        Ok(Json(self.service.refresh(dto)?))
    }

     #[get("/me")]
     #[use_guard(AuthGuard)]
     async fn me(&self, #[custom(current_user)] user_id: u64) -> Result<Json<PublicUser>, HttpError> {
         Ok(Json(self.service.me(user_id)?.public_view()))
     }
 }
"#
}

fn auth_controller_basic() -> &'static str {
    r#"use std::sync::Arc;
use ironic::prelude::*;
use super::super::services::AuthService;
use crate::modules::auth::dto::{LoginDto, RegisterDto};
use crate::modules::auth::entities::user::PublicUser;

#[controller("/auth")]
#[derive(Injectable)]
pub struct AuthController { service: Arc<AuthService> }

#[routes]
impl AuthController {
    #[post("/register")]
    async fn register(&self, #[body] dto: RegisterDto) -> Result<Json<PublicUser>, HttpError> {
        Ok(Json(self.service.register(dto)?.public_view()))
    }

    #[post("/login")]
    async fn login(&self, #[body] dto: LoginDto) -> Result<Json<PublicUser>, HttpError> {
        Ok(Json(self.service.login(dto)?.public_view()))
    }
}
"#
}

fn auth_controller_jwt() -> &'static str {
    auth_controller_full()
}

fn auth_controller_oauth() -> &'static str {
    r#"use std::sync::Arc;
use ironic::prelude::*;
use serde_json::json;
use super::super::services::AuthService;
use crate::modules::auth::dto::TokenResponse;
use crate::modules::auth::entities::user::PublicUser;

#[controller("/auth")]
#[derive(Injectable)]
pub struct AuthController { service: Arc<AuthService> }

#[routes]
impl AuthController {
    #[get("/oauth/:provider")]
    async fn login(&self, #[param] provider: String) -> Result<Json<serde_json::Value>, HttpError> {
        let url = self.service.oauth_url(&provider)?;
        Ok(Json(json!({"url": url})))
    }

    #[get("/oauth/callback")]
    async fn callback(&self, #[query] code: String) -> Result<Json<TokenResponse>, HttpError> {
        Ok(Json(self.service.exchange_code(&code)?))
    }

    #[get("/me")]
    #[use_guard(AuthGuard)]
    async fn me(&self, #[custom(CurrentUser)] user_id: u64) -> Result<Json<PublicUser>, HttpError> {
        Ok(Json(PublicUser { id: user_id, email: "oauth@user.com".into(), name: "OAuth User".into(), role: crate::modules::auth::entities::role::Role::User, provider: "oauth".into() }))
    }
}
"#
}

// ── Guards ────────────────────────────────────────────────────────────

fn auth_guard() -> &'static str {
    r#"use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};
use super::super::services::AuthService;

pub struct AuthGuard;

impl Guard for AuthGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let token = context.request().headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(String::from);

            match token {
                Some(t) => match AuthService::verify_token(&t) {
                    Ok(claims) => {
                        let uid: u64 = claims.sub.parse().unwrap_or(0);
                        context.insert_extension(uid);
                        context.insert_extension(claims.role);
                        Ok(GuardDecision::Allow)
                    }
                    Err(_) => Ok(GuardDecision::Deny),
                },
                None => Ok(GuardDecision::Deny),
            }
        })
    }
}
"#
}

fn auth_guard_basic() -> &'static str {
    r#"use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};

pub struct AuthGuard;

impl Guard for AuthGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let user_id = context.request().headers()
                .get("X-User-Id")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            match user_id {
                Some(id) => { context.insert_extension(id); Ok(GuardDecision::Allow) }
                None => Ok(GuardDecision::Deny("Missing X-User-Id header")),
            }
        })
    }
}
"#
}

fn auth_guard_oauth() -> &'static str {
    auth_guard()
}

fn role_guard() -> &'static str {
    r#"use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};

#[allow(dead_code)]
pub struct RoleGuard {
    #[allow(dead_code)]
    required_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(Roles: &[&str]) -> Self { Self { required_roles: Roles.iter().map(|s| s.to_string()).collect() } }
}

impl Guard for RoleGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let user_role = context.extension::<String>().cloned().unwrap_or_default();
            if self.required_roles.iter().any(|r| r == &user_role) {
                Ok(GuardDecision::Allow)
            } else {
                Ok(GuardDecision::Deny)
            }
        })
    }
}
"#
}

// ── Decorators ────────────────────────────────────────────────────────

fn current_user_decorator() -> &'static str {
    r#"use ironic::{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator};

pub struct CurrentUser;

impl CurrentUser {
    #[must_use]
    pub fn new() -> Self { Self }
}

impl ParameterExtractor for CurrentUser {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let value: Box<dyn std::any::Any + Send> = Box::new(context.extension::<u64>().copied());
            Ok(value)
        })
    }
    fn description(&self) -> &'static str { "current_user" }
}

#[allow(non_camel_case_types)]
create_param_decorator!(current_user, CurrentUser);
"#
}

fn roles_decorator() -> &'static str {
    r#"use ironic::{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator};

pub struct Roles;

impl Roles {
    #[must_use]
    pub fn new() -> Self { Self }
}

impl ParameterExtractor for Roles {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let value: Box<dyn std::any::Any + Send> = Box::new(context.extension::<String>().cloned());
            Ok(value)
        })
    }
    fn description(&self) -> &'static str { "roles" }
}

#[allow(non_camel_case_types)]
create_param_decorator!(roles, Roles);
"#
}

// ── Tests ─────────────────────────────────────────────────────────────

fn tests_mod() -> &'static str {
    "/// Unit tests — service and guard logic in isolation (no HTTP).\n#[cfg(test)]\nmod unit;\n/// Integration tests — full HTTP request/response through the framework.\n#[cfg(test)]\nmod integration;\n"
}

fn unit_password_test() -> &'static str {
    "//! Unit tests for PasswordService.\n\nuse crate::modules::auth::services::password_service::PasswordService;\n\n#[test]\nfn hash_and_verify() {\n    let svc = PasswordService;\n    let hash = svc.hash(\"password123\").unwrap();\n    assert!(svc.verify(\"password123\", &hash).unwrap());\n    assert!(!svc.verify(\"wrong\", &hash).unwrap());\n}\n\n#[test]\nfn unique_salts() {\n    let svc = PasswordService;\n    let h1 = svc.hash(\"password123\").unwrap();\n    let h2 = svc.hash(\"password123\").unwrap();\n    assert_ne!(h1, h2, \"same password should produce different hashes\");\n}\n"
}

fn unit_auth_test() -> &'static str {
    "//! Unit tests for AuthService.\n\nuse std::sync::Arc;\nuse crate::modules::auth::dto::{LoginDto, RegisterDto};\nuse crate::modules::auth::services::auth_service::AuthService;\nuse crate::modules::auth::services::password_service::PasswordService;\n\n#[test]\nfn register_and_login() {\n    let svc = AuthService { password: Arc::new(PasswordService) };\n    let user = svc.register(RegisterDto { email: \"test@test.com\".into(), password: \"pass123\".into(), name: \"Test\".into() }).unwrap();\n    assert_eq!(user.email, \"test@test.com\");\n    let tokens = svc.login(LoginDto { email: \"test@test.com\".into(), password: \"pass123\".into() }).unwrap();\n    assert!(!tokens.access_token.is_empty());\n}\n\n#[test]\nfn duplicate_email_rejected() {\n    let svc = AuthService { password: Arc::new(PasswordService) };\n    svc.register(RegisterDto { email: \"dup@test.com\".into(), password: \"pass\".into(), name: \"A\".into() }).unwrap();\n    assert!(svc.register(RegisterDto { email: \"dup@test.com\".into(), password: \"pass\".into(), name: \"B\".into() }).is_err());\n}\n\n#[test]\nfn wrong_password_rejected() {\n    let svc = AuthService { password: Arc::new(PasswordService) };\n    svc.register(RegisterDto { email: \"x@test.com\".into(), password: \"correct\".into(), name: \"X\".into() }).unwrap();\n    assert!(svc.login(LoginDto { email: \"x@test.com\".into(), password: \"wrong\".into() }).is_err());\n}\n"
}

fn unit_guard_test() -> &'static str {
    "//! Unit tests for AuthGuard and RoleGuard.\n\nuse std::sync::Arc;\nuse crate::modules::auth::guards::auth_guard::AuthGuard;\nuse crate::modules::auth::guards::role_guard::RoleGuard;\nuse ironic::{Guard, GuardDecision, FrameworkRequest, GuardFuture, RequestContext};\n\n#[tokio::test]\nasync fn auth_guard_denies_missing_header() {\n    let mut ctx = RequestContext::new(FrameworkRequest::new(ironic::HttpMethod::GET, \"/\".parse().unwrap(), ironic::HeaderMap::new(), vec![]));\n    let decision = AuthGuard.can_activate(&mut ctx).await.unwrap();\n    assert!(matches!(decision, GuardDecision::Deny(_)));\n}\n\n#[tokio::test]\nasync fn role_guard_denies_wrong_role() {\n    let mut ctx = RequestContext::new(FrameworkRequest::new(ironic::HttpMethod::GET, \"/\".parse().unwrap(), ironic::HeaderMap::new(), vec![]));\n    ctx.insert_extension(\"user\".to_string());\n    let guard = RoleGuard::new(&[\"admin\"]);\n    let decision = guard.can_activate(&mut ctx).await.unwrap();\n    assert!(matches!(decision, GuardDecision::Deny(_)));\n}\n"
}

fn integration_auth_test() -> &'static str {
    r#"//! Integration tests for full auth flow.

use ironic::{HttpStatus, TestApplication};
use serde_json::json;
use super::super::*;

async fn app() -> TestApplication { TestApplication::new::<AuthModule>().await.unwrap() }

#[tokio::test]
async fn register_and_login_flow() {
    let a = app().await;
    let resp = a.post("/auth/register").json(&json!({"email":"flow@test.com","password":"pass123","name":"Flow"})).send().await;
    assert_eq!(resp.status(), HttpStatus::OK);

    let resp = a.post("/auth/login").json(&json!({"email":"flow@test.com","password":"pass123"})).send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let tokens: serde_json::Value = resp.json().unwrap();
    assert!(tokens["access_token"].as_str().unwrap().len() > 10);
    a.shutdown().await.unwrap();
}

#[tokio::test]
async fn login_invalid_credentials() {
    let a = app().await;
    let resp = a.post("/auth/login").json(&json!({"email":"nobody@test.com","password":"wrong"})).send().await;
    assert_eq!(resp.status(), HttpStatus::UNAUTHORIZED);
    a.shutdown().await.unwrap();
}
"#
}

fn integration_auth_basic_test() -> &'static str {
    r#"use ironic::{HttpStatus, TestApplication};
use serde_json::json;
use super::super::*;

async fn app() -> TestApplication { TestApplication::new::<AuthModule>().await.unwrap() }

#[tokio::test]
async fn register_and_login() {
    let a = app().await;
    let resp = a.post("/auth/register").json(&json!({"email":"basic@test.com","password":"pass","name":"Basic"})).send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let resp = a.post("/auth/login").json(&json!({"email":"basic@test.com","password":"pass"})).send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    a.shutdown().await.unwrap();
}
"#
}

fn integration_auth_jwt_test() -> &'static str {
    integration_auth_test()
}

fn integration_auth_oauth_test() -> &'static str {
    r#"use ironic::{HttpStatus, TestApplication};
use super::super::*;

async fn app() -> TestApplication { TestApplication::new::<AuthModule>().await.unwrap() }

#[tokio::test]
async fn oauth_google_returns_url() {
    let a = app().await;
    let resp = a.get("/auth/oauth/google").send().await;
    assert_eq!(resp.status(), HttpStatus::OK);
    let body: serde_json::Value = resp.json().unwrap();
    assert!(body["url"].as_str().unwrap().contains("google"));
    a.shutdown().await.unwrap();
}
"#
}
