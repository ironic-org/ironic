use super::naming::Names;

pub(crate) fn module(pascal: &str) -> String {
    format!(
        "use ironic::prelude::*;\n\n#[derive(Module)]\n#[module()]\npub struct {pascal}Module;\n"
    )
}

pub(crate) fn service(names: &Names) -> String {
    format!(
        "use ironic::prelude::*;\n\n#[derive(Injectable)]\npub struct {0}Service;\n\nimpl {0}Service {{\n    #[must_use]\n    pub const fn name(&self) -> &'static str {{\n        \"{1}\"\n    }}\n}}\n",
        names.pascal, names.kebab
    )
}

pub(crate) fn controller(names: &Names) -> String {
    format!(
        "use ironic::prelude::*;\n\n#[controller(\"/{}\")]\n#[derive(Injectable)]\npub struct {}Controller;\n\n#[routes]\nimpl {}Controller {{\n    #[get(\"/\")]\n    #[allow(clippy::unused_async)]\n    async fn list(&self) -> Result<&'static str, HttpError> {{\n        Ok(\"{} controller\")\n    }}\n}}\n",
        names.kebab, names.pascal, names.pascal, names.kebab
    )
}

pub(crate) fn resource_module(names: &Names) -> String {
    format!(
        "use ironic::prelude::*;\n\npub mod {0}_controller;\npub mod {0}_service;\n\npub use {0}_controller::{1}Controller;\npub use {0}_service::{1}Service;\n\n#[derive(Module)]\n#[module(\n    providers = [{1}Service],\n    controllers = [{1}Controller],\n)]\npub struct {1}Module;\n",
        names.snake, names.pascal
    )
}

pub(crate) fn resource_controller(names: &Names) -> String {
    format!(
        "use std::sync::Arc;\n\nuse ironic::prelude::*;\n\nuse super::{0}_service::{1}Service;\n\n#[controller(\"/{2}\")]\n#[derive(Injectable)]\npub struct {1}Controller {{\n    service: Arc<{1}Service>,\n}}\n\n#[routes]\nimpl {1}Controller {{\n    #[get(\"/\")]\n    #[allow(clippy::unused_async)]\n    async fn list(&self) -> Result<String, HttpError> {{\n        Ok(self.service.name().to_owned())\n    }}\n}}\n",
        names.snake, names.pascal, names.kebab
    )
}

pub(crate) fn decorator(names: &Names) -> String {
    format!(
        "use ironic::{{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator}};\n\nstruct {0};\n\nimpl ParameterExtractor for {0} {{\n    fn extract<'a>(&'a self, _context: &'a mut RequestContext) -> ExtractFuture<'a> {{\n        Box::pin(async move {{\n            Ok(Box::new(String::new()) as ironic::ExtractedValue)\n        }})\n    }}\n\n    fn description(&self) -> &'static str {{\n        \"{1}\"\n    }}\n}}\n\ncreate_param_decorator!({1}, {0});\n",
        names.pascal, names.snake
    )
}

pub(crate) fn filter(names: &Names) -> String {
    format!(
        "use ironic::{{ExceptionFilter, FilterContext, FrameworkResponse, HttpError, HttpStatus}};\n\npub struct {0}Filter;\n\nimpl ExceptionFilter for {0}Filter {{\n    fn catch(&self, error: &HttpError, _ctx: &FilterContext) -> Result<FrameworkResponse, HttpError> {{\n        Err(HttpError::bad_request(\"UNHANDLED\", error.message()))\n    }}\n}}\n",
        names.pascal
    )
}

pub(crate) fn gateway(names: &Names) -> String {
    format!(
        "use ironic::{{web_socket_gateway, subscribe_message, HttpError}};\n\n#[web_socket_gateway(\"/{1}\")]\npub struct {0}Gateway;\n\n#[routes]\nimpl {0}Gateway {{\n    #[subscribe_message(\"message\")]\n    #[allow(clippy::unused_async)]\n    async fn on_message(&self, payload: String) -> Result<String, HttpError> {{\n        Ok(format!(\"Echo: {{}}\", payload))\n    }}\n}}\n",
        names.pascal, names.kebab
    )
}

pub(crate) fn guard(names: &Names) -> String {
    format!(
        "use ironic::{{Guard, GuardDecision, GuardFuture, RequestContext, HttpError}};\n\npub struct {0}Guard;\n\nimpl Guard for {0}Guard {{\n    fn can_activate<'a>(&'a self, _context: &'a mut RequestContext) -> GuardFuture<'a> {{\n        Box::pin(async move {{ Ok(GuardDecision::Allow) }})\n    }}\n}}\n",
        names.pascal
    )
}

pub(crate) fn interceptor(names: &Names) -> String {
    format!(
        "use ironic::{{Interceptor, InterceptorNext, PipelineFuture, RequestContext, HttpError}};\n\npub struct {0}Interceptor;\n\nimpl Interceptor for {0}Interceptor {{\n    fn intercept<'a>(&'a self, context: &'a mut RequestContext, next: InterceptorNext<'a>) -> PipelineFuture<'a> {{\n        Box::pin(async move {{ next.run(context).await }})\n    }}\n}}\n",
        names.pascal
    )
}

pub(crate) fn middleware(names: &Names) -> String {
    format!(
        "use ironic::{{Middleware, MiddlewareNext, PipelineFuture, RequestContext, HttpError}};\n\npub struct {0}Middleware;\n\nimpl Middleware for {0}Middleware {{\n    fn handle<'a>(&'a self, context: &'a mut RequestContext, next: MiddlewareNext<'a>) -> PipelineFuture<'a> {{\n        Box::pin(async move {{ next.run(context).await }})\n    }}\n}}\n",
        names.pascal
    )
}

pub(crate) fn pipe(names: &Names) -> String {
    format!(
        "use ironic::{{ParameterPipe, PipeFuture, ExtractedValue, RequestContext, HttpError}};\n\npub struct {0}Pipe;\n\nimpl ParameterPipe for {0}Pipe {{\n    fn transform<'a>(&'a self, value: ExtractedValue, _context: &'a mut RequestContext) -> PipeFuture<'a> {{\n        Box::pin(async move {{ Ok(value) }})\n    }}\n\n    fn description(&self) -> &'static str {{\n        \"{1}\"\n    }}\n}}\n",
        names.pascal, names.snake
    )
}

pub(crate) fn provider(names: &Names) -> String {
    format!(
        "use ironic::prelude::*;\n\n#[derive(Injectable)]\npub struct {0}Provider;\n\nimpl {0}Provider {{\n    #[must_use]\n    pub const fn name(&self) -> &'static str {{\n        \"{1}\"\n    }}\n}}\n",
        names.pascal, names.kebab
    )
}
