use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    application::dto::{
        DiplomaShareLinkResponse, StudentDiplomaSearchRequest, StudentDiplomaSearchResponse,
        UserResponse,
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
