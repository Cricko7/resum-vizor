use axum::{
    extract::{FromRequestParts, State},
    http::{HeaderMap, Request, header},
    middleware::Next,
    response::Response,
};
use std::time::Duration;

use crate::{
    domain::{
        ids::{AtsApiKeyId, UserId},
        user::UserRole,
    },
    error::AppError,
    http::AppState,
};
use axum::extract::FromRef;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedAtsClient {
    pub api_key_id: AtsApiKeyId,
    pub hr_user_id: UserId,
    pub integration_name: String,
    pub daily_request_limit: usize,
    pub burst_request_limit: usize,
    pub burst_window_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedAutomationClient {
    pub api_key_id: AtsApiKeyId,
    pub hr_user_id: UserId,
    pub integration_name: String,
    pub daily_request_limit: usize,
    pub burst_request_limit: usize,
    pub burst_window_seconds: u64,
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

impl<S> FromRequestParts<S> for AuthenticatedAtsClient
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
        let key = authenticate_integration_client(&parts.headers, &state, IntegrationTarget::Ats)
            .await?;

        Ok(Self {
            api_key_id: key.id,
            hr_user_id: key.hr_user_id,
            integration_name: key.name,
            daily_request_limit: key.daily_request_limit,
            burst_request_limit: key.burst_request_limit,
            burst_window_seconds: key.burst_window_seconds,
        })
    }
}

impl<S> FromRequestParts<S> for AuthenticatedAutomationClient
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
        let key = authenticate_integration_client(
            &parts.headers,
            &state,
            IntegrationTarget::HrAutomation,
        )
        .await?;

        Ok(Self {
            api_key_id: key.id,
            hr_user_id: key.hr_user_id,
            integration_name: key.name,
            daily_request_limit: key.daily_request_limit,
            burst_request_limit: key.burst_request_limit,
            burst_window_seconds: key.burst_window_seconds,
        })
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

#[derive(Debug, Clone, Copy)]
enum IntegrationTarget {
    Ats,
    HrAutomation,
}

async fn authenticate_integration_client(
    headers: &HeaderMap,
    state: &AppState,
    target: IntegrationTarget,
) -> Result<crate::domain::ats::AtsApiKey, AppError> {
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let key = match target {
        IntegrationTarget::Ats => state.ats_service.authorize_api_key_for_ats(api_key).await?,
        IntegrationTarget::HrAutomation => {
            state
                .ats_service
                .authorize_api_key_for_hr_automation(api_key)
                .await?
        }
    };

    let rate_limit_key = format!("integration_api_key:{}", key.id.0);
    if !state
        .hr_rate_limiter
        .allow(&rate_limit_key, key.daily_request_limit, Duration::from_secs(86_400))
        .await
    {
        return Err(AppError::RateLimited);
    }

    let burst_rate_limit_key = format!("integration_api_key_burst:{}", key.id.0);
    if !state
        .hr_rate_limiter
        .allow(
            &burst_rate_limit_key,
            key.burst_request_limit,
            Duration::from_secs(key.burst_window_seconds),
        )
        .await
    {
        return Err(AppError::RateLimited);
    }

    Ok(key)
}
