pub mod auth;
pub mod common;
pub mod hr;
pub mod middleware;
mod router;
pub mod student;
pub mod university;

use std::sync::Arc;

pub use router::create_router;

use crate::{
    application::{
        ports::{HealthChecker, JwtProvider},
        services::{AtsService, AuthService, DiplomaService},
    },
    config::Settings,
    infrastructure::rate_limit::HrRateLimiter,
};

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub diploma_service: Arc<DiplomaService>,
    pub ats_service: Arc<AtsService>,
    pub auth_service: Arc<AuthService>,
    pub jwt_provider: Arc<dyn JwtProvider>,
    pub health_checker: Arc<dyn HealthChecker>,
    pub hr_rate_limiter: Arc<dyn HrRateLimiter>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        diploma_service: Arc<DiplomaService>,
        ats_service: Arc<AtsService>,
        auth_service: Arc<AuthService>,
        jwt_provider: Arc<dyn JwtProvider>,
        health_checker: Arc<dyn HealthChecker>,
        hr_rate_limiter: Arc<dyn HrRateLimiter>,
    ) -> Self {
        Self {
            settings,
            diploma_service,
            ats_service,
            auth_service,
            jwt_provider,
            health_checker,
            hr_rate_limiter,
        }
    }
}
