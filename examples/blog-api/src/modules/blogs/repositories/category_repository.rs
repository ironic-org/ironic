use std::collections::HashMap;
use std::sync::Mutex;

use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::blogs::entities::Category;

static CATEGORIES: std::sync::LazyLock<Mutex<HashMap<Uuid, Category>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Injectable)]
pub struct CategoryRepository;

#[allow(dead_code)]
impl CategoryRepository {
    pub fn list(&self) -> Result<Vec<Category>, HttpError> {
        let cats = CATEGORIES.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        let mut result: Vec<Category> = cats.values().cloned().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    pub fn find(&self, id: Uuid) -> Result<Option<Category>, HttpError> {
        let cats = CATEGORIES.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        Ok(cats.get(&id).cloned())
    }

    pub fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, HttpError> {
        let cats = CATEGORIES.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        Ok(cats.values().find(|c| c.slug == slug).cloned())
    }

    pub fn create(&self, category: Category) -> Result<Category, HttpError> {
        let mut cats = CATEGORIES.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        cats.insert(category.id, category.clone());
        Ok(category)
    }

    pub fn update(&self, category: Category) -> Result<Category, HttpError> {
        let mut cats = CATEGORIES.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        cats.insert(category.id, category.clone());
        Ok(category)
    }

    pub fn delete(&self, id: Uuid) -> Result<bool, HttpError> {
        let mut cats = CATEGORIES.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        Ok(cats.remove(&id).is_some())
    }
}
