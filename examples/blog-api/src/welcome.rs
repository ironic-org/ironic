use ironic::prelude::*;

#[controller("/")]
#[derive(Injectable)]
struct WelcomeController;

#[routes]
impl WelcomeController {
    #[get]
    async fn index(&self) -> Result<Json<Value>, HttpError> {
        Ok(Json(ironic::json::json!({
            "name": "blog-api",
            "framework": "Ironic",
            "version": env!("CARGO_PKG_VERSION"),
            "status": "running",
            "endpoints": {
                "api": "/api/blogs",
                "categories": "/api/categories",
                "stats": "/api/stats",
                "health": "/health",
            }
        })))
    }
}

#[derive(Module)]
#[module(controllers = [WelcomeController])]
pub struct WelcomeModule;
