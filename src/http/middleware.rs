use axum::{
    extract::{FromRequestParts, State},
    http::{HeaderMap, Request, header},
    middleware::Next,
    response::Response,
};

use crate::{
    domain::{ids::UserId, user::UserRole},
    error::AppError,
    http::AppState,
};
use axum::extract::FromRef;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub role: UserRole,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        decode_authenticated(&parts.headers, &state)
    }
}

pub async fn require_university_role(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    require_role(state, request, next, UserRole::University).await
}

pub async fn require_student_role(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    require_role(state, request, next, UserRole::Student).await
}

pub async fn require_hr_role(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    require_role(state, request, next, UserRole::Hr).await
}

pub async fn enforce_hr_automation_rate_limit(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let key = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(',').next().unwrap_or(value).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "global".to_string());

    if !state.hr_rate_limiter.allow(&key).await {
        return Err(AppError::RateLimited);
    }

    Ok(next.run(request).await)
}

async fn require_role(
    state: AppState,
    request: Request<axum::body::Body>,
    next: Next,
    expected_role: UserRole,
) -> Result<Response, AppError> {
    let authenticated = decode_authenticated(request.headers(), &state)?;
    let role_header = parse_role_header(request.headers())?;

    if role_header != expected_role {
        return Err(AppError::Forbidden(format!(
            "role header must be '{}'",
            expected_role
        )));
    }

    if authenticated.role != expected_role {
        return Err(AppError::Forbidden(
            "jwt role does not match route role".into(),
        ));
    }

    Ok(next.run(request).await)
}

fn decode_authenticated(headers: &HeaderMap, state: &AppState) -> Result<AuthenticatedUser, AppError> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;
    let claims = state.jwt_provider.decode_token(token)?;
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    Ok(AuthenticatedUser {
        user_id: UserId(user_id),
        role: claims.role,
    })
}

fn parse_role_header(headers: &HeaderMap) -> Result<UserRole, AppError> {
    let role = headers
        .get("role")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Forbidden("role header is required".into()))?;

    role.parse()
}
