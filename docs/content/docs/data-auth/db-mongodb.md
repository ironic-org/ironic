---
title: MongoDB
description: Document storage with MongoDB — repositories, ObjectId handling, and manual client configuration.
---

# MongoDB


## Feature flag

```toml
ironic = { features = ["mongodb"] }
```

## Connection URL

```
mongodb://user:password@localhost:27017/mydb
```

## Repository

```rust
use mongodb::{Client, Collection, bson::{doc, Document, oid::ObjectId}};

#[derive(Injectable)]
pub class UserService {
    db: Arc<Client>,
}

impl UserService {
    fn users(&self) -> Collection<Document> {
        self.db.database("mydb").collection("users")
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Document, HttpError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| HttpError::bad_request("INVALID_ID", "Invalid ObjectId"))?;

        self.users()
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("NOT_FOUND", format!("User {id} not found")))
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<Document, HttpError> {
        let doc = doc! {
            "name": name,
            "email": email,
            "created_at": chrono::Utc::now().to_rfc3339(),
        };

        let result = self.users()
            .insert_one(doc)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        self.find_by_id(&result.inserted_id.as_object_id().unwrap().to_hex())
            .await
    }

    pub async fn list(&self) -> Result<Vec<Document>, HttpError> {
        let mut cursor = self.users()
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .limit(50)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        let mut results = Vec::new();
        while cursor.advance().await.map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))? {
            results.push(cursor.deserialize_current().unwrap());
        }

        Ok(results)
    }
}
```

---

## Manual connection setup


When you need full control over pool configuration, build the connection manually and register it as a provider:

### MongoDB

```rust
use mongodb::{Client, options::ClientOptions};

let mut options = ClientOptions::parse("mongodb://user:password@localhost:27017/mydb")
    .await?;

options.max_pool_size = Some(20);
options.min_pool_size = Some(5);
options.connect_timeout = Some(Duration::from_secs(10));
options.server_selection_timeout = Some(Duration::from_secs(5));
options.app_name = Some("ironic-app".to_string());

let client = Client::with_options(options)?;
let client: Arc<Client> = Arc::new(client);
```
