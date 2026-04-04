use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    application::dto::{
        AtsApiKeyListResponse, AtsApiKeySummary, AtsVerifyRequest, AtsVerifyResponse,
        CreateAtsApiKeyRequest, CreateAtsApiKeyResponse, HrRegistrySearchRequest,
        HrRegistrySearchResponse, VerifyDiplomaRequest, VerifyDiplomaResponse,
    },
    error::AppError,
    http::{
        AppState,
        middleware::{AuthenticatedAtsClient, AuthenticatedAutomationClient, AuthenticatedUser},
    },
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
    _authenticated: AuthenticatedAutomationClient,
    Json(payload): Json<HrRegistrySearchRequest>,
) -> Result<Json<HrRegistrySearchResponse>, AppError> {
    let result = state.diploma_service.search_hr_registry(payload).await?;
    Ok(Json(result))
}

pub async fn create_ats_api_key(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(payload): Json<CreateAtsApiKeyRequest>,
) -> Result<Json<CreateAtsApiKeyResponse>, AppError> {
    let result = state
        .ats_service
        .create_api_key(authenticated.user_id, payload)
        .await?;
    Ok(Json(result))
}

pub async fn list_ats_api_keys(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<AtsApiKeyListResponse>, AppError> {
    let result = state.ats_service.list_api_keys(authenticated.user_id).await?;
    Ok(Json(result))
}

pub async fn revoke_ats_api_key(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(api_key_id): Path<crate::domain::ids::AtsApiKeyId>,
) -> Result<Json<AtsApiKeySummary>, AppError> {
    let result = state
        .ats_service
        .revoke_api_key(authenticated.user_id, api_key_id)
        .await?;
    Ok(Json(result))
}

pub async fn ats_verify(
    State(state): State<AppState>,
    authenticated: AuthenticatedAtsClient,
    Json(payload): Json<AtsVerifyRequest>,
) -> Result<Json<AtsVerifyResponse>, AppError> {
    let _ = authenticated.api_key_id;
    let _ = authenticated.hr_user_id;
    let result = state
        .ats_service
        .verify_for_ats(&authenticated.integration_name, payload, &state.diploma_service)
        .await?;
    Ok(Json(result))
}
