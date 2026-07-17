use crate::FeatureToggle;
use ironic_http::{Guard, GuardDecision, GuardFuture, RequestContext};

/// Guard that gates routes behind a runtime feature toggle.
///
/// ```ignore
/// let toggle = FeatureToggle::new("my-feature");
/// #[guard(FeatureGateGuard::new(toggle))]
/// ```
pub struct FeatureGateGuard {
    name: String,
}

impl FeatureGateGuard {
    /// Creates a guard for the named feature flag.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

impl Guard for FeatureGateGuard {
    fn can_activate<'a>(&'a self, _context: &'a mut RequestContext) -> GuardFuture<'a> {
        let name = self.name.clone();
        Box::pin(async move {
            let toggle = FeatureToggle::default();
            if toggle.is_enabled(&name) {
                Ok(GuardDecision::Allow)
            } else {
                Ok(GuardDecision::Deny)
            }
        })
    }
}
