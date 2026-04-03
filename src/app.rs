use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    application::services::{AuthService, DiplomaService},
    config::Settings,
    http::{AppState, create_router},
    infrastructure::{
        auth::{ArgonPasswordHasher, JwtService},
        hashing::Blake3DiplomaHasher,
        persistence::postgres::PostgresAppRepository,
        rate_limit::SimpleRateLimiter,
        signing::UniversityRecordSigner,
    },
};

pub async fn run(settings: Settings) -> anyhow::Result<()> {
    init_tracing(&settings);

    let hasher = Blake3DiplomaHasher::new(settings.security.diploma_hash_key.clone());
    let repository = Arc::new(PostgresAppRepository::connect(&settings.database).await?);
    repository.migrate().await?;
    let health_checker: Arc<dyn crate::application::ports::HealthChecker> = repository.clone();
    let jwt_provider = Arc::new(JwtService::new(
        &settings.security.jwt_secret,
        settings.security.jwt_ttl_minutes,
    ));
    let password_hasher = Arc::new(ArgonPasswordHasher);
    let signer = Arc::new(UniversityRecordSigner::new(
        &settings.security.university_signing_key,
    ));
    let hr_rate_limiter = Arc::new(SimpleRateLimiter::new(
        settings.server.hr_api_rate_limit_requests,
        std::time::Duration::from_secs(settings.server.hr_api_rate_limit_window_seconds),
    ));
    let diploma_service = Arc::new(DiplomaService::new(
        repository.clone(),
        Arc::new(hasher),
        signer,
        jwt_provider.clone(),
    ));
    let auth_service = Arc::new(AuthService::new(
        repository,
        password_hasher,
        jwt_provider.clone(),
        settings.security.jwt_ttl_minutes,
    ));

    let state = AppState::new(
        settings.clone(),
        diploma_service,
        auth_service,
        jwt_provider,
        health_checker,
        hr_rate_limiter,
    );
    let app = create_router(state);

    let address = SocketAddr::from(([0, 0, 0, 0], settings.server.port));
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind to {}", address))?;

    info!("server listening on {}", address);
    axum::serve(listener, app).await.context("axum server failed")
}

fn init_tracing(settings: &Settings) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| settings.server.log_level.clone().into()),
        )
        .with_target(false)
        .compact()
        .init();
}
