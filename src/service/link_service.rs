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
    pub async fn create_link(
        &self,
        target_url: &str,
        ttl: Option<Ttl>,
    ) -> AppResult<LinkResponse> {
        // Validate URL
        let url =
            Url::parse(target_url).map_err(|e| AppError::InvalidUrl(format!("{}: {}", e, target_url)))?;

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
}
