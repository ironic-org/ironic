use super::naming::Names;

pub(crate) fn module(pascal: &str) -> String {
    format!(
        "use rustframe::prelude::*;\n\n#[derive(Module)]\n#[module()]\npub struct {pascal}Module;\n"
    )
}

pub(crate) fn service(names: &Names) -> String {
    format!(
        "use rustframe::prelude::*;\n\n#[derive(Injectable)]\npub struct {0}Service;\n\nimpl {0}Service {{\n    #[must_use]\n    pub const fn name(&self) -> &'static str {{\n        \"{1}\"\n    }}\n}}\n",
        names.pascal, names.kebab
    )
}

pub(crate) fn controller(names: &Names) -> String {
    format!(
        "use rustframe::prelude::*;\n\n#[controller(\"/{}\")]\n#[derive(Injectable)]\npub struct {}Controller;\n\n#[routes]\nimpl {}Controller {{\n    #[get(\"/\")]\n    #[allow(clippy::unused_async)]\n    async fn list(&self) -> Result<&'static str, HttpError> {{\n        Ok(\"{} controller\")\n    }}\n}}\n",
        names.kebab, names.pascal, names.pascal, names.kebab
    )
}

pub(crate) fn resource_module(names: &Names) -> String {
    format!(
        "use rustframe::prelude::*;\n\npub mod {0}_controller;\npub mod {0}_service;\n\npub use {0}_controller::{1}Controller;\npub use {0}_service::{1}Service;\n\n#[derive(Module)]\n#[module(\n    providers = [{1}Service],\n    controllers = [{1}Controller],\n)]\npub struct {1}Module;\n",
        names.snake, names.pascal
    )
}

pub(crate) fn resource_controller(names: &Names) -> String {
    format!(
        "use std::sync::Arc;\n\nuse rustframe::prelude::*;\n\nuse super::{0}_service::{1}Service;\n\n#[controller(\"/{2}\")]\n#[derive(Injectable)]\npub struct {1}Controller {{\n    service: Arc<{1}Service>,\n}}\n\n#[routes]\nimpl {1}Controller {{\n    #[get(\"/\")]\n    #[allow(clippy::unused_async)]\n    async fn list(&self) -> Result<String, HttpError> {{\n        Ok(self.service.name().to_owned())\n    }}\n}}\n",
        names.snake, names.pascal, names.kebab
    )
}
