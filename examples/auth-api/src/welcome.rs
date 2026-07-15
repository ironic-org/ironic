use ironic::prelude::*;

#[controller("/")]
#[derive(Injectable)]
struct WelcomeController;

#[routes]
impl WelcomeController {
    #[get]
    async fn index(&self) -> Result<Json<serde_json::Value>, HttpError> {
        Ok(Json(serde_json::json!({
            "name": "auth-api",
            "framework": "Ironic",
            "version": "0.3.5",
            "status": "running",
            "health": "/health",
            "docs": "/docs"
        })))
    }
}

#[derive(Module)]
#[module(controllers = [WelcomeController])]
pub struct WelcomeModule;
