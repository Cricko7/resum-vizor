use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{
    application::dto::{
        CreateDiplomaQrRequest, DeleteDiplomaQrResponse, DiplomaQrResponse,
        DiplomaShareLinkResponse, StudentDiplomaSearchRequest, StudentDiplomaSearchResponse, UserResponse,
    },
    domain::ids::DiplomaId,
    error::{AppError, ErrorBody},
    http::{AppState, middleware::AuthenticatedUser},
};

#[utoipa::path(
    get,
    path = "/api/v1/student/profile",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'")
    ),
    responses(
        (status = 200, description = "Student profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody)
    )
)]
pub async fn profile(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    Ok(Json(user.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/student/search",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'")
    ),
    request_body = StudentDiplomaSearchRequest,
    responses(
        (status = 200, description = "Student diploma search results", body = StudentDiplomaSearchResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody)
    )
)]
pub async fn search_my_diplomas(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(payload): Json<StudentDiplomaSearchRequest>,
) -> Result<Json<StudentDiplomaSearchResponse>, AppError> {
    let result = state
        .diploma_service
        .search_student_diplomas(authenticated.user_id, payload)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/student/diplomas/{diploma_id}/share-link",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'"),
        ("diploma_id" = DiplomaId, Path, description = "Diploma identifier")
    ),
    responses(
        (status = 200, description = "Temporary diploma share link generated", body = DiplomaShareLinkResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Diploma not found", body = ErrorBody)
    )
)]
pub async fn generate_share_link(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
) -> Result<Json<DiplomaShareLinkResponse>, AppError> {
    let result = state
        .diploma_service
        .generate_diploma_share_link(
            authenticated.user_id,
            diploma_id,
            &state.settings.server.base_url,
            state.settings.security.diploma_link_ttl_minutes,
        )
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/student/diplomas/{diploma_id}/qr",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'"),
        ("diploma_id" = DiplomaId, Path, description = "Diploma identifier")
    ),
    request_body = CreateDiplomaQrRequest,
    responses(
        (status = 200, description = "QR created or reused", body = DiplomaQrResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Diploma not found", body = ErrorBody),
        (status = 502, description = "QR service error", body = ErrorBody),
        (status = 503, description = "QR service unavailable", body = ErrorBody)
    )
)]
pub async fn create_or_get_qr(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
    Json(payload): Json<CreateDiplomaQrRequest>,
) -> Result<Json<DiplomaQrResponse>, AppError> {
    let result = state
        .qr_service
        .create_or_get_diploma_qr(authenticated.user_id, diploma_id, payload)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/v1/student/diplomas/{diploma_id}/qr",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'"),
        ("diploma_id" = DiplomaId, Path, description = "Diploma identifier")
    ),
    responses(
        (status = 200, description = "QR metadata", body = DiplomaQrResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "QR not found", body = ErrorBody),
        (status = 502, description = "QR service error", body = ErrorBody),
        (status = 503, description = "QR service unavailable", body = ErrorBody)
    )
)]
pub async fn get_qr(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
) -> Result<Json<DiplomaQrResponse>, AppError> {
    let result = state
        .qr_service
        .get_diploma_qr(authenticated.user_id, diploma_id)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/v1/student/diplomas/{diploma_id}/qr",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'"),
        ("diploma_id" = DiplomaId, Path, description = "Diploma identifier")
    ),
    responses(
        (status = 200, description = "QR deleted", body = DeleteDiplomaQrResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "Diploma not found", body = ErrorBody)
    )
)]
pub async fn delete_qr(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
) -> Result<Json<DeleteDiplomaQrResponse>, AppError> {
    let result = state
        .qr_service
        .delete_diploma_qr(authenticated.user_id, diploma_id)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/v1/student/diplomas/{diploma_id}/qr/content",
    tag = "Student",
    security(("bearer_auth" = [])),
    params(
        ("role" = String, Header, description = "Must be 'student'"),
        ("diploma_id" = DiplomaId, Path, description = "Diploma identifier")
    ),
    responses(
        (status = 200, description = "QR image content"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 403, description = "Forbidden", body = ErrorBody),
        (status = 404, description = "QR not found", body = ErrorBody),
        (status = 409, description = "QR is not ready", body = ErrorBody),
        (status = 502, description = "QR service error", body = ErrorBody),
        (status = 503, description = "QR service unavailable", body = ErrorBody)
    )
)]
pub async fn get_qr_content(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
) -> Result<Response, AppError> {
    let result = state
        .qr_service
        .get_diploma_qr_content(authenticated.user_id, diploma_id)
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&result.content_type).map_err(|_| AppError::Internal)?,
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&result.bytes.len().to_string()).map_err(|_| AppError::Internal)?,
    );

    Ok((StatusCode::OK, headers, result.bytes).into_response())
}
