use axum::{Json, extract::State};

use crate::{
    application::dto::{
        AuthResponse, ChangePasswordRequest, LoginRequest, RegisterUserRequest, UserResponse,
    },
    error::AppError,
    http::{AppState, middleware::AuthenticatedUser},
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.register(payload).await?;
    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.login(payload).await?;
    Ok(Json(response))
}

pub async fn change_password(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<(), AppError> {
    state
        .auth_service
        .change_password(authenticated.user_id, payload)
        .await
}

pub async fn me(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    Ok(Json(user.into()))
}
