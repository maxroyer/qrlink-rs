mod config;
mod domain;
mod error;
mod http;
mod qr;
mod rate_limit;
mod repository;
mod service;

use std::net::SocketAddr;

use config::Config;
use repository::{init_db, LinkRepository};
use service::{LinkService, QrService};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "url_shortener=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting URL Shortener service");
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

    // Create router
    let app = http::create_router(link_service, qr_service, rate_limiter);

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

