# Migrations

## Location

All migrations live in `migrations/` with a timestamp prefix:

```
migrations/
├── 20250101000000_create_todos_table.sql
```

## How they run

Migrations run automatically at startup in `src/platform/database.rs`:

```rust
sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
    .await
    .expect("invalid migrations directory")
    .run(&pool)
    .await
    .expect("failed to run migrations");
```

## Add a new migration

```bash
# Create the migration file
touch migrations/$(date -u +%Y%m%d%H%M%S)_add_category_to_todos.sql
```

Then write the SQL:

```sql
ALTER TABLE todos ADD COLUMN category VARCHAR(100);
```

Restart the app — migrations run automatically.
