use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::modules::auth::services::Claims;

pub struct JwtGuard;

impl Guard for JwtGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let token = context
                .request()
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .unwrap_or_default();

            let secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "ironic-dev-secret".into());

            let key = DecodingKey::from_secret(secret.as_bytes());
            match decode::<Claims>(token, &key, &Validation::default()) {
                Ok(data) => {
                    context.insert_extension(data.claims);
                    Ok(GuardDecision::Allow)
                }
                Err(_) => Ok(GuardDecision::Deny),
            }
        })
    }
}
