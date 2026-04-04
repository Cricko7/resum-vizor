use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use secrecy::SecretString;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use resume_vizor::{
    application::{
        ports::{AppRepository, HealthChecker, JwtProvider},
        services::{AtsService, AuthService, DiplomaService},
    },
    config::{DatabaseSettings, SecuritySettings, ServerSettings, Settings},
    http::{AppState, create_router},
    infrastructure::{
        api_keys::Blake3AtsKeyManager,
        auth::{ArgonPasswordHasher, JwtService},
        cache::InMemoryResponseCache,
        hashing::Blake3DiplomaHasher,
        persistence::in_memory::InMemoryAppRepository,
        rate_limit::{HrRateLimiter, SimpleRateLimiter},
        signing::UniversityRecordSigner,
    },
};

fn secret(value: &str) -> SecretString {
    SecretString::new(value.to_string().into_boxed_str())
}

fn test_settings() -> Settings {
    Settings {
        server: ServerSettings {
            port: 8080,
            log_level: "info".to_string(),
            base_url: "http://localhost:8080".to_string(),
            request_cache_ttl_seconds: 60,
            hr_api_rate_limit_requests: 60,
            hr_api_rate_limit_window_seconds: 60,
            integration_api_key_burst_window_seconds: 10,
            integration_api_key_ats_only_burst_limit: 30,
            integration_api_key_hr_automation_only_burst_limit: 20,
            integration_api_key_combined_burst_limit: 40,
        },
        database: DatabaseSettings {
            url: secret("postgres://postgres:postgres@localhost:5432/resume_vizor"),
            max_connections: 10,
        },
        redis: None,
        security: SecuritySettings {
            diploma_hash_key: secret("hash-secret"),
            jwt_secret: secret("jwt-secret"),
            ats_api_key_secret: secret("ats-secret"),
            jwt_ttl_minutes: 60,
            university_signing_key: secret("sign-secret"),
            diploma_link_ttl_minutes: 30,
        },
    }
}

fn build_app() -> Router {
    let settings = test_settings();
    let repository = Arc::new(InMemoryAppRepository::default());
    let app_repository: Arc<dyn AppRepository> = repository.clone();
    let health_checker: Arc<dyn HealthChecker> = repository.clone();
    let jwt_provider: Arc<dyn JwtProvider> =
        Arc::new(JwtService::new(&settings.security.jwt_secret, settings.security.jwt_ttl_minutes));
    let hr_rate_limiter: Arc<dyn HrRateLimiter> = Arc::new(SimpleRateLimiter::new());
    let diploma_service = Arc::new(DiplomaService::new(
        app_repository.clone(),
        Arc::new(Blake3DiplomaHasher::new(settings.security.diploma_hash_key.clone())),
        Arc::new(UniversityRecordSigner::new(&settings.security.university_signing_key)),
        jwt_provider.clone(),
        Arc::new(InMemoryResponseCache::new()),
        Duration::from_secs(settings.server.request_cache_ttl_seconds),
    ));
    let auth_service = Arc::new(AuthService::new(
        app_repository.clone(),
        Arc::new(ArgonPasswordHasher),
        jwt_provider.clone(),
        settings.security.jwt_ttl_minutes,
    ));
    let ats_service = Arc::new(AtsService::new(
        app_repository,
        Arc::new(Blake3AtsKeyManager::new(&settings.security.ats_api_key_secret)),
        settings.server.integration_api_key_burst_window_seconds,
        settings.server.integration_api_key_ats_only_burst_limit,
        settings.server.integration_api_key_hr_automation_only_burst_limit,
        settings.server.integration_api_key_combined_burst_limit,
    ));

    create_router(AppState::new(
        settings,
        diploma_service,
        ats_service,
        auth_service,
        jwt_provider,
        health_checker,
        hr_rate_limiter,
    ))
}

async fn send_request(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    headers: &[(&str, &str)],
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }

    let request = if let Some(body) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .expect("request should build")
    } else {
        builder.body(Body::empty()).expect("request should build")
    };

    let response = app.clone().oneshot(request).await.expect("request should succeed");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("response body should be valid json")
    };

    (status, value)
}

fn bearer_token(response: &Value) -> String {
    response["access_token"]
        .as_str()
        .expect("access_token should exist")
        .to_string()
}

#[tokio::test]
async fn health_and_openapi_endpoints_are_available() {
    let app = build_app();

    let (health_status, health_body) = send_request(&app, Method::GET, "/health", None, &[]).await;
    assert_eq!(health_status, StatusCode::OK);
    assert_eq!(health_body["status"], "ok");

    let (openapi_status, openapi_body) =
        send_request(&app, Method::GET, "/api-docs/openapi.json", None, &[]).await;
    assert_eq!(openapi_status, StatusCode::OK);
    let paths = openapi_body["paths"].as_object().expect("paths should be object");
    assert!(paths.contains_key("/api/v1/auth/login"));
    assert!(paths.contains_key("/api/v1/ats/verify"));
}

#[tokio::test]
async fn register_login_and_me_work_end_to_end() {
    let app = build_app();

    let register_body = json!({
        "email": "student@example.com",
        "password": "superpass",
        "full_name": "Test Student",
        "student_number": "ST-1001",
        "role": "student",
        "university_id": null,
        "university_code": null
    });

    let (register_status, _register_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        Some(register_body),
        &[],
    )
    .await;
    assert_eq!(register_status, StatusCode::OK);

    let (login_status, login_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        Some(json!({
            "email": "student@example.com",
            "password": "superpass"
        })),
        &[],
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);

    let token = bearer_token(&login_response);
    let auth_header = format!("Bearer {token}");

    let (me_status, me_response) = send_request(
        &app,
        Method::GET,
        "/api/v1/auth/me",
        None,
        &[("authorization", auth_header.as_str())],
    )
    .await;
    assert_eq!(me_status, StatusCode::OK);
    assert_eq!(me_response["email"], "student@example.com");
    assert_eq!(me_response["role"], "student");
}

#[tokio::test]
async fn student_can_claim_share_and_publish_diploma() {
    let app = build_app();
    let university_id = Uuid::new_v4().to_string();

    let (_, university_register) = send_request(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        Some(json!({
            "email": "uni@example.com",
            "password": "superpass",
            "full_name": "Test University",
            "student_number": null,
            "role": "university",
            "university_id": university_id,
            "university_code": "UNI-001"
        })),
        &[],
    )
    .await;
    let university_token = bearer_token(&university_register);
    let university_auth = format!("Bearer {university_token}");

    let (create_status, create_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/university/diplomas",
        Some(json!({
            "student_full_name": "Ivan Petrov",
            "student_number": "ST-1001",
            "student_birth_date": null,
            "diploma_number": "DP-2026-0001",
            "degree": "bachelor",
            "program_name": "computer science",
            "graduation_date": "2026-06-30",
            "honors": false
        })),
        &[("authorization", university_auth.as_str()), ("role", "university")],
    )
    .await;
    assert_eq!(create_status, StatusCode::OK);
    let diploma_id = create_response["diploma_id"]
        .as_str()
        .expect("diploma_id should exist")
        .to_string();

    let (_, student_register) = send_request(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        Some(json!({
            "email": "ivan@example.com",
            "password": "superpass",
            "full_name": "Ivan Petrov",
            "student_number": "ST-1001",
            "role": "student",
            "university_id": null,
            "university_code": null
        })),
        &[],
    )
    .await;
    let student_token = bearer_token(&student_register);
    let student_auth = format!("Bearer {student_token}");

    let (search_status, search_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/student/search",
        Some(json!({
            "diploma_number": "DP-2026-0001",
            "student_full_name": "Ivan Petrov"
        })),
        &[("authorization", student_auth.as_str()), ("role", "student")],
    )
    .await;
    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(
        search_response["items"].as_array().expect("items array").len(),
        1
    );

    let share_uri = format!("/api/v1/student/diplomas/{diploma_id}/share-link");
    let (share_status, share_response) = send_request(
        &app,
        Method::POST,
        &share_uri,
        None,
        &[("authorization", student_auth.as_str()), ("role", "student")],
    )
    .await;
    assert_eq!(share_status, StatusCode::OK);

    let access_url = share_response["access_url"]
        .as_str()
        .expect("access_url should exist");
    let token = access_url.rsplit('/').next().expect("token should exist");
    let public_uri = format!("/api/v1/public/diplomas/access/{token}");

    let (public_status, public_response) =
        send_request(&app, Method::GET, &public_uri, None, &[]).await;
    assert_eq!(public_status, StatusCode::OK);
    assert_eq!(public_response["university_code"], "UNI-001");
    assert_eq!(public_response["status"], "active");
}

#[tokio::test]
async fn combined_hr_api_key_can_call_ats_and_automation_endpoints() {
    let app = build_app();
    let university_id = Uuid::new_v4().to_string();

    let (_, university_register) = send_request(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        Some(json!({
            "email": "uni@example.com",
            "password": "superpass",
            "full_name": "Test University",
            "student_number": null,
            "role": "university",
            "university_id": university_id,
            "university_code": "UNI-900"
        })),
        &[],
    )
    .await;
    let university_auth = format!("Bearer {}", bearer_token(&university_register));

    let (create_status, _) = send_request(
        &app,
        Method::POST,
        "/api/v1/university/diplomas",
        Some(json!({
            "student_full_name": "Alice Hr",
            "student_number": "ST-7777",
            "student_birth_date": null,
            "diploma_number": "DP-ATS-0001",
            "degree": "master",
            "program_name": "analytics",
            "graduation_date": "2026-06-30",
            "honors": false
        })),
        &[("authorization", university_auth.as_str()), ("role", "university")],
    )
    .await;
    assert_eq!(create_status, StatusCode::OK);

    let (_, hr_register) = send_request(
        &app,
        Method::POST,
        "/api/v1/auth/register",
        Some(json!({
            "email": "hr@example.com",
            "password": "superpass",
            "full_name": "Recruiter",
            "student_number": null,
            "role": "hr",
            "university_id": null,
            "university_code": null
        })),
        &[],
    )
    .await;
    let hr_auth = format!("Bearer {}", bearer_token(&hr_register));

    let (api_key_status, api_key_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/hr/api-keys",
        Some(json!({
            "name": "Greenhouse prod",
            "scope": "combined"
        })),
        &[("authorization", hr_auth.as_str()), ("role", "hr")],
    )
    .await;
    assert_eq!(api_key_status, StatusCode::OK);
    let api_key = api_key_response["api_key"]
        .as_str()
        .expect("api_key should exist");

    let (ats_status, ats_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/ats/verify",
        Some(json!({
            "diploma_number": "DP-ATS-0001",
            "university_code": "UNI-900",
            "candidate_reference": "candidate-42",
            "resume_reference": "resume-42"
        })),
        &[("x-api-key", api_key)],
    )
    .await;
    assert_eq!(ats_status, StatusCode::OK);
    assert_eq!(ats_response["decision"], "verified");
    assert_eq!(ats_response["verified"], true);

    let (automation_status, automation_response) = send_request(
        &app,
        Method::POST,
        "/api/v1/hr/automation/verify",
        Some(json!({
            "diploma_number": "DP-ATS-0001",
            "university_code": "UNI-900"
        })),
        &[("x-api-key", api_key)],
    )
    .await;
    assert_eq!(automation_status, StatusCode::OK);
    assert_eq!(
        automation_response["items"]
            .as_array()
            .expect("items should be array")
            .len(),
        1
    );
}
