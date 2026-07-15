use ironic::{Guard, GuardDecision, GuardFuture, RequestContext};

#[allow(dead_code)]
pub struct RoleGuard {
    #[allow(dead_code)]
    required_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(roles: &[&str]) -> Self {
        Self {
            required_roles: roles.iter().map(|s| s.to_string()).collect(),
        }
    }
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
