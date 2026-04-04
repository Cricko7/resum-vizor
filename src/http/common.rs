use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    application::dto::PublicDiplomaView,
    error::{AppError, ErrorBody},
    http::AppState,
    infrastructure::metrics,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub checks: ReadinessChecks,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessChecks {
    pub database: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Operations",
    responses((status = 200, description = "Service is alive", body = HealthResponse))
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "resume-vizor-backend",
    })
}

#[utoipa::path(
    get,
    path = "/health/live",
    tag = "Operations",
    responses((status = 200, description = "Liveness probe", body = HealthResponse))
)]
pub async fn liveness_check() -> Json<HealthResponse> {
    health_check().await
}

#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "Operations",
    responses(
        (status = 200, description = "Readiness probe passed", body = ReadinessResponse),
        (status = 503, description = "Readiness probe failed", body = ReadinessResponse)
    )
)]
pub async fn readiness_check(State(state): State<AppState>) -> Response {
    let is_ready = state.health_checker.is_ready().await;

    let body = Json(ReadinessResponse {
        status: if is_ready { "ready" } else { "not_ready" },
        service: "resume-vizor-backend",
        checks: ReadinessChecks {
            database: if is_ready { "up" } else { "down" },
        },
    });

    if is_ready {
        (StatusCode::OK, body).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, body).into_response()
    }
}

pub async fn metrics_handler() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );

    (StatusCode::OK, headers, metrics::render_metrics()).into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/public/diplomas/access/{token}",
    tag = "Public",
    params(
        ("token" = String, Path, description = "Temporary diploma access token")
    ),
    responses(
        (status = 200, description = "Public diploma view", body = PublicDiplomaView),
        (status = 401, description = "Invalid or expired access token", body = ErrorBody),
        (status = 404, description = "Diploma not found", body = ErrorBody)
    )
)]
pub async fn public_diploma_access(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<PublicDiplomaView>, AppError> {
    let diploma = state.diploma_service.resolve_public_diploma_view(&token).await?;
    Ok(Json(diploma))
}
