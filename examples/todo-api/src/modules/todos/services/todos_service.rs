use std::{collections::HashMap, sync::Mutex};

use ironic::prelude::*;

use crate::modules::todos::{
    dto::{CreateTodosDto, UpdateTodosDto},
    entities::Todo,
};

#[derive(Injectable)]
pub struct TodosService;

#[derive(Default)]
struct TodoStore {
    items: HashMap<u64, Todo>,
    next_id: u64,
}

static STORE: std::sync::LazyLock<Mutex<TodoStore>> =
    std::sync::LazyLock::new(|| Mutex::new(TodoStore::default()));

impl TodosService {
    fn store(&self) -> &Mutex<TodoStore> {
        &STORE
    }

    #[cfg(test)]
    pub fn reset() {
        if let Ok(mut s) = STORE.lock() {
            *s = TodoStore::default();
        }
    }

    pub fn list(&self) -> Vec<Todo> {
        self.store()
            .lock()
            .unwrap()
            .items
            .values()
            .cloned()
            .collect()
    }

    pub fn find(&self, id: u64) -> Result<Todo, HttpError> {
        self.store()
            .lock()
            .unwrap()
            .items
            .get(&id)
            .cloned()
            .ok_or_else(|| HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found")))
    }

    pub fn create(&self, dto: CreateTodosDto) -> Todo {
        let mut store = self.store().lock().unwrap();
        let id = store.next_id;
        store.next_id += 1;
        let todo = Todo {
            id,
            title: dto.title,
            completed: false,
            priority: dto.priority,
        };
        store.items.insert(id, todo.clone());
        todo
    }

    pub fn update(&self, id: u64, dto: UpdateTodosDto) -> Result<Todo, HttpError> {
        let mut store = self.store().lock().unwrap();
        let todo = store.items.get_mut(&id).ok_or_else(|| {
            HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found"))
        })?;
        if let Some(title) = dto.title {
            todo.title = title;
        }
        if let Some(completed) = dto.completed {
            todo.completed = completed;
        }
        if let Some(priority) = dto.priority {
            todo.priority = priority;
        }
        Ok(todo.clone())
    }

    pub fn delete(&self, id: u64) -> Result<(), HttpError> {
        self.store()
            .lock()
            .unwrap()
            .items
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found")))
    }
}
