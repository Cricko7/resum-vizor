use axum::{Json, extract::State};

use crate::{
    application::dto::{
        AuthResponse, ChangePasswordRequest, LoginRequest, RegisterUserRequest, UserResponse,
    },
    error::{AppError, ErrorBody},
    http::{AppState, middleware::AuthenticatedUser},
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterUserRequest,
    responses(
        (status = 200, description = "User registered and authenticated", body = AuthResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 409, description = "User already exists", body = ErrorBody)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.register(payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User authenticated", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorBody)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = state.auth_service.login(payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    tag = "Auth",
    security(("bearer_auth" = [])),
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed"),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody)
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "Auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current authenticated user", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody)
    )
)]
pub async fn me(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    Ok(Json(user.into()))
}
