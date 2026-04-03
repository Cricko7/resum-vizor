use axum::{Router, middleware, routing::{get, post}};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::http::{
    AppState, auth, common, hr,
    middleware::{
        enforce_hr_automation_rate_limit, require_hr_role, require_student_role,
        require_university_role,
    },
    student, university,
};
use crate::infrastructure::metrics::metrics_middleware;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/change-password", post(auth::change_password))
        .route("/me", get(auth::me));

    let university_routes = Router::new()
        .route("/diplomas", post(university::register_diploma))
        .route("/diplomas/import", post(university::import_diplomas))
        .route("/diplomas/{diploma_id}/revoke", post(university::revoke_diploma))
        .route("/diplomas/{diploma_id}/restore", post(university::restore_diploma))
        .layer(middleware::from_fn_with_state(state.clone(), require_university_role));

    let student_routes = Router::new()
        .route("/profile", get(student::profile))
        .route("/search", post(student::search_my_diplomas))
        .route("/diplomas/{diploma_id}/share-link", post(student::generate_share_link))
        .layer(middleware::from_fn_with_state(state.clone(), require_student_role));

    let hr_routes = Router::new()
        .route("/verify", post(hr::verify_diploma))
        .route("/registry/search", post(hr::search_registry))
        .route(
            "/automation/verify",
            post(hr::automation_verify).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_hr_automation_rate_limit,
            )),
        )
        .layer(middleware::from_fn_with_state(state.clone(), require_hr_role));

    Router::new()
        .route("/health", get(common::health_check))
        .route("/health/live", get(common::liveness_check))
        .route("/health/ready", get(common::readiness_check))
        .route("/metrics", get(common::metrics_handler))
        .route("/api/v1/public/diplomas/access/{token}", get(common::public_diploma_access))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/university", university_routes)
        .nest("/api/v1/student", student_routes)
        .nest("/api/v1/hr", hr_routes)
        .layer(middleware::from_fn(metrics_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
