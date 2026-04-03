use axum::{Json, extract::State};
use serde::Serialize;

use crate::{
    application::dto::{RegisterDiplomaRequest, RegisterDiplomaResponse},
    error::AppError,
    http::AppState,
};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "resume-visitor-backend",
    })
}

pub async fn register_diploma(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDiplomaRequest>,
) -> Result<Json<RegisterDiplomaResponse>, AppError> {
    let diploma = state.diploma_service.register_diploma(payload).await?;
    Ok(Json(diploma.into()))
}
