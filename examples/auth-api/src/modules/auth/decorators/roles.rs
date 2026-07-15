use ironic::{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator};

pub struct Roles;

impl Roles {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ParameterExtractor for Roles {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let value: Box<dyn std::any::Any + Send> =
                Box::new(context.extension::<String>().cloned());
            Ok(value)
        })
    }
    fn description(&self) -> &'static str {
        "roles"
    }
}

#[allow(non_camel_case_types)]
create_param_decorator!(roles, Roles);
