use std::sync::OnceLock;

pub static DB_POOL: OnceLock<sqlx::PgPool> = OnceLock::new();

pub fn db() -> &'static sqlx::PgPool {
    DB_POOL
        .get()
        .expect("DATABASE_URL must be set and pool initialized")
}

pub async fn build_pool() -> sqlx::PgPool {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(
            super::config::env("DB_POOL_SIZE")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        )
        .connect(&url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("invalid migrations directory")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    tracing::info!("database pool ready (max: {})", pool.size());
    pool
}
