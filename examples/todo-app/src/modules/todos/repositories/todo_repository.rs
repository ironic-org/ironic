use crate::modules::todos::entities::Todo;
use ironic::prelude::*;
use uuid::Uuid;

fn db() -> &'static sqlx::PgPool {
    crate::platform::database::db()
}

#[derive(Injectable)]
pub struct TodoRepository;

impl TodoRepository {
    pub async fn list(&self, include_completed: bool) -> Result<Vec<Todo>, HttpError> {
        let todos = if include_completed {
            sqlx::query_as::<_, Todo>("SELECT * FROM todos ORDER BY created_at DESC")
                .fetch_all(db())
                .await
        } else {
            sqlx::query_as::<_, Todo>(
                "SELECT * FROM todos WHERE completed = FALSE ORDER BY created_at DESC",
            )
            .fetch_all(db())
            .await
        };

        todos.map_err(|e| {
            tracing::error!(error = %e, "failed to list todos");
            HttpError::internal("DB_ERROR", e.to_string())
        })
    }

    pub async fn find(&self, id: Uuid) -> Result<Option<Todo>, HttpError> {
        sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
            .bind(id)
            .fetch_optional(db())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to find todo");
                HttpError::internal("DB_ERROR", e.to_string())
            })
    }

    pub async fn create(
        &self,
        title: &str,
        description: &Option<String>,
    ) -> Result<Todo, HttpError> {
        sqlx::query_as::<_, Todo>(
            "INSERT INTO todos (title, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(title)
        .bind(description)
        .fetch_one(db())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to create todo");
            HttpError::internal("DB_ERROR", e.to_string())
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        title: &str,
        description: &Option<String>,
        completed: bool,
    ) -> Result<Todo, HttpError> {
        sqlx::query_as::<_, Todo>(
            "UPDATE todos SET title = $1, description = $2, completed = $3, updated_at = NOW() WHERE id = $4 RETURNING *",
        )
        .bind(title)
        .bind(description)
        .bind(completed)
        .bind(id)
        .fetch_one(db())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to update todo");
            HttpError::internal("DB_ERROR", e.to_string())
        })
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, HttpError> {
        let rows = sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(db())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to delete todo");
                HttpError::internal("DB_ERROR", e.to_string())
            })?
            .rows_affected();

        Ok(rows > 0)
    }

    pub async fn toggle(&self, id: Uuid) -> Result<Todo, HttpError> {
        sqlx::query_as::<_, Todo>(
            "UPDATE todos SET completed = NOT completed, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(db())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to toggle todo");
            HttpError::internal("DB_ERROR", e.to_string())
        })
    }

    pub async fn delete_completed(&self) -> Result<u64, HttpError> {
        let result = sqlx::query("DELETE FROM todos WHERE completed = TRUE")
            .execute(db())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to clear completed todos");
                HttpError::internal("DB_ERROR", e.to_string())
            })?;

        Ok(result.rows_affected())
    }
}
