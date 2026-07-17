use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::stats::services::StatsService;

#[controller("/api/stats")]
#[derive(Injectable)]
pub struct StatsController {
    service: Arc<StatsService>,
}

#[routes]
impl StatsController {
    #[get("/blog")]
    async fn blog_stats(&self) -> Result<Json<serde_json::Value>, HttpError> {
        let stats = self.service.blog_summary()?;
        Ok(Json(stats))
    }

    #[get("/blog/tags")]
    async fn tag_breakdown(&self) -> Result<Json<serde_json::Value>, HttpError> {
        let breakdown = self.service.tag_breakdown()?;
        Ok(Json(breakdown))
    }
}
