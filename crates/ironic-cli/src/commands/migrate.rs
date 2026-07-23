use std::{
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{CliError, cli::MigrateAction};

const MIGRATIONS_DIR: &str = "./migrations";

#[cfg(not(feature = "sqlx-postgres"))]
const NO_SQLX_MSG: &str = "The `migrate up|down|status` commands require the `sqlx` feature.\n\
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

#[cfg(test)]
mod tests {
    use crate::CliError;

    #[test]
    fn migrations_dir_constant() {
        assert_eq!(super::MIGRATIONS_DIR, "./migrations");
    }

    #[cfg(not(feature = "sqlx-postgres"))]
    #[test]
    fn no_sqlx_message_is_not_empty() {
        let msg = super::NO_SQLX_MSG;
        assert!(!msg.is_empty());
        assert!(msg.contains("sqlx"));
    }

    /// Runs all CWD-sensitive migration tests sequentially to avoid interference.
    #[test]
    fn migration_operations() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        // create_migration_creates_file
        let mut output = Vec::new();
        super::create_migration("create_users", &mut output).unwrap();
        let entries: Vec<_> = std::fs::read_dir("./migrations").unwrap().collect();
        assert_eq!(entries.len(), 1);
        let filename = entries[0].as_ref().unwrap().file_name().to_string_lossy().to_string();
        assert!(filename.ends_with("_create_users.sql"));
        let content = std::fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
        assert!(content.contains("-- Migration: create_users"));
        assert!(content.contains("-- Write your up SQL here"));
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("✓ Created"));

        // create_migration_rejects_duplicate
        let mut out = Vec::new();
        let result = super::create_migration("create_users", &mut out);
        assert!(matches!(result, Err(CliError::FileConflict { .. })));

        // create_migration_works_with_any_name
        let mut out2 = Vec::new();
        super::create_migration("add_index_to_users_email", &mut out2).unwrap();
        super::create_migration("create_orders_table", &mut out2).unwrap();
        let count = std::fs::read_dir("./migrations").unwrap().count();
        assert_eq!(count, 3);

        #[cfg(feature = "sqlx-postgres")]
        {
            let result = super::load_database_url();
            assert!(result.is_err());
        }

        std::env::set_current_dir(original).unwrap();
    }
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
    DbPool::connect(&url)
        .await
        .map_err(|e| CliError::CommandFailed {
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

        migrator
            .run(&pool)
            .await
            .map_err(|e| CliError::CommandFailed {
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

        migrator
            .undo(&pool, steps)
            .await
            .map_err(|e| CliError::CommandFailed {
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

        let applied: Vec<(String,)> =
            sqlx::query_as("SELECT version::text FROM _sqlx_migrations ORDER BY version")
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
            writeln!(
                output,
                "  {status}  {version_str:>8}  {}",
                migration.description
            )?;
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
