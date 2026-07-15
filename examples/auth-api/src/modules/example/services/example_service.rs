use crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};
use crate::modules::example::entities::Example;
use ironic::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Injectable)]
pub struct ExampleService;

static STORE: std::sync::LazyLock<Mutex<Store>> = std::sync::LazyLock::new(|| {
    Mutex::new(Store {
        items: HashMap::new(),
        next_id: 1,
    })
});

struct Store {
    items: HashMap<u64, Example>,
    next_id: u64,
}

impl ExampleService {
    pub fn list(&self) -> Vec<Example> {
        STORE.lock().unwrap().items.values().cloned().collect()
    }

    pub fn find(&self, id: u64) -> Result<Example, HttpError> {
        STORE
            .lock()
            .unwrap()
            .items
            .get(&id)
            .cloned()
            .ok_or_else(|| {
                HttpError::not_found("EXAMPLE_NOT_FOUND", format!("Item {id} not found"))
            })
    }

    pub fn create(&self, dto: CreateExampleDto) -> Example {
        let mut store = STORE.lock().unwrap();
        let id = store.next_id;
        store.next_id += 1;
        let item = Example {
            id,
            name: dto.name,
            description: dto.description.unwrap_or_default(),
        };
        store.items.insert(id, item.clone());
        item
    }

    pub fn update(&self, id: u64, dto: UpdateExampleDto) -> Result<Example, HttpError> {
        let mut store = STORE.lock().unwrap();
        let item = store.items.get_mut(&id).ok_or_else(|| {
            HttpError::not_found("EXAMPLE_NOT_FOUND", format!("Item {id} not found"))
        })?;
        if let Some(name) = dto.name {
            item.name = name;
        }
        if let Some(desc) = dto.description {
            item.description = desc;
        }
        Ok(item.clone())
    }

    pub fn delete(&self, id: u64) -> Result<(), HttpError> {
        STORE
            .lock()
            .unwrap()
            .items
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| {
                HttpError::not_found("EXAMPLE_NOT_FOUND", format!("Item {id} not found"))
            })
    }
}
