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
    #[cache(ttl_secs = 30)]
    #[api(summary = "Blog statistics", tag = "Stats")]
    #[resp(200, "Blog statistics")]
    async fn blog_stats(&self) -> Result<Json<Value>, HttpError> {
        let stats = self.service.blog_summary()?;
        Ok(Json(stats))
    }

    #[get("/blog/tags")]
    #[cache(ttl_secs = 30)]
    #[api(summary = "Tag breakdown", tag = "Stats")]
    #[resp(200, "Tag breakdown")]
    async fn tag_breakdown(&self) -> Result<Json<Value>, HttpError> {
        let breakdown = self.service.tag_breakdown()?;
        Ok(Json(breakdown))
    }
}
