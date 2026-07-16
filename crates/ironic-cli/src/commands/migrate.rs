use std::{
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{CliError, cli::MigrateAction};

const MIGRATIONS_DIR: &str = "./migrations";

#[cfg(not(feature = "sqlx-postgres"))]
const NO_SQLX_MSG: &str =
    "The `migrate up|down|status` commands require the `sqlx` feature.\n\
     Install with: cargo install ironic --features sqlx-postgres";

pub(crate) fn execute(action: MigrateAction, output: &mut impl Write) -> Result<(), CliError> {
    match action {
        MigrateAction::Create { name } => create_migration(&name, output),
        #[cfg(feature = "sqlx-postgres")]
        MigrateAction::Up => run_migrations(output),
        #[cfg(not(feature = "sqlx-postgres"))]
        MigrateAction::Up => {
            writeln!(output, "{NO_SQLX_MSG}")?;
            Ok(())
        }
        #[cfg(feature = "sqlx-postgres")]
        MigrateAction::Down { steps } => revert_migrations(steps, output),
        #[cfg(not(feature = "sqlx-postgres"))]
        MigrateAction::Down { .. } => {
            writeln!(output, "{NO_SQLX_MSG}")?;
            Ok(())
        }
        #[cfg(feature = "sqlx-postgres")]
        MigrateAction::Status => show_status(output),
        #[cfg(not(feature = "sqlx-postgres"))]
        MigrateAction::Status => {
            writeln!(output, "{NO_SQLX_MSG}")?;
            Ok(())
        }
    }
}

fn create_migration(name: &str, output: &mut impl Write) -> Result<(), CliError> {
    let dir = Path::new(MIGRATIONS_DIR);
    fs::create_dir_all(dir)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let filename = format!("{timestamp}_{name}.sql");
    let path = dir.join(&filename);

    if path.exists() {
        return Err(CliError::FileConflict { path });
    }

    let mut file = fs::File::create(&path)?;
    writeln!(file, "-- Migration: {name}")?;
    writeln!(file, "-- Created: {timestamp}")?;
    writeln!(file)?;
    writeln!(file, "-- Write your up SQL here")?;
    writeln!(file)?;

    writeln!(output, "  ✓ Created {filename}")?;
    Ok(())
}

#[cfg(feature = "sqlx-postgres")]
fn load_database_url() -> Result<String, CliError> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }
    if let Ok(content) = fs::read_to_string("./.env") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some(value) = line.strip_prefix("DATABASE_URL=") {
                return Ok(value.trim_matches('"').to_owned());
            }
        }
    }
    Err(CliError::CommandFailed {
        program: "ironic migrate".into(),
        status: "DATABASE_URL is not set — check your .env file or set the DATABASE_URL environment variable".into(),
    })
}

#[cfg(feature = "sqlx-postgres")]
type DbPool = sqlx::PgPool;

#[cfg(feature = "sqlx-postgres")]
async fn connect() -> Result<DbPool, CliError> {
    let url = load_database_url()?;
    DbPool::connect(&url).await.map_err(|e| CliError::CommandFailed {
        program: "ironic migrate".into(),
        status: format!("failed to connect to database: {e}"),
    })
}

#[cfg(feature = "sqlx-postgres")]
fn run_migrations(output: &mut impl Write) -> Result<(), CliError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::CommandFailed {
        program: "ironic migrate".into(),
        status: format!("failed to start runtime: {e}"),
    })?;
    rt.block_on(async {
        let pool = connect().await?;
        let migrator = sqlx::migrate::Migrator::new(Path::new(MIGRATIONS_DIR))
            .await
            .map_err(|e| CliError::CommandFailed {
                program: "ironic migrate".into(),
                status: format!("failed to load migrations: {e}"),
            })?;

        migrator.run(&pool).await.map_err(|e| CliError::CommandFailed {
            program: "ironic migrate".into(),
            status: format!("migration run failed: {e}"),
        })?;

        writeln!(output, "  ✓ Migrations applied successfully")?;
        Ok(())
    })
}

#[cfg(feature = "sqlx-postgres")]
fn revert_migrations(steps: i64, output: &mut impl Write) -> Result<(), CliError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::CommandFailed {
        program: "ironic migrate".into(),
        status: format!("failed to start runtime: {e}"),
    })?;
    rt.block_on(async {
        let pool = connect().await?;
        let migrator = sqlx::migrate::Migrator::new(Path::new(MIGRATIONS_DIR))
            .await
            .map_err(|e| CliError::CommandFailed {
                program: "ironic migrate".into(),
                status: format!("failed to load migrations: {e}"),
            })?;

        migrator.undo(&pool, steps).await.map_err(|e| CliError::CommandFailed {
            program: "ironic migrate".into(),
            status: format!("migration revert failed: {e}"),
        })?;

        writeln!(output, "  ✓ Reverted {steps} migration(s)")?;
        Ok(())
    })
}

#[cfg(feature = "sqlx-postgres")]
fn show_status(output: &mut impl Write) -> Result<(), CliError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| CliError::CommandFailed {
        program: "ironic migrate".into(),
        status: format!("failed to start runtime: {e}"),
    })?;
    rt.block_on(async {
        let pool = connect().await?;
        let migrator = sqlx::migrate::Migrator::new(Path::new(MIGRATIONS_DIR))
            .await
            .map_err(|e| CliError::CommandFailed {
                program: "ironic migrate".into(),
                status: format!("failed to load migrations: {e}"),
            })?;

        let applied: Vec<(String,)> = sqlx::query_as(
            "SELECT version::text FROM _sqlx_migrations ORDER BY version"
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        writeln!(output, "  Migration status:")?;
        writeln!(output)?;

        for migration in migrator.migrations.iter() {
            let version_str = migration.version.to_string();
            let status = if applied.iter().any(|(v,)| *v == version_str) {
                "  ✓ Applied  "
            } else {
                "  ⏳ Pending "
            };
            writeln!(output, "  {status}  {version_str:>8}  {}", migration.description)?;
        }

        writeln!(output)?;
        writeln!(
            output,
            "  Total: {}, applied: {}",
            migrator.migrations.len(),
            applied.len(),
        )?;
        Ok(())
    })
}
