use std::sync::Arc;

use rustframe::prelude::*;

struct Allow;

impl Guard for Allow {
    fn can_activate<'a>(&'a self, _context: &'a mut RequestContext) -> GuardFuture<'a> {
        Box::pin(async { Ok(GuardDecision::Allow) })
    }
}

struct Around;

impl Interceptor for Around {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        next.run(context)
    }
}

#[derive(Injectable)]
#[injectable(eager)]
struct Service;

#[derive(Injectable)]
#[injectable(scope = "transient")]
struct TransientService;

#[controller("/items")]
#[use_guard(Allow)]
#[use_interceptor(Around)]
#[derive(Injectable)]
struct ItemsController {
    service: Arc<Service>,
}

#[routes]
impl ItemsController {
    #[get("/:id")]
    #[use_guard(Allow)]
    #[use_interceptor(Around)]
    async fn get(&self, #[param] id: u64, #[header("x-name")] name: String) -> Result<String, HttpError> {
        let _ = &self.service;
        Ok(format!("{id}:{name}"))
    }

    #[post]
    async fn post(&self, #[body] value: String) -> Result<String, HttpError> { Ok(value) }

    #[put]
    async fn put(&self, #[query] value: String) -> Result<String, HttpError> { Ok(value) }

    #[patch]
    async fn patch(&self) -> Result<(), HttpError> { Ok(()) }

    #[delete]
    async fn delete(&self) -> Result<(), HttpError> { Ok(()) }

    #[head]
    async fn head(&self) -> Result<(), HttpError> { Ok(()) }

    #[options]
    async fn options(&self) -> Result<(), HttpError> { Ok(()) }
}

#[derive(Module)]
#[module(
    providers = [Service, TransientService],
    controllers = [ItemsController],
    exports = [Service]
)]
struct FeatureModule;

#[derive(Module)]
#[module(imports = [FeatureModule])]
struct AppModule;

#[rustframe::main]
async fn main() {
    let _ = AppModule::definition();
}
