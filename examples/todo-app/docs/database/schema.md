# Database schema

## Table: `todos`

| Column | Type | Constraints | Description |
|---|---|---|---|
| `id` | `UUID` | `PK DEFAULT uuid_generate_v4()` | Primary key |
| `title` | `VARCHAR(500)` | `NOT NULL` | Todo title |
| `description` | `TEXT` | nullable | Optional description |
| `completed` | `BOOLEAN` | `NOT NULL DEFAULT FALSE` | Completion status |
| `created_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | Creation timestamp |
| `updated_at` | `TIMESTAMPTZ` | `NOT NULL DEFAULT NOW()` | Last update timestamp |

## Indexes

| Name | Columns |
|---|---|
| `idx_todos_completed` | `completed` |
| `idx_todos_created_at` | `created_at DESC` |

## Migration

File: `migrations/20250101000000_create_todos_table.sql`

```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE todos (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(500) NOT NULL,
    description TEXT,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_todos_completed ON todos(completed);
CREATE INDEX idx_todos_created_at ON todos(created_at DESC);
```
