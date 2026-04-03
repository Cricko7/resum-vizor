use axum::{Json, extract::State};

use crate::{
    application::dto::{
        HrRegistrySearchRequest, HrRegistrySearchResponse, VerifyDiplomaRequest,
        VerifyDiplomaResponse,
    },
    error::AppError,
    http::{AppState, middleware::AuthenticatedUser},
};

pub async fn verify_diploma(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(payload): Json<VerifyDiplomaRequest>,
) -> Result<Json<VerifyDiplomaResponse>, AppError> {
    let result = state.diploma_service.verify_diploma(payload).await?;
    Ok(Json(result.into()))
}

pub async fn search_registry(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(payload): Json<HrRegistrySearchRequest>,
) -> Result<Json<HrRegistrySearchResponse>, AppError> {
    let result = state.diploma_service.search_hr_registry(payload).await?;
    Ok(Json(result))
}

pub async fn automation_verify(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(payload): Json<HrRegistrySearchRequest>,
) -> Result<Json<HrRegistrySearchResponse>, AppError> {
    let result = state.diploma_service.search_hr_registry(payload).await?;
    Ok(Json(result))
}
