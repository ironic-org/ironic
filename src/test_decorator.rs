use ironic::{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator};

struct Test;

impl ParameterExtractor for Test {
    fn extract<'a>(&'a self, _context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            Ok(Box::new(String::new()) as ironic::ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "test"
    }
}

create_param_decorator!(test, Test);
