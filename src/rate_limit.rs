use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A simple in-memory rate limiter using a fixed window algorithm.
/// Limits requests per IP address to prevent abuse.
#[derive(Clone)]
pub struct RateLimiter {
    /// Maximum requests per window
    limit: u32,
    /// Window duration
    window: Duration,
    /// IP -> (count, window_start)
    state: Arc<RwLock<HashMap<IpAddr, (u32, Instant)>>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(limit_per_minute: u32) -> Self {
        Self {
            limit: limit_per_minute,
            window: Duration::from_secs(60),
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request is allowed for the given IP address.
    /// Returns Ok(remaining) if allowed, Err(retry_after_secs) if rate limited.
    pub async fn check(&self, ip: IpAddr) -> Result<u32, u64> {
        let now = Instant::now();
        let mut state = self.state.write().await;

        let entry = state.entry(ip).or_insert((0, now));

        // Check if we need to reset the window
        if now.duration_since(entry.1) >= self.window {
            entry.0 = 0;
            entry.1 = now;
        }

        if entry.0 >= self.limit {
            let retry_after = self.window.as_secs()
                - now.duration_since(entry.1).as_secs();
            return Err(retry_after.max(1));
        }

        entry.0 += 1;
        let remaining = self.limit - entry.0;

        Ok(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limit_per_minute = 10;
        let limiter = RateLimiter::new(limit_per_minute);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        for i in 0..limit_per_minute {
            let result = limiter.check(ip).await;
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limit_per_minute = 5;
        let limiter = RateLimiter::new(limit_per_minute);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Use up the limit
        for _ in 0..limit_per_minute {
            limiter.check(ip).await.unwrap();
        }

        // Next request should be blocked
        let result = limiter.check(ip).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips_independent() {
        let limiter = RateLimiter::new(2);
        let ip1: IpAddr = "127.0.0.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.1".parse().unwrap();

        // Use up ip1's limit
        limiter.check(ip1).await.unwrap();
        limiter.check(ip1).await.unwrap();

        // ip1 should be blocked
        assert!(limiter.check(ip1).await.is_err());

        // ip2 should still work
        assert!(limiter.check(ip2).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_integration_60_per_minute() {
        let limit_per_minute = 60;
        let limiter = RateLimiter::new(limit_per_minute);
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // First 60 requests should succeed
        for i in 1..=limit_per_minute {
            let result = limiter.check(ip).await;
            assert!(result.is_ok(), "Request {} should succeed", i);
        }

        // 61st request should be rate limited
        let result = limiter.check(ip).await;
        assert!(result.is_err(), "Request 61 should be rate limited");
        
        // Verify retry_after is returned
        let retry_after = result.unwrap_err();
        assert!(retry_after > 0 && retry_after <= limit_per_minute as u64, 
                "retry_after should be between 1 and 60 seconds, got {}", retry_after);
    }
}
