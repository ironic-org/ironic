use ironic::prelude::*;

#[controller("/")]
#[derive(Injectable)]
struct WelcomeController;

#[routes]
impl WelcomeController {
    #[get]
    async fn index(&self) -> Result<Json<serde_json::Value>, HttpError> {
        Ok(Json(serde_json::json!({
            "name": "todo-example",
            "framework": "Ironic",
            "version": "0.4.0",
            "status": "running",
            "endpoints": {
                "api": "/api/todos",
                "health": "/health",
                "docs": "/docs"
            }
        })))
    }
}

#[derive(Module)]
#[module(controllers = [WelcomeController])]
pub struct WelcomeModule;
