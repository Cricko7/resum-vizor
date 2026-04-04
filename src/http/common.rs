use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{
    application::dto::PublicDiplomaView,
    error::AppError,
    http::AppState,
    infrastructure::metrics,
};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub checks: ReadinessChecks,
}

#[derive(Debug, Serialize)]
pub struct ReadinessChecks {
    pub database: &'static str,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "resume-vizor-backend",
    })
}

pub async fn liveness_check() -> Json<HealthResponse> {
    health_check().await
}

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

pub async fn public_diploma_access(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<PublicDiplomaView>, AppError> {
    let diploma = state.diploma_service.resolve_public_diploma_view(&token).await?;
    Ok(Json(diploma))
}

