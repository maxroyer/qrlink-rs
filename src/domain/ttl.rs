use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Time-to-live presets for links.
/// Only fixed presets are supported to keep the system predictable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ttl {
    /// Link expires in 1 week
    #[serde(rename = "1_week")]
    OneWeek,
    /// Link expires in 1 month (30 days)
    #[serde(rename = "1_month")]
    OneMonth,
    /// Link expires in 1 year (365 days)
    #[serde(rename = "1_year")]
    OneYear,
    /// Link never expires
    Never,
}

impl Ttl {
    /// Calculate the expiration datetime from the given starting point.
    /// Returns None for Never variant (no expiration).
    pub fn expires_at(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Ttl::OneWeek => Some(now + Duration::weeks(1)),
            Ttl::OneMonth => Some(now + Duration::days(30)),
            Ttl::OneYear => Some(now + Duration::days(365)),
            Ttl::Never => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_one_week() {
        let now = Utc::now();
        let expires = Ttl::OneWeek.expires_at(now).unwrap();
        assert_eq!((expires - now).num_days(), 7);
    }

    #[test]
    fn test_ttl_one_month() {
        let now = Utc::now();
        let expires = Ttl::OneMonth.expires_at(now).unwrap();
        assert_eq!((expires - now).num_days(), 30);
    }

    #[test]
    fn test_ttl_one_year() {
        let now = Utc::now();
        let expires = Ttl::OneYear.expires_at(now).unwrap();
        assert_eq!((expires - now).num_days(), 365);
    }
}
