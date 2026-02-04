mod config;
mod domain;
mod error;
mod http;
mod qr;
mod rate_limit;
mod repository;
mod service;

use std::net::SocketAddr;
use std::time::Duration;

use config::Config;
use repository::{LinkRepository, init_db};
use service::{LinkService, QrService};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qrlink=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting QRLink service");
    tracing::info!("Base URL: {}", config.base_url);
    tracing::info!("Database: {}", config.database_url);

    // Initialize database
    let pool = init_db(&config.database_url).await?;
    tracing::info!("Database initialized");

    // Create repositories
    let link_repo = LinkRepository::new(pool.clone());

    // Create services
    let link_service = LinkService::new(link_repo, config.base_url.clone());
    let qr_service = QrService::new(&config)?;

    // Create rate limiter (IP-based, no authentication needed)
    let rate_limiter = rate_limit::RateLimiter::new(config.rate_limit_per_minute);

    // Optional admin secret
    let admin_secret = config.admin_secret.clone();

    // Create router
    let app = http::create_router(link_service.clone(), qr_service, rate_limiter, admin_secret);

    // Start cleanup task if enabled
    if config.cleanup_interval_minutes > 0 {
        let cleanup_service = link_service.clone();
        let interval_minutes = config.cleanup_interval_minutes;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60));
            loop {
                interval.tick().await;
                tracing::info!("Running cleanup of expired links");
                match cleanup_service.cleanup_expired().await {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("Cleaned up {} expired link(s)", count);
                        }
                    }
                    Err(e) => tracing::error!("Failed to cleanup expired links: {}", e),
                }
            }
        });
        tracing::info!("Cleanup task enabled (interval: {}m)", interval_minutes);
    } else {
        tracing::info!("Cleanup task disabled");
    }

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
