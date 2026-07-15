use ironic::{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator};

pub struct CurrentUser;

impl CurrentUser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ParameterExtractor for CurrentUser {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let value: Box<dyn std::any::Any + Send> =
                Box::new(context.extension::<u64>().copied());
            Ok(value)
        })
    }
    fn description(&self) -> &'static str {
        "current_user"
    }
}

#[allow(non_camel_case_types)]
create_param_decorator!(current_user, CurrentUser);
