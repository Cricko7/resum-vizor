use std::sync::Arc;

use crate::{
    application::{
        dto::{
            AtsApiKeyListResponse, AtsApiKeySummary, AtsVerificationDecision, AtsVerifyRequest,
            AtsVerifyResponse, CreateAtsApiKeyRequest, CreateAtsApiKeyResponse,
            HrRegistrySearchRequest,
        },
        ports::{AppRepository, AtsKeyManager},
    },
    domain::{
        ats::{AtsApiKey, IntegrationApiScope},
        ids::{AtsApiKeyId, UserId},
        user::{User, UserRole},
    },
    error::AppError,
};

use super::services_diploma::DiplomaService;

pub const DEFAULT_INTEGRATION_API_KEY_DAILY_LIMIT: usize = 1_000;

pub struct AtsService {
    repository: Arc<dyn AppRepository>,
    key_manager: Arc<dyn AtsKeyManager>,
    burst_window_seconds: u64,
    ats_only_burst_limit: usize,
    hr_automation_only_burst_limit: usize,
    combined_burst_limit: usize,
}

impl AtsService {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        key_manager: Arc<dyn AtsKeyManager>,
        burst_window_seconds: u64,
        ats_only_burst_limit: usize,
        hr_automation_only_burst_limit: usize,
        combined_burst_limit: usize,
    ) -> Self {
        Self {
            repository,
            key_manager,
            burst_window_seconds,
            ats_only_burst_limit,
            hr_automation_only_burst_limit,
            combined_burst_limit,
        }
    }

    pub async fn create_api_key(
        &self,
        hr_user_id: UserId,
        request: CreateAtsApiKeyRequest,
    ) -> Result<CreateAtsApiKeyResponse, AppError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("name is required".into()));
        }

        self.ensure_hr_user(hr_user_id).await?;

        let api_key = self.key_manager.generate_api_key()?;
        let key_hash = self.key_manager.hash_api_key(&api_key)?;
        let key_prefix = self.key_manager.key_prefix(&api_key);
        let key = AtsApiKey::new(
            hr_user_id,
            name.to_string(),
            request.scope,
            key_prefix.clone(),
            key_hash,
            self.daily_limit_for_scope(request.scope),
            self.burst_limit_for_scope(request.scope),
            self.burst_window_seconds,
        );
        let created = self.repository.create_ats_api_key(key).await?;

        Ok(CreateAtsApiKeyResponse {
            api_key_id: created.id,
            name: created.name,
            scope: created.scope,
            key_prefix,
            api_key,
            daily_request_limit: created.daily_request_limit,
            burst_request_limit: created.burst_request_limit,
            burst_window_seconds: created.burst_window_seconds,
            created_at: created.created_at,
        })
    }

    pub async fn list_api_keys(&self, hr_user_id: UserId) -> Result<AtsApiKeyListResponse, AppError> {
        self.ensure_hr_user(hr_user_id).await?;

        let items = self
            .repository
            .list_ats_api_keys_by_hr_user(hr_user_id)
            .await?
            .into_iter()
            .map(AtsApiKeySummary::from)
            .collect();

        Ok(AtsApiKeyListResponse { items })
    }

    pub async fn revoke_api_key(
        &self,
        hr_user_id: UserId,
        api_key_id: AtsApiKeyId,
    ) -> Result<AtsApiKeySummary, AppError> {
        self.ensure_hr_user(hr_user_id).await?;

        let mut api_key = self
            .repository
            .find_ats_api_key_by_id(api_key_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if api_key.hr_user_id != hr_user_id {
            return Err(AppError::Forbidden(
                "cannot manage ATS key owned by another HR user".into(),
            ));
        }

        if api_key.revoked_at.is_none() {
            api_key.revoke();
            api_key = self.repository.update_ats_api_key(api_key).await?;
        }

        Ok(api_key.into())
    }

    pub async fn authorize_api_key_for_ats(&self, api_key: &str) -> Result<AtsApiKey, AppError> {
        let api_key = self.authenticate_api_key(api_key).await?;

        if !api_key.allows_ats_verify() {
            return Err(AppError::Forbidden(
                "api key is not allowed to access /api/v1/ats/verify".into(),
            ));
        }

        Ok(api_key)
    }

    pub async fn authorize_api_key_for_hr_automation(
        &self,
        api_key: &str,
    ) -> Result<AtsApiKey, AppError> {
        let api_key = self.authenticate_api_key(api_key).await?;

        if !api_key.allows_hr_automation() {
            return Err(AppError::Forbidden(
                "api key is not allowed to access /api/v1/hr/automation/verify".into(),
            ));
        }

        Ok(api_key)
    }

    pub async fn verify_for_ats(
        &self,
        integration_name: &str,
        request: AtsVerifyRequest,
        diploma_service: &DiplomaService,
    ) -> Result<AtsVerifyResponse, AppError> {
        let result = diploma_service
            .search_hr_registry(HrRegistrySearchRequest {
                diploma_number: request.diploma_number,
                university_code: request.university_code,
            })
            .await?;

        let mut risk_flags = Vec::new();

        if result.items.is_empty() {
            risk_flags.push("no_matches".to_string());
        }
        if result.items.len() > 1 {
            risk_flags.push("multiple_matches".to_string());
        }
        if result.items.iter().any(|item| {
            matches!(item.status, crate::domain::diploma::DiplomaStatus::Revoked)
        }) {
            risk_flags.push("revoked_match_present".to_string());
        }

        let verified = result.items.len() == 1
            && result.items.first().is_some_and(|item| {
                matches!(item.status, crate::domain::diploma::DiplomaStatus::Active)
            });

        let decision = if result.items.is_empty() {
            AtsVerificationDecision::NotFound
        } else if verified {
            AtsVerificationDecision::Verified
        } else {
            AtsVerificationDecision::ManualReview
        };

        Ok(AtsVerifyResponse {
            decision,
            verified,
            match_count: result.items.len(),
            checked_at: chrono::Utc::now(),
            candidate_reference: request.candidate_reference,
            resume_reference: request.resume_reference,
            integration_name: integration_name.to_string(),
            risk_flags,
            items: result.items,
        })
    }

    async fn authenticate_api_key(&self, api_key: &str) -> Result<AtsApiKey, AppError> {
        let key_hash = self.key_manager.hash_api_key(api_key)?;
        let mut api_key = self
            .repository
            .find_ats_api_key_by_hash(&key_hash)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !api_key.is_active() {
            return Err(AppError::Unauthorized);
        }

        api_key.mark_used();
        self.repository.update_ats_api_key(api_key.clone()).await
    }

    async fn ensure_hr_user(&self, user_id: UserId) -> Result<User, AppError> {
        let user = self
            .repository
            .find_user_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if user.role != UserRole::Hr {
            return Err(AppError::Forbidden(
                "ATS integrations can be managed only by HR accounts".into(),
            ));
        }

        Ok(user)
    }

    fn daily_limit_for_scope(&self, scope: IntegrationApiScope) -> usize {
        let _ = scope;
        DEFAULT_INTEGRATION_API_KEY_DAILY_LIMIT
    }

    fn burst_limit_for_scope(&self, scope: IntegrationApiScope) -> usize {
        match scope {
            IntegrationApiScope::AtsOnly => self.ats_only_burst_limit,
            IntegrationApiScope::HrAutomationOnly => self.hr_automation_only_burst_limit,
            IntegrationApiScope::Combined => self.combined_burst_limit,
        }
    }
}
