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
    error::AppError,
    http::{AppState, middleware::AuthenticatedUser},
};

pub async fn profile(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    Ok(Json(user.into()))
}

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
