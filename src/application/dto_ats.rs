use serde::{Deserialize, Serialize};

use crate::domain::{
    ats::{AtsApiKey, IntegrationApiScope},
    ids::AtsApiKeyId,
};

use super::dto_diploma::PublicDiplomaView;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAtsApiKeyRequest {
    pub name: String,
    pub scope: IntegrationApiScope,
}

#[derive(Debug, Clone, Serialize)]
pub struct AtsApiKeySummary {
    pub api_key_id: AtsApiKeyId,
    pub name: String,
    pub scope: IntegrationApiScope,
    pub key_prefix: String,
    pub daily_request_limit: usize,
    pub burst_request_limit: usize,
    pub burst_window_seconds: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub active: bool,
}

impl From<AtsApiKey> for AtsApiKeySummary {
    fn from(value: AtsApiKey) -> Self {
        let active = value.is_active();
        Self {
            api_key_id: value.id,
            name: value.name,
            scope: value.scope,
            key_prefix: value.key_prefix,
            daily_request_limit: value.daily_request_limit,
            burst_request_limit: value.burst_request_limit,
            burst_window_seconds: value.burst_window_seconds,
            created_at: value.created_at,
            last_used_at: value.last_used_at,
            revoked_at: value.revoked_at,
            active,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateAtsApiKeyResponse {
    pub api_key_id: AtsApiKeyId,
    pub name: String,
    pub scope: IntegrationApiScope,
    pub key_prefix: String,
    pub api_key: String,
    pub daily_request_limit: usize,
    pub burst_request_limit: usize,
    pub burst_window_seconds: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AtsApiKeyListResponse {
    pub items: Vec<AtsApiKeySummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtsVerifyRequest {
    pub diploma_number: Option<String>,
    pub university_code: Option<String>,
    pub candidate_reference: Option<String>,
    pub resume_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AtsVerificationDecision {
    Verified,
    ManualReview,
    NotFound,
}

#[derive(Debug, Clone, Serialize)]
pub struct AtsVerifyResponse {
    pub decision: AtsVerificationDecision,
    pub verified: bool,
    pub match_count: usize,
    pub checked_at: chrono::DateTime<chrono::Utc>,
    pub candidate_reference: Option<String>,
    pub resume_reference: Option<String>,
    pub integration_name: String,
    pub risk_flags: Vec<String>,
    pub items: Vec<PublicDiplomaView>,
}
