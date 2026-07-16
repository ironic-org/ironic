---
title: Database Migrations
description: Manage schema changes with Ironic's built-in migration CLI — create, apply, revert, and inspect migrations using SQLx under the hood.
---

# Database Migrations

## What you'll learn

- Create timestamped migration files with `ironic migrate create`
- Apply pending migrations against your database
- Revert migrations when you need to roll back
- Check which migrations are applied or pending
- Connect to PostgreSQL via DATABASE_URL and `.env`
- Best practices for migration workflows in development and production

---

## Overview

Ironic provides a first-class migration CLI built on top of `sqlx::migrate::Migrator`. Unlike using `sqlx-cli` directly, Ironic's `migrate` commands are integrated into the same binary as your application, sharing feature flags and configuration conventions.

| Command | Requires | Description |
|---------|----------|-------------|
| `ironic migrate create <name>` | Always available | Create a new timestamped SQL migration file |
| `ironic migrate up` | `sqlx-postgres` | Run all pending migrations |
| `ironic migrate down` | `sqlx-postgres` | Revert the last N migrations |
| `ironic migrate status` | `sqlx-postgres` | Show which migrations are applied vs pending |

> **Note:** `up`, `down`, and `status` require a SQLx database backend feature (`sqlx-postgres`, `sqlx-mysql`, or `sqlx-sqlite`). Without it, Ironic prints a message telling you to install with the feature flag. `create` is always available — it only writes a file and needs no database connection.

---

## Feature flags

Add the appropriate feature to your `Cargo.toml`:

```toml
[dependencies]
ironic = { features = ["sqlx-postgres"] }
```

| Feature | Database | Pool type |
|---------|----------|-----------|
| `sqlx-postgres` | PostgreSQL | `sqlx::PgPool` |
| `sqlx-mysql` | MySQL | `sqlx::MySqlPool` |
| `sqlx-sqlite` | SQLite | `sqlx::SqlitePool` |

Install the CLI with:

```bash
cargo install ironic --features sqlx-postgres
```

---

## Database URL

All `ironic migrate` commands that connect to the database read the connection string from the `DATABASE_URL` environment variable.

### Via environment variable

```bash
export DATABASE_URL=postgres://user:password@localhost:5432/myapp
ironic migrate up
```

### Via `.env` file

If `DATABASE_URL` is not set in the environment, Ironic looks for a `.env` file in the current directory:

```bash
# .env
DATABASE_URL=postgres://user:password@localhost:5432/myapp
```

The parser handles comments (`#`), empty lines, and quoted values:

```bash
# .env — all valid forms
DATABASE_URL=postgres://user:password@localhost:5432/myapp
DATABASE_URL="postgres://user:pass@localhost:5432/myapp"
```

If neither source provides a URL, the command fails with a clear message.

---

## Creating migrations

```bash
ironic migrate create add_users_table
```

This creates a file in `./migrations/` with a Unix timestamp prefix:

```
migrations/
└── 1742169600_add_users_table.sql
```

The generated file includes a header comment:

```sql
-- Migration: add_users_table
-- Created: 1742169600

-- Write your up SQL here
```

Edit the file and add your schema change:

```sql
-- Migration: add_users_table
-- Created: 1742169600

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Naming conventions

| Name | Example file |
|------|-------------|
| `create_users_table` | `1742169600_create_users_table.sql` |
| `add_email_to_users` | `1742169601_add_email_to_users.sql` |
| `create_orders_and_items` | `1742169602_create_orders_and_items.sql` |

Use descriptive snake_case names. Ironic does not enforce a naming convention, but consistency makes `status` output easier to read.

### Conflict detection

If a file with the same name already exists, `migrate create` returns an error:

```
RF_CLI_FILE_CONFLICT: refusing to overwrite `./migrations/1742169600_add_users_table.sql`
```

Migration files are meant to be committed to version control and never renamed or reordered once shared with a team.

---

## Applying migrations

```bash
ironic migrate up
```

This connects to your database, reads all `.sql` files from `./migrations/`, and applies any that have not yet been recorded in the `_sqlx_migrations` table.

Output on success:

```
  ✓ Migrations applied successfully
```

### What happens under the hood

1. Ironic reads every `.sql` file from `./migrations/` and sorts by timestamp
2. `sqlx::migrate::Migrator` checks the `_sqlx_migrations` tracking table
3. Each un-applied migration runs inside its own database transaction
4. On success, the migration version and checksum are recorded

The `_sqlx_migrations` table is created automatically if it does not exist:

```sql
CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time_ms BIGINT NOT NULL
);
```

### At application startup

You should also run migrations programmatically at boot so every deployment applies pending changes automatically:

```rust
use sqlx::migrate::Migrator;
use std::path::Path;

pub async fn run_migrations(pool: &PgPool) -> Result<(), HttpError> {
    let migrator = Migrator::new(Path::new("./migrations"))
        .await
        .map_err(|e| HttpError::internal("MIGRATE_LOAD", e.to_string()))?;

    migrator
        .run(pool)
        .await
        .map_err(|e| HttpError::internal("MIGRATE_RUN", e.to_string()))
}
```

Ironic's CLI `up` command is useful during development and in CI, while programmatic migration at startup ensures production deployments never miss a migration even if the CLI is not invoked separately.

---

## Reverting migrations

```bash
# Revert the last migration
ironic migrate down --steps 1

# Revert the last 3 migrations
ironic migrate down --steps 3
```

Output:

```
  ✓ Reverted 1 migration(s)
```

### How revert works

1. Ironic reads the `_sqlx_migrations` table for the most recent `N` applied versions
2. Each entry is reversed by running its SQL in `down` mode — but **sqlx migrations are unidirectional by default**
3. The migration is then removed from `_sqlx_migrations`

### Writing reversible migrations

To support `down`, include the rollback SQL in your migration file using `-- down:` tag comments. sqlx's `undo()` runs the entire file, so you must structure it carefully. The recommended pattern is:

```sql
-- Create the users table (up)

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL
);
```

For a true revert, create a separate `down` migration file and apply it explicitly. The `--steps` parameter in `ironic migrate down` removes the version tracking entry; the actual table drop requires a separate SQL statement (or you can ALTER/drop in the reverse order).

> **Best practice:** In most production workflows, migrations are additive-only. Instead of reverting, create a new migration that alters or drops the problematic schema. `ironic migrate down` is most useful in local development when iterating rapidly.

---

## Checking status

```bash
ironic migrate status
```

Output:

```
  Migration status:

    ✓ Applied      1742169600  create_users_table
    ✓ Applied      1742169601  add_email_to_users
    ⏳ Pending     1742169602  create_orders

  Total: 3 migrations, 2 applied
```

This queries the `_sqlx_migrations` table and compares it against the migration files on disk. Migrations present on disk but missing from the database are shown as **Pending**. Migrations recorded in both are shown as **Applied**.

Use `status` to:
- Verify which migrations have been applied before a deploy
- Check if a colleague's branch introduced new migrations
- Debug a failed migration (the database stores checksums — a mismatch indicates tampering or file changes)

---

## Migration lifecycle

### Development workflow

```bash
# 1. Create a migration
ironic migrate create add_user_preferences

# 2. Edit the SQL file
vim migrations/1742169603_add_user_preferences.sql

# 3. Apply it
ironic migrate up

# 4. Check it worked
ironic migrate status

# 5. Iterate — create a new migration if you need to change the schema
ironic migrate create add_preferences_index
```

### Team workflow

1. Developer A creates a migration and commits it
2. Developer B pulls the branch and runs `ironic migrate up`
3. The migration applies only once thanks to the `_sqlx_migrations` tracking table
4. On merge to `main`, CI runs `ironic migrate up` against the staging database
5. Production deployment runs programmatic migrations at startup

### Production considerations

- **Never edit an applied migration file** — the checksum will not match and sqlx rejects it
- **Always create new migrations** for changes, even if they modify the same table
- **Commit migration files** to version control alongside your code
- **Run migrations before** your application starts serving traffic (Ironic's startup sequence handles this)

---

## Comparison: `ironic migrate` vs `sqlx-cli`

| Aspect | `ironic migrate` | `sqlx-cli` |
|--------|-----------------|------------|
| Installation | Bundled with `ironic` CLI | Separate `cargo install sqlx-cli` |
| Feature flags | `sqlx-postgres`, `sqlx-mysql`, `sqlx-sqlite` | Database-specific binaries (e.g., `sqlx-cli --features postgres`) |
| Migration directory | Always `./migrations/` | Configurable |
| File format | Timestamped `.sql` with header | Timestamped `.sql` |
| `create` command | Always available without DB | Requires `DATABASE_URL` |
| `status` output | Tabular with ✓/⏳ icons | Tabular with status column |
| Programmatic use | Same `sqlx::Migrator` under the hood | Same `sqlx::Migrator` under the hood |
| Scope | Ironic project conventions | Works with any Rust project using SQLx |

Both tools use `sqlx::migrate::Migrator` internally, so they are compatible. You can switch between them freely — the `_sqlx_migrations` table format is identical.

---

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Running `migrate create` without a `migrations/` directory | Ironic creates it automatically |
| Forgetting to set `DATABASE_URL` | Set it in the environment or `.env` file |
| Editing an applied migration | Create a new migration instead — checksums will not match |
| Running `down` in production | Use additive migrations — `down` is for local development |
| Installing without `sqlx-postgres` | `cargo install ironic --features sqlx-postgres` |
| Migration file with syntax errors | Test with `ironic migrate up` on a local database first |
| Committing `target/` or `migrations/*.sql` backup files | Add `*.bak` to `.gitignore` |

---

## What you learned

- [x] Create timestamped migration files with `ironic migrate create`
- [x] Apply pending migrations with `ironic migrate up`
- [x] Revert migrations with `ironic migrate down --steps N`
- [x] Inspect migration state with `ironic migrate status`
- [x] Configure `DATABASE_URL` via environment or `.env`
- [x] Understand the migration lifecycle from dev to production
- [x] Choose between `ironic migrate` and `sqlx-cli`
- [x] Avoid common migration pitfalls

## Next steps

- [Writing database providers](/docs/data-auth/database-integrations)
- [CLI reference](/docs/getting-started/cli)
- [Deployment guide](/docs/more/deployment)
