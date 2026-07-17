use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::blogs::BlogService;

#[derive(Injectable)]
pub struct StatsService {
    blog_service: Arc<BlogService>,
}

impl StatsService {
    pub fn blog_summary(&self) -> Result<serde_json::Value, HttpError> {
        let stats = self.blog_service.stats()?;
        Ok(serde_json::json!({
            "totalPosts": stats.total,
            "publishedPosts": stats.published,
            "draftPosts": stats.draft,
            "totalWords": stats.total_words,
            "uniqueTags": stats.unique_tags,
        }))
    }

    pub fn tag_breakdown(&self) -> Result<serde_json::Value, HttpError> {
        let posts = self
            .blog_service
            .list(&crate::modules::blogs::dto::BlogFilterDto {
                published: None,
                author: None,
                tag: None,
                category_id: None,
                search: None,
            })?;

        let mut tag_counts: std::collections::BTreeMap<String, u64> =
            std::collections::BTreeMap::new();
        for post in &posts {
            for tag in &post.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let tags: Vec<serde_json::Value> = tag_counts
            .into_iter()
            .map(|(tag, count)| {
                serde_json::json!({
                    "tag": tag,
                    "count": count,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "totalPosts": posts.len(),
            "tags": tags,
        }))
    }
}
