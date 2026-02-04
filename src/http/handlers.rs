use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use url::Url;
use uuid::Uuid;

use crate::domain::{LinkResponse, Ttl};
use crate::error::{AppError, AppResult};
use crate::http::router::AppState;

/// Request body for creating a new link.
#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    #[serde(default)]
    pub ttl: Option<Ttl>,
}

/// Request body for generating a QR code from a raw URL.
#[derive(Debug, Deserialize)]
pub struct CreateQrRequest {
    pub url: String,
}

/// Response for creating a new link.
#[derive(Debug, Serialize)]
pub struct CreateLinkResponse {
    #[serde(flatten)]
    pub link: LinkResponse,
}

/// Handler for creating a new short link.
/// POST /api/v1/links
pub async fn create_link(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CreateLinkRequest>,
) -> AppResult<(StatusCode, Json<CreateLinkResponse>)> {
    // Rate limiting by IP
    if let Err(retry_after) = state.rate_limiter.check(addr.ip()).await {
        return Err(AppError::RateLimitExceeded(retry_after));
    }

    let link = state.link_service.create_link(&req.url, req.ttl).await?;

    Ok((StatusCode::CREATED, Json(CreateLinkResponse { link })))
}

/// Handler for generating a QR code from a raw URL (no DB, no shortening).
/// POST /api/v1/qr
pub async fn create_qr(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CreateQrRequest>,
) -> AppResult<Response> {
    // Rate limiting by IP
    if let Err(retry_after) = state.rate_limiter.check(addr.ip()).await {
        return Err(AppError::RateLimitExceeded(retry_after));
    }

    let url =
        Url::parse(&req.url).map_err(|e| AppError::InvalidUrl(format!("{}: {}", e, req.url)))?;

    let png_data = state.qr_service.generate_for_url(url.as_str())?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png")],
        png_data,
    )
        .into_response())
}

/// Handler for listing all links.
/// GET /api/v1/links
/// Requires admin secret if configured.
pub async fn list_links(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<LinkResponse>>> {
    if let Some(required_secret) = &state.admin_secret {
        let provided = headers
            .get("x-admin-secret")
            .and_then(|value| value.to_str().ok());

        if provided != Some(required_secret.as_str()) {
            return Err(AppError::AdminRightsRequired);
        }
    }

    let links = state.link_service.list_all().await?;
    Ok(Json(links))
}

/// Handler for deleting a link.
/// DELETE /api/v1/links/:id
/// Requires admin secret if configured.
pub async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    if let Some(required_secret) = &state.admin_secret {
        let provided = headers
            .get("x-admin-secret")
            .and_then(|value| value.to_str().ok());

        if provided != Some(required_secret.as_str()) {
            return Err(AppError::AdminRightsRequired);
        }
    }

    state.link_service.delete_link(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Handler for redirecting to a short link.
/// GET /:short_code
pub async fn redirect(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
) -> Result<Response, AppError> {
    let link = state.link_service.resolve(&short_code).await?;
    Ok(Redirect::temporary(link.target_url.as_str()).into_response())
}

/// Health check endpoint.
/// GET /health
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "qrlink"
    }))
}
