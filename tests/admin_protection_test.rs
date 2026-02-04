use qrlink::{
    domain::Ttl,
    repository::{LinkRepository, init_db},
    service::LinkService,
};
use uuid::Uuid;

/// Test that list and delete functions work at the service level
/// (Note: admin protection is enforced at handler level, not service level)

#[tokio::test]
async fn test_list_all_links_service_level() {
    let pool = init_db("sqlite::memory:").await.unwrap();
    let repo = LinkRepository::new(pool);
    let service = LinkService::new(repo, "http://test.local".to_string());

    // Create test links
    service
        .create_link("https://example1.com", Some(Ttl::OneWeek))
        .await
        .unwrap();
    service
        .create_link("https://example2.com", Some(Ttl::OneMonth))
        .await
        .unwrap();

    // List should work at service level
    let links = service.list_all().await.unwrap();
    assert_eq!(links.len(), 2);
}

#[tokio::test]
async fn test_delete_link_service_level() {
    let pool = init_db("sqlite::memory:").await.unwrap();
    let repo = LinkRepository::new(pool);
    let service = LinkService::new(repo, "http://test.local".to_string());

    // Create a test link
    let link = service
        .create_link("https://example.com", Some(Ttl::OneWeek))
        .await
        .unwrap();
    let link_id = link.id;

    // Delete should work at service level
    service.delete_link(link_id).await.unwrap();

    // Link should no longer be in list
    let links = service.list_all().await.unwrap();
    assert_eq!(links.len(), 0);
}

#[tokio::test]
async fn test_delete_nonexistent_link_service_level() {
    let pool = init_db("sqlite::memory:").await.unwrap();
    let repo = LinkRepository::new(pool);
    let service = LinkService::new(repo, "http://test.local".to_string());

    // Try to delete a link that doesn't exist
    let fake_id = Uuid::new_v4();
    let result = service.delete_link(fake_id).await;

    // Should return LinkNotFound error
    assert!(result.is_err());
    match result {
        Err(e) => assert!(e.to_string().contains("Link not found")),
        Ok(_) => panic!("Expected error but got Ok"),
    }
}
