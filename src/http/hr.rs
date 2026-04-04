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
    error::{AppError, ErrorBody},
    http::{
        AppState,
        middleware::{AuthenticatedAtsClient, AuthenticatedAutomationClient, AuthenticatedUser},
    },
};

#[utoipa::path(
    post,
    path = "/api/v1/hr/verify",
    tag = "HR",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'hr'")
    ),
    request_body = VerifyDiplomaRequest,
    responses(
        (status = 200, description = "Diploma verification result", body = VerifyDiplomaResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody)
    )
)]
pub async fn verify_diploma(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(payload): Json<VerifyDiplomaRequest>,
) -> Result<Json<VerifyDiplomaResponse>, AppError> {
    let result = state.diploma_service.verify_diploma(payload).await?;
    Ok(Json(result.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/hr/registry/search",
    tag = "HR",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'hr'")
    ),
    request_body = HrRegistrySearchRequest,
    responses(
        (status = 200, description = "Registry search results", body = HrRegistrySearchResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody)
    )
)]
pub async fn search_registry(
    State(state): State<AppState>,
    _authenticated: AuthenticatedUser,
    Json(payload): Json<HrRegistrySearchRequest>,
) -> Result<Json<HrRegistrySearchResponse>, AppError> {
    let result = state.diploma_service.search_hr_registry(payload).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/hr/automation/verify",
    tag = "Integrations",
    security(("api_key_auth" = [])),
    request_body = HrRegistrySearchRequest,
    responses(
        (status = 200, description = "Automation registry verification result", body = HrRegistrySearchResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Invalid API key", body = ErrorBody),
        (status = 403, description = "Scope does not allow this endpoint", body = ErrorBody),
        (status = 429, description = "Rate limit exceeded", body = ErrorBody)
    )
)]
pub async fn automation_verify(
    State(state): State<AppState>,
    _authenticated: AuthenticatedAutomationClient,
    Json(payload): Json<HrRegistrySearchRequest>,
) -> Result<Json<HrRegistrySearchResponse>, AppError> {
    let result = state.diploma_service.search_hr_registry(payload).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/hr/api-keys",
    tag = "HR",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'hr'")
    ),
    request_body = CreateAtsApiKeyRequest,
    responses(
        (status = 200, description = "API key created", body = CreateAtsApiKeyResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody)
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/hr/api-keys",
    tag = "HR",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'hr'")
    ),
    responses(
        (status = 200, description = "Issued integration API keys", body = AtsApiKeyListResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody)
    )
)]
pub async fn list_ats_api_keys(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<AtsApiKeyListResponse>, AppError> {
    let result = state.ats_service.list_api_keys(authenticated.user_id).await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/hr/api-keys/{api_key_id}/revoke",
    tag = "HR",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'hr'"),
        ("api_key_id" = crate::domain::ids::AtsApiKeyId, Path, description = "API key identifier")
    ),
    responses(
        (status = 200, description = "API key revoked", body = AtsApiKeySummary),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "API key not found", body = ErrorBody)
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/ats/verify",
    tag = "Integrations",
    security(("api_key_auth" = [])),
    request_body = AtsVerifyRequest,
    responses(
        (status = 200, description = "ATS verification result", body = AtsVerifyResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Invalid API key", body = ErrorBody),
        (status = 403, description = "Scope does not allow this endpoint", body = ErrorBody),
        (status = 429, description = "Rate limit exceeded", body = ErrorBody)
    )
)]
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
