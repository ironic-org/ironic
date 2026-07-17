use crate::modules::blogs::dto::CreateBlogDto;
use crate::modules::blogs::repositories::{BlogRepository, CategoryRepository};
use crate::modules::blogs::services::BlogService;
use std::sync::Arc;

fn make_service() -> BlogService {
    BlogService {
        blog_repo: Arc::new(BlogRepository),
        category_repo: Arc::new(CategoryRepository),
    }
}

#[tokio::test]
async fn test_create_and_find() {
    let svc = make_service();
    let dto = CreateBlogDto {
        title: "Hello World".into(),
        content: "This is my first post".into(),
        excerpt: Some("A greeting".into()),
        tags: Some(vec!["hello".into(), "first".into()]),
        author: Some("Alice".into()),
        publish: Some(true),
        category_ids: None,
    };

    let post = svc.create(dto).expect("create should succeed");
    assert_eq!(post.title, "Hello World");
    assert_eq!(post.slug, "hello-world");
    assert!(post.published);
    assert_eq!(post.author, "Alice");

    let found = svc.find(post.id).expect("find should succeed");
    assert_eq!(found.id, post.id);
}

#[tokio::test]
async fn test_find_by_slug() {
    let svc = make_service();
    svc.create(CreateBlogDto {
        title: "My Rust Journey".into(),
        content: "Learning Rust has been amazing".into(),
        excerpt: None,
        tags: None,
        author: None,
        publish: Some(false),
        category_ids: None,
    })
    .expect("create should succeed");

    let post = svc
        .find_by_slug("my-rust-journey")
        .expect("find_by_slug should succeed");
    assert_eq!(post.title, "My Rust Journey");
}

#[tokio::test]
async fn test_update_post() {
    let svc = make_service();
    let post = svc
        .create(CreateBlogDto {
            title: "Original Title".into(),
            content: "Original content".into(),
            excerpt: None,
            tags: None,
            author: None,
            publish: Some(false),
            category_ids: None,
        })
        .expect("create should succeed");

    let updated = svc
        .update(
            post.id,
            crate::modules::blogs::dto::UpdateBlogDto {
                title: Some("Updated Title".into()),
                content: None,
                excerpt: Some(Some("New excerpt".into())),
                tags: Some(vec!["updated".into()]),
                published: Some(true),
                category_ids: None,
            },
        )
        .expect("update should succeed");

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.excerpt, Some("New excerpt".into()));
    assert!(updated.published);
}

#[tokio::test]
async fn test_publish_unpublish() {
    let svc = make_service();
    let post = svc
        .create(CreateBlogDto {
            title: "Draft Post".into(),
            content: "Waiting to be published".into(),
            excerpt: None,
            tags: None,
            author: None,
            publish: Some(false),
            category_ids: None,
        })
        .expect("create should succeed");

    assert!(!post.published);

    let published = svc.publish(post.id).expect("publish should succeed");
    assert!(published.published);

    let unpublished = svc.unpublish(post.id).expect("unpublish should succeed");
    assert!(!unpublished.published);
}

#[tokio::test]
async fn test_delete_post() {
    let svc = make_service();
    let post = svc
        .create(CreateBlogDto {
            title: "To Delete".into(),
            content: "This will be deleted".into(),
            excerpt: None,
            tags: None,
            author: None,
            publish: Some(false),
            category_ids: None,
        })
        .expect("create should succeed");

    svc.delete(post.id).expect("delete should succeed");
    assert!(svc.find(post.id).is_err());
}

#[tokio::test]
async fn test_slug_conflict() {
    let svc = make_service();
    svc.create(CreateBlogDto {
        title: "Same Title".into(),
        content: "First post".into(),
        excerpt: None,
        tags: None,
        author: None,
        publish: Some(false),
        category_ids: None,
    })
    .expect("first create should succeed");

    let result = svc.create(CreateBlogDto {
        title: "Same Title".into(),
        content: "Second post".into(),
        excerpt: None,
        tags: None,
        author: None,
        publish: Some(false),
        category_ids: None,
    });
    assert!(result.is_err());
}

#[tokio::test]
async fn test_category_crud() {
    let svc = make_service();
    let cat = svc
        .create_category("Technology".into(), Some("Tech related posts".into()))
        .expect("create category should succeed");
    assert_eq!(cat.slug, "technology");

    let cats = svc.categories().expect("list categories should succeed");
    assert!(cats.iter().any(|c| c.id == cat.id));
}

#[tokio::test]
async fn test_assign_category_to_post() {
    let svc = make_service();
    let cat = svc
        .create_category("News".into(), None)
        .expect("create category should succeed");

    let post = svc
        .create(CreateBlogDto {
            title: "News Post".into(),
            content: "Some news".into(),
            excerpt: None,
            tags: None,
            author: None,
            publish: Some(true),
            category_ids: None,
        })
        .expect("create post should succeed");

    let updated = svc
        .add_category(post.id, cat.id)
        .expect("add category should succeed");
    assert!(updated.category_ids.contains(&cat.id));

    let removed = svc
        .remove_category(post.id, cat.id)
        .expect("remove category should succeed");
    assert!(!removed.category_ids.contains(&cat.id));
}

#[tokio::test]
async fn test_stats() {
    let svc = make_service();
    let stats = svc.stats().expect("stats should succeed");
    assert_eq!(stats.total, 0);
}
