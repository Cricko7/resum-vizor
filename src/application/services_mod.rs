mod services_auth;
mod services_ats;
mod services_diploma;
mod services_qr;
mod services_support;
#[cfg(test)]
mod services_tests;

pub use services_auth::AuthService;
pub use services_ats::{AtsService, DEFAULT_INTEGRATION_API_KEY_DAILY_LIMIT};
pub use services_diploma::DiplomaService;
pub use services_qr::QrService;
