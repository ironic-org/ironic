use super::super::services::AuthService;
use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};

pub struct AuthGuard;

impl Guard for AuthGuard {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async move {
            let token = context
                .request()
                .headers()
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
