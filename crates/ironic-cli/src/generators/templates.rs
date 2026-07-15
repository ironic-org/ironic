use super::naming::Names;

pub(crate) fn module(pascal: &str) -> String {
    format!(
        "use ironic::prelude::*;\n\npub mod controller;\npub mod services;\npub mod dto;\npub mod entities;\n\n#[derive(Module)]\n#[module()]\npub struct {pascal}Module;\n"
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
        "use ironic::prelude::*;\n\npub mod controller;\npub mod repositories;\npub mod services;\npub mod dto;\npub mod entities;\n\n#[cfg(test)]\nmod tests;\n\npub use controller::{}Controller;\npub use repositories::{}Repository;\npub use services::{}Service;\n\n#[derive(Module)]\n#[module(\n    providers = [{}Repository, {}Service],\n    controllers = [{}Controller],\n)]\npub struct {}Module;\n",
        names.pascal, names.pascal, names.pascal, names.pascal, names.pascal, names.pascal, names.pascal
    )
}

pub(crate) fn resource_controller(names: &Names) -> String {
    format!(
        "use std::sync::Arc;\n\nuse ironic::prelude::*;\n\nuse super::super::services::{}Service;\n\n#[controller(\"/{}\")]\n#[derive(Injectable)]\npub struct {}Controller {{\n    service: Arc<{}Service>,\n}}\n\n#[routes]\nimpl {}Controller {{\n    #[get(\"/\")]\n    async fn list(&self) -> Result<String, HttpError> {{\n        Ok(self.service.name().to_owned())\n    }}\n}}\n",
        names.pascal, names.kebab, names.pascal, names.pascal, names.pascal
    )
}

pub(crate) fn controller_mod(names: &Names) -> String {
    format!(
        "pub mod {0}_controller;\npub use {0}_controller::{1}Controller;\n",
        names.snake, names.pascal
    )
}

pub(crate) fn services_mod(names: &Names) -> String {
    format!(
        "pub mod {0}_service;\npub use {0}_service::{1}Service;\n",
        names.snake, names.pascal
    )
}

pub(crate) fn dto_mod(names: &Names) -> String {
    format!(
        "pub mod create_{0}_dto;\npub mod update_{0}_dto;\n#[allow(unused_imports)]\npub use create_{0}_dto::Create{1}Dto;\n#[allow(unused_imports)]\npub use update_{0}_dto::Update{1}Dto;\n",
        names.snake, names.pascal
    )
}

pub(crate) fn entities_mod(names: &Names) -> String {
    format!(
        "pub mod {0};\n#[allow(unused_imports)]\npub use {0}::{1};\n",
        names.snake, names.pascal
    )
}

pub(crate) fn create_dto(names: &Names) -> String {
    format!(
        "use serde::{{Deserialize, Serialize}};\n\n#[allow(dead_code)]\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Create{0}Dto {{\n    pub name: String,\n}}\n",
        names.pascal
    )
}

pub(crate) fn update_dto(names: &Names) -> String {
    format!(
        "use serde::{{Deserialize, Serialize}};\n\n#[allow(dead_code)]\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Update{0}Dto {{\n    pub name: Option<String>,\n}}\n",
        names.pascal
    )
}

pub(crate) fn entity(names: &Names) -> String {
    format!(
        "use serde::{{Deserialize, Serialize}};\n\n#[allow(dead_code)]\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct {0} {{\n    pub id: String,\n    pub name: String,\n}}\n",
        names.pascal
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

pub(crate) fn repository_mod(names: &Names) -> String {
    format!(
        "pub mod {0}_repository;\npub use {0}_repository::{1}Repository;\n",
        names.snake, names.pascal
    )
}

pub(crate) fn repository(names: &Names) -> String {
    format!(
        "use ironic::prelude::*;\n\n#[derive(Injectable)]\npub struct {0}Repository;\n\nimpl {0}Repository {{\n    #[must_use]\n    pub const fn name(&self) -> &'static str {{\n        \"{1}\"\n    }}\n}}\n",
        names.pascal, names.kebab
    )
}

pub(crate) fn test_mod(_names: &Names) -> String {
    "/// Unit tests — service and business logic in isolation (no HTTP).\n#[cfg(test)]\nmod unit;\n/// Integration tests — full HTTP request/response through the framework.\n#[cfg(test)]\nmod integration;\n"
        .to_owned()
}

pub(crate) fn test_unit(names: &Names) -> String {
    format!(
        "//! Unit tests for `{0}Service` — verifies business logic without HTTP overhead.\n\nuse super::super::*;\n\n#[test]\nfn service_has_the_correct_name() {{\n    let svc = {0}Service;\n    assert_eq!(svc.name(), \"{1}\");\n}}\n\n#[test]\nfn service_is_injectable() {{\n    let _def = {0}Service::provider_definition();\n}}\n",
        names.pascal, names.kebab
    )
}

pub(crate) fn test_integration(names: &Names) -> String {
    format!(
        "//! Integration tests for `{0}` — full HTTP request/response cycles.\n//! Run with `cargo test`.\n\nuse ironic::{{HttpStatus, TestApplication}};\n\nuse super::super::*;\n\nasync fn app() -> TestApplication {{\n    TestApplication::new::<{0}Module>()\n        .await\n        .expect(\"test application must initialise\")\n}}\n\n#[tokio::test]\nasync fn list_endpoint_returns_empty_when_no_data() {{\n    let app = app().await;\n    let response = app.get(\"/{1}\").send().await;\n    assert_eq!(response.status(), HttpStatus::OK);\n    app.shutdown().await.unwrap();\n}}\n\n#[tokio::test]\nasync fn get_endpoint_returns_not_found_for_missing_id() {{\n    let app = app().await;\n    app.get(\"/{1}/999\")\n        .send()\n        .await\n        .assert_status(404);\n    app.shutdown().await.unwrap();\n}}\n",
        names.pascal, names.kebab
    )
}
