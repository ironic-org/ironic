use uuid::Uuid;
use ironic::prelude::*;
use tracing::instrument;
use crate::modules::todos::dto::{CreateTodoDto, UpdateTodoDto};
use crate::modules::todos::entities::Todo;

fn db() -> &'static sqlx::PgPool {
    crate::platform::database::db()
}

#[derive(Injectable)]
pub struct TodoService;

impl TodoService {
    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    pub async fn find(&self, id: Uuid) -> Result<Todo, HttpError> {
        sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
            .bind(id)
            .fetch_optional(db())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to find todo");
                HttpError::internal("DB_ERROR", e.to_string())
            })?
            .ok_or_else(|| HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found")))
    }

    #[instrument(skip(self))]
    pub async fn create(&self, dto: CreateTodoDto) -> Result<Todo, HttpError> {
        sqlx::query_as::<_, Todo>(
            "INSERT INTO todos (title, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(&dto.title)
        .bind(&dto.description)
        .fetch_one(db())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to create todo");
            HttpError::internal("DB_ERROR", e.to_string())
        })
    }

    #[instrument(skip(self))]
    pub async fn update(&self, id: Uuid, dto: UpdateTodoDto) -> Result<Todo, HttpError> {
        let current = self.find(id).await?;

        let title = dto.title.unwrap_or(current.title);
        let description = dto.description.or(current.description);
        let completed = dto.completed.unwrap_or(current.completed);

        sqlx::query_as::<_, Todo>(
            "UPDATE todos SET title = $1, description = $2, completed = $3, updated_at = NOW() WHERE id = $4 RETURNING *",
        )
        .bind(&title)
        .bind(&description)
        .bind(completed)
        .bind(id)
        .fetch_one(db())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to update todo");
            HttpError::internal("DB_ERROR", e.to_string())
        })
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), HttpError> {
        let rows = sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(db())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to delete todo");
                HttpError::internal("DB_ERROR", e.to_string())
            })?
            .rows_affected();

        if rows == 0 {
            return Err(HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found")));
        }

        tracing::info!(todo_id = %id, "todo deleted");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn toggle(&self, id: Uuid) -> Result<Todo, HttpError> {
        self.find(id).await?;

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

    #[instrument(skip(self))]
    pub async fn delete_completed(&self) -> Result<u64, HttpError> {
        let result = sqlx::query("DELETE FROM todos WHERE completed = TRUE")
            .execute(db())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to clear completed todos");
                HttpError::internal("DB_ERROR", e.to_string())
            })?;

        let count = result.rows_affected();
        tracing::info!(count, "cleared completed todos");
        Ok(count)
    }
}
