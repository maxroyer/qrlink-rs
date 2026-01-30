use chrono::Utc;
use url::Url;
use uuid::Uuid;

use crate::domain::{Link, LinkResponse, ShortCode, Ttl};
use crate::error::{AppError, AppResult};
use crate::repository::LinkRepository;

/// Maximum number of retries when generating a short code.
const MAX_RETRIES: usize = 5;

/// Service for link-related business operations.
#[derive(Clone)]
pub struct LinkService {
    repo: LinkRepository,
    base_url: String,
}

impl LinkService {
    pub fn new(repo: LinkRepository, base_url: String) -> Self {
        Self { repo, base_url }
    }

    /// Create a new short link.
    pub async fn create_link(&self, target_url: &str, ttl: Option<Ttl>) -> AppResult<LinkResponse> {
        // Validate URL
        let url = Url::parse(target_url)
            .map_err(|e| AppError::InvalidUrl(format!("{}: {}", e, target_url)))?;

        let now = Utc::now();
        let expires_at = ttl.and_then(|t| t.expires_at(now));

        // Try to create with collision retry
        for _ in 0..MAX_RETRIES {
            let id = Uuid::new_v4();
            let short_code = ShortCode::generate();

            match self
                .repo
                .create(id, &short_code, &url, now, expires_at)
                .await
            {
                Ok(link) => return Ok(LinkResponse::from_link(&link, &self.base_url)),
                Err(AppError::ShortCodeExhausted) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(AppError::ShortCodeExhausted)
    }

    /// Resolve a short code to a link for redirection.
    pub async fn resolve(&self, short_code: &str) -> AppResult<Link> {
        let link = self
            .repo
            .find_by_short_code(short_code)
            .await?
            .ok_or(AppError::LinkNotFound)?;

        if link.is_expired(Utc::now()) {
            return Err(AppError::LinkExpired);
        }

        Ok(link)
    }

    /// List all links (no authentication required).
    pub async fn list_all(&self) -> AppResult<Vec<LinkResponse>> {
        let links = self.repo.list_all().await?;
        Ok(links
            .iter()
            .map(|l| LinkResponse::from_link(l, &self.base_url))
            .collect())
    }

    /// Delete a link by ID.
    pub async fn delete_link(&self, link_id: Uuid) -> AppResult<()> {
        let deleted = self.repo.delete(link_id).await?;
        if !deleted {
            return Err(AppError::LinkNotFound);
        }
        Ok(())
    }

    /// Clean up expired links (for periodic job).
    pub async fn cleanup_expired(&self) -> AppResult<u64> {
        self.repo.delete_expired().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::init_db;
    use chrono::Duration;

    #[tokio::test]
    async fn test_cleanup_expired_links() {
        // Setup in-memory database
        let pool = init_db("sqlite::memory:").await.unwrap();
        let repo = LinkRepository::new(pool);
        let service = LinkService::new(repo.clone(), "http://test.local".to_string());

        let now = Utc::now();

        // Create link that expired 1 hour ago (manually via repo)
        let expired_id = Uuid::new_v4();
        let expired_code = ShortCode::generate();
        let expired_url = Url::parse("https://expired.com").unwrap();
        let expired_at = now - Duration::hours(1);
        repo.create(
            expired_id,
            &expired_code,
            &expired_url,
            now,
            Some(expired_at),
        )
        .await
        .unwrap();

        // Create link that expires in 1 week (via service)
        let valid_link = service
            .create_link("https://valid.com", Some(Ttl::OneWeek))
            .await
            .unwrap();

        // Create link with no expiration (via service)
        let permanent_link = service
            .create_link("https://permanent.com", None)
            .await
            .unwrap();

        // Verify all links exist
        let all_links_before = service.list_all().await.unwrap();
        assert_eq!(all_links_before.len(), 3);

        // Run cleanup
        let deleted_count = service.cleanup_expired().await.unwrap();

        // Should have deleted only the expired link
        assert_eq!(deleted_count, 1);

        // Verify only 2 links remain
        let all_links_after = service.list_all().await.unwrap();
        assert_eq!(all_links_after.len(), 2);

        // Verify the right links remain
        let remaining_codes: Vec<String> = all_links_after
            .iter()
            .map(|l| l.short_code.clone())
            .collect();

        assert!(remaining_codes.contains(&valid_link.short_code));
        assert!(remaining_codes.contains(&permanent_link.short_code));
        assert!(!remaining_codes.contains(&expired_code.to_string()));
    }

    #[tokio::test]
    async fn test_cleanup_no_expired_links() {
        // Setup in-memory database
        let pool = init_db("sqlite::memory:").await.unwrap();
        let repo = LinkRepository::new(pool);
        let service = LinkService::new(repo, "http://test.local".to_string());

        // Create only valid links
        service
            .create_link("https://valid1.com", Some(Ttl::OneWeek))
            .await
            .unwrap();

        service
            .create_link("https://valid2.com", None)
            .await
            .unwrap();

        // Run cleanup
        let deleted_count = service.cleanup_expired().await.unwrap();

        // Should not have deleted anything
        assert_eq!(deleted_count, 0);

        // Verify both links still exist
        let all_links = service.list_all().await.unwrap();
        assert_eq!(all_links.len(), 2);
    }
}
