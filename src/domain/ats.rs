use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{AtsApiKeyId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationApiScope {
    AtsOnly,
    HrAutomationOnly,
    Combined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtsApiKey {
    pub id: AtsApiKeyId,
    pub hr_user_id: UserId,
    pub name: String,
    pub scope: IntegrationApiScope,
    pub key_prefix: String,
    pub key_hash: String,
    pub daily_request_limit: usize,
    pub burst_request_limit: usize,
    pub burst_window_seconds: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl AtsApiKey {
    pub fn new(
        hr_user_id: UserId,
        name: String,
        scope: IntegrationApiScope,
        key_prefix: String,
        key_hash: String,
        daily_request_limit: usize,
        burst_request_limit: usize,
        burst_window_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: AtsApiKeyId::new(),
            hr_user_id,
            name,
            scope,
            key_prefix,
            key_hash,
            daily_request_limit,
            burst_request_limit,
            burst_window_seconds,
            created_at: now,
            updated_at: now,
            last_used_at: None,
            revoked_at: None,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn revoke(&mut self) {
        let now = Utc::now();
        self.revoked_at = Some(now);
        self.updated_at = now;
    }

    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }

    pub fn allows_ats_verify(&self) -> bool {
        matches!(self.scope, IntegrationApiScope::AtsOnly | IntegrationApiScope::Combined)
    }

    pub fn allows_hr_automation(&self) -> bool {
        matches!(
            self.scope,
            IntegrationApiScope::HrAutomationOnly | IntegrationApiScope::Combined
        )
    }
}
