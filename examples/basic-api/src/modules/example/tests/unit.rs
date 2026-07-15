//! Unit tests for `ExampleService`.

use crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};
use crate::modules::example::services::ExampleService;

#[test]
fn create_and_find() {
    let svc = ExampleService;
    let item = svc.create(CreateExampleDto {
        name: "Test".into(),
        description: None,
    });
    assert_eq!(item.name, "Test");
    let found = svc.find(item.id).unwrap();
    assert_eq!(found.name, "Test");
}

#[test]
fn update_works() {
    let svc = ExampleService;
    let item = svc.create(CreateExampleDto {
        name: "Old".into(),
        description: None,
    });
    let updated = svc
        .update(
            item.id,
            UpdateExampleDto {
                name: Some("New".into()),
                description: None,
            },
        )
        .unwrap();
    assert_eq!(updated.name, "New");
}

#[test]
fn delete_works() {
    let svc = ExampleService;
    let item = svc.create(CreateExampleDto {
        name: "Del".into(),
        description: None,
    });
    assert!(svc.delete(item.id).is_ok());
    assert!(svc.find(item.id).is_err());
}

#[test]
fn not_found_error() {
    let svc = ExampleService;
    let err = svc.find(999).unwrap_err();
    assert_eq!(err.status(), ironic::HttpStatus::NOT_FOUND);
}

#[test]
fn list_works() {
    let svc = ExampleService;
    svc.create(CreateExampleDto {
        name: "A".into(),
        description: None,
    });
    svc.create(CreateExampleDto {
        name: "B".into(),
        description: None,
    });
    assert!(svc.list().len() >= 2);
}
