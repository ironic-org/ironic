//! Unit tests — service and business logic in isolation (no HTTP).

use crate::modules::todos::{
    TodosService,
    dto::{CreateTodosDto, UpdateTodosDto},
    entities::Priority,
};

#[test]
fn service_starts_empty() {
    TodosService::reset();
    let svc = TodosService;
    assert_eq!(svc.list().len(), 0);
}

#[test]
fn create_and_find_todo() {
    TodosService::reset();
    let svc = TodosService;
    let dto = CreateTodosDto {
        title: "Test".into(),
        priority: Priority::High,
    };
    let todo = svc.create(dto);
    assert_eq!(todo.title, "Test");
    assert!(!todo.completed);

    let found = svc.find(todo.id).unwrap();
    assert_eq!(found.title, "Test");
}

#[test]
fn update_todo() {
    TodosService::reset();
    let svc = TodosService;
    let todo = svc.create(CreateTodosDto {
        title: "Old".into(),
        priority: Priority::Low,
    });

    let updated = svc
        .update(
            todo.id,
            UpdateTodosDto {
                title: Some("New".into()),
                completed: Some(true),
                priority: None,
            },
        )
        .unwrap();
    assert_eq!(updated.title, "New");
    assert!(updated.completed);
}

#[test]
fn delete_todo() {
    TodosService::reset();
    let svc = TodosService;
    let todo = svc.create(CreateTodosDto {
        title: "Delete me".into(),
        priority: Priority::Medium,
    });
    assert!(svc.delete(todo.id).is_ok());
    assert!(svc.find(todo.id).is_err());
}

#[test]
fn not_found_error() {
    TodosService::reset();
    let svc = TodosService;
    let err = svc.find(999).unwrap_err();
    assert_eq!(err.status(), ironic::HttpStatus::NOT_FOUND);
}
