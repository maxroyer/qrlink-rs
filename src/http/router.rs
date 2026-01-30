use axum::{
    Router,
    extract::connect_info::IntoMakeServiceWithConnectInfo,
    routing::{delete, get, post},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::rate_limit::RateLimiter;
use crate::service::{LinkService, QrService};

use super::handlers;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub link_service: LinkService,
    pub qr_service: QrService,
    pub rate_limiter: RateLimiter,
}

/// Create the main application router.
pub fn create_router(
    link_service: LinkService,
    qr_service: QrService,
    rate_limiter: RateLimiter,
) -> IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr> {
    let state = AppState {
        link_service,
        qr_service,
        rate_limiter,
    };

    // API routes (public, no authentication)
    let api_routes = Router::new()
        .route("/links", post(handlers::create_link))
        .route("/links", get(handlers::list_links))
        .route("/links/{id}", delete(handlers::delete_link))
        .route("/qr", post(handlers::create_qr));

    // Public routes
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/{short_code}", get(handlers::redirect))
        .route("/{short_code}/qr", get(handlers::get_qr_code));

    // Serve static files from public directory
    let static_service = ServeDir::new("public");

    // Combine all routes
    Router::new()
        .nest("/api/v1", api_routes)
        .merge(public_routes)
        .route_service("/", ServeFile::new("public/index.html"))
        .route_service("/app.js", ServeFile::new("public/app.js"))
        .route_service("/styles.css", ServeFile::new("public/styles.css"))
        .fallback_service(static_service)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .into_make_service_with_connect_info::<std::net::SocketAddr>()
}
