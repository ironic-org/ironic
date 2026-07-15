# Todos API

Base path: `/api/todos`

## List todos

```http
GET /api/todos?include_completed=false
```

Query parameters:

| Param | Type | Default | Description |
|---|---|---|---|
| `include_completed` | `bool` | `false` | Include completed todos in results |

Response `200`:

```json
[
    {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "Buy groceries",
        "description": "Milk, eggs, bread",
        "completed": false,
        "created_at": "2026-07-15T12:00:00Z",
        "updated_at": "2026-07-15T12:00:00Z"
    }
]
```

## Get todo

```http
GET /api/todos/:id
```

Response `200` — single todo object.  
Response `404` — not found.

## Create todo

```http
POST /api/todos
Content-Type: application/json

{
    "title": "Buy groceries",
    "description": "Milk, eggs, bread"
}
```

Validation rules:

| Field | Rule |
|---|---|
| `title` | Required, 1–500 characters |
| `description` | Optional |

Response `200` — the created todo with generated UUID.  
Response `400` — validation error.

## Update todo

```http
PUT /api/todos/:id
Content-Type: application/json

{
    "title": "Updated title",
    "description": "Updated description",
    "completed": true
}
```

All fields are optional — partial updates supported.  
Response `200` — the updated todo.  
Response `404` — not found.

## Delete todo

```http
DELETE /api/todos/:id
```

Response `200` — deleted.  
Response `404` — not found.

## Toggle completed

```http
POST /api/todos/:id/toggle
```

Flips `completed` to its opposite value.  
Response `200` — toggled todo.

## Clear completed

```http
DELETE /api/todos/completed
```

Deletes all completed todos.  
Response `200`:

```json
{
    "deleted": 3
}
```

## Error format

```json
{
    "error": "TODO_NOT_FOUND",
    "message": "Todo 550e8400-e29b-41d4-a716-446655440000 not found",
    "status": 404
}
```
