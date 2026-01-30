use std::path::PathBuf;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database URL (SQLite path)
    pub database_url: String,
    /// Base URL for generating short links (e.g., "https://s.domain.local")
    pub base_url: String,
    /// HTTP server host
    pub host: String,
    /// HTTP server port
    pub port: u16,
    /// Rate limit per minute per token
    pub rate_limit_per_minute: u32,
    /// Optional path to branding logo for QR codes
    pub qr_branding_logo: Option<PathBuf>,
    /// QR code size in pixels
    pub qr_size: u32,
    /// Cleanup interval in minutes (0 to disable)
    pub cleanup_interval_minutes: u64,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:data/shortener.db".to_string());

        let base_url =
            std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidPort)?;

        let rate_limit_per_minute = std::env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidRateLimit)?;

        let qr_branding_logo = std::env::var("QR_BRANDING_LOGO")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let default_path = PathBuf::from("assets/logo.svg");
                if default_path.exists() {
                    Some(default_path)
                } else {
                    None
                }
            })
            .filter(|p| p.exists());

        let qr_size = std::env::var("QR_SIZE")
            .unwrap_or_else(|_| "512".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidQrSize)?;

        let cleanup_interval_minutes = std::env::var("CLEANUP_INTERVAL_MINUTES")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidCleanupInterval)?;

        Ok(Config {
            database_url,
            base_url,
            host,
            port,
            rate_limit_per_minute,
            qr_branding_logo,
            qr_size,
            cleanup_interval_minutes,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port number")]
    InvalidPort,
    #[error("Invalid rate limit value")]
    InvalidRateLimit,
    #[error("Invalid QR size value")]
    InvalidQrSize,
    #[error("Invalid cleanup interval value")]
    InvalidCleanupInterval,
}
