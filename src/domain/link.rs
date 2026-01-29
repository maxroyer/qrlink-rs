use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::ShortCode;

/// A shortened link.
#[derive(Debug, Clone)]
pub struct Link {
    /// Unique identifier
    pub id: Uuid,
    /// The short code used in the URL
    pub short_code: ShortCode,
    /// The target URL to redirect to
    pub target_url: Url,
    /// When the link was created
    pub created_at: DateTime<Utc>,
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

impl Link {
    /// Check if this link has expired.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        match self.expires_at {
            Some(exp) => now >= exp,
            None => false,
        }
    }
}

/// Response DTO for a link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResponse {
    pub id: Uuid,
    pub short_code: String,
    pub short_url: String,
    pub target_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl LinkResponse {
    pub fn from_link(link: &Link, base_url: &str) -> Self {
        LinkResponse {
            id: link.id,
            short_code: link.short_code.as_str().to_string(),
            short_url: format!("{}/{}", base_url.trim_end_matches('/'), link.short_code),
            target_url: link.target_url.to_string(),
            created_at: link.created_at,
            expires_at: link.expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_link(expires_at: Option<DateTime<Utc>>) -> Link {
        Link {
            id: Uuid::new_v4(),
            short_code: ShortCode::from_existing("Ab3kP9x".to_string()),
            target_url: Url::parse("https://example.com").unwrap(),
            created_at: Utc::now(),
            expires_at,
        }
    }

    #[test]
    fn test_link_not_expired_when_no_expiry() {
        let link = create_test_link(None);
        assert!(!link.is_expired(Utc::now()));
    }

    #[test]
    fn test_link_not_expired_before_expiry() {
        let future = Utc::now() + chrono::Duration::hours(1);
        let link = create_test_link(Some(future));
        assert!(!link.is_expired(Utc::now()));
    }

    #[test]
    fn test_link_expired_at_expiry() {
        let now = Utc::now();
        let link = create_test_link(Some(now));
        assert!(link.is_expired(now));
    }

    #[test]
    fn test_link_expired_after_expiry() {
        let past = Utc::now() - chrono::Duration::hours(1);
        let link = create_test_link(Some(past));
        assert!(link.is_expired(Utc::now()));
    }
}
