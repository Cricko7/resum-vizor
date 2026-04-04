use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Context;
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    application::services::{AtsService, AuthService, DiplomaService},
    config::Settings,
    http::{AppState, create_router},
    infrastructure::{
        api_keys::Blake3AtsKeyManager,
        auth::{ArgonPasswordHasher, JwtService},
        cache::{InMemoryResponseCache, RedisResponseCache, ResponseCache},
        hashing::Blake3DiplomaHasher,
        persistence::postgres::PostgresAppRepository,
        rate_limit::{HrRateLimiter, RedisRateLimiter, SimpleRateLimiter},
        signing::UniversityRecordSigner,
    },
};

pub async fn run(settings: Settings) -> anyhow::Result<()> {
    init_tracing(&settings);
    let request_cache_ttl = Duration::from_secs(settings.server.request_cache_ttl_seconds);

    let hasher = Blake3DiplomaHasher::new(settings.security.diploma_hash_key.clone());
    let repository = Arc::new(PostgresAppRepository::connect(&settings.database).await?);
    repository.migrate().await?;
    let health_checker: Arc<dyn crate::application::ports::HealthChecker> = repository.clone();
    let jwt_provider = Arc::new(JwtService::new(
        &settings.security.jwt_secret,
        settings.security.jwt_ttl_minutes,
    ));
    let password_hasher = Arc::new(ArgonPasswordHasher);
    let ats_key_manager = Arc::new(Blake3AtsKeyManager::new(
        &settings.security.ats_api_key_secret,
    ));
    let signer = Arc::new(UniversityRecordSigner::new(
        &settings.security.university_signing_key,
    ));
    let hr_rate_limiter: Arc<dyn HrRateLimiter> = if let Some(redis_settings) = &settings.redis {
        let limiter = RedisRateLimiter::connect(redis_settings)
        .await
        .context("failed to initialize Redis-backed HR rate limiter")?;
        info!(
            "configured HR automation rate limiter backend: {}",
            limiter.backend_name()
        );
        Arc::new(limiter)
    } else {
        let limiter = SimpleRateLimiter::new();
        info!(
            "configured HR automation rate limiter backend: {}",
            limiter.backend_name()
        );
        Arc::new(limiter)
    };
    let response_cache: Arc<dyn ResponseCache> = if let Some(redis_settings) = &settings.redis {
        let cache = RedisResponseCache::connect(redis_settings)
            .await
            .context("failed to initialize Redis-backed response cache")?;
        info!("configured response cache backend: {}", cache.backend_name());
        Arc::new(cache)
    } else {
        let cache = InMemoryResponseCache::new();
        info!("configured response cache backend: {}", cache.backend_name());
        Arc::new(cache)
    };
    let diploma_service = Arc::new(DiplomaService::new(
        repository.clone(),
        Arc::new(hasher),
        signer,
        jwt_provider.clone(),
        response_cache,
        request_cache_ttl,
    ));
    let auth_service = Arc::new(AuthService::new(
        repository.clone(),
        password_hasher,
        jwt_provider.clone(),
        settings.security.jwt_ttl_minutes,
    ));
    let ats_service = Arc::new(AtsService::new(
        repository,
        ats_key_manager,
        settings.server.integration_api_key_burst_window_seconds,
        settings.server.integration_api_key_ats_only_burst_limit,
        settings.server.integration_api_key_hr_automation_only_burst_limit,
        settings.server.integration_api_key_combined_burst_limit,
    ));

    let state = AppState::new(
        settings.clone(),
        diploma_service,
        ats_service,
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
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("axum server failed")
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

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        let mut stream =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        stream.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            info!("received SIGTERM, starting graceful shutdown");
        }
    }
}
