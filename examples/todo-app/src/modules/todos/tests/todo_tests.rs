use crate::modules::todos::dto::{CreateTodoDto, UpdateTodoDto};
use garde::Validate;

#[test]
fn create_todo_dto_validates_title() {
    let dto = CreateTodoDto {
        title: "".into(),
        description: None,
    };
    assert!(
        dto.validate().is_err(),
        "empty title should fail validation"
    );
}

#[test]
fn create_todo_dto_accepts_valid_input() {
    let dto = CreateTodoDto {
        title: "Buy groceries".into(),
        description: Some("Milk, eggs, bread".into()),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn create_todo_dto_rejects_overly_long_title() {
    let dto = CreateTodoDto {
        title: "x".repeat(501),
        description: None,
    };
    assert!(dto.validate().is_err());
}

#[test]
fn update_todo_dto_allows_partial_update() {
    let dto = UpdateTodoDto {
        title: None,
        description: Some("Updated description".into()),
        completed: None,
    };
    assert_eq!(dto.title, None);
    assert_eq!(dto.description.as_deref(), Some("Updated description"));
}

#[test]
fn update_todo_dto_allows_full_update() {
    let dto = UpdateTodoDto {
        title: Some("New title".into()),
        description: Some("New description".into()),
        completed: Some(true),
    };
    assert_eq!(dto.title.as_deref(), Some("New title"));
    assert_eq!(dto.completed, Some(true));
}
