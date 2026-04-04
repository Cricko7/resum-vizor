use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    application::ports::{AtsApiKeyRepository, DiplomaRepository, HealthChecker, UserRepository},
    domain::{
        ats::AtsApiKey,
        diploma::Diploma,
        ids::{AtsApiKeyId, CertificateId, DiplomaId, UserId},
        user::User,
    },
    error::AppError,
};

#[derive(Debug, Default)]
pub struct InMemoryAppRepository {
    diplomas_by_id: Arc<RwLock<HashMap<String, Diploma>>>,
    users_by_id: Arc<RwLock<HashMap<String, User>>>,
    ats_api_keys_by_id: Arc<RwLock<HashMap<String, AtsApiKey>>>,
}

#[async_trait]
impl DiplomaRepository for InMemoryAppRepository {
    async fn save(&self, diploma: Diploma) -> Result<Diploma, AppError> {
        let mut storage = self.diplomas_by_id.write().await;

        if storage
            .values()
            .any(|item| item.hashed_payload.canonical_document_hash == diploma.hashed_payload.canonical_document_hash)
        {
            return Err(AppError::Conflict("diploma already registered".into()));
        }

        storage.insert(diploma.id.0.to_string(), diploma.clone());
        Ok(diploma)
    }

    async fn find_by_student_id(&self, student_id: UserId) -> Result<Vec<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        let mut diplomas = storage
            .values()
            .filter(|diploma| diploma.student_account_id == Some(student_id))
            .cloned()
            .collect::<Vec<_>>();
        diplomas.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(diplomas)
    }

    async fn find_by_certificate_id(
        &self,
        certificate_id: CertificateId,
    ) -> Result<Option<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        Ok(storage
            .values()
            .find(|diploma| diploma.certificate_id.0 == certificate_id.0)
            .cloned())
    }

    async fn find_by_canonical_hash(&self, canonical_hash: &str) -> Result<Option<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        Ok(storage
            .values()
            .find(|diploma| diploma.hashed_payload.verification_lookup_hash == canonical_hash)
            .cloned())
    }

    async fn find_by_id(&self, diploma_id: DiplomaId) -> Result<Option<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        Ok(storage.get(&diploma_id.0.to_string()).cloned())
    }

    async fn update(&self, diploma: Diploma) -> Result<Diploma, AppError> {
        let mut storage = self.diplomas_by_id.write().await;
        storage.insert(diploma.id.0.to_string(), diploma.clone());
        Ok(diploma)
    }

    async fn search_by_student_name_hash(&self, full_name_hash: &str) -> Result<Vec<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        let mut diplomas = storage
            .values()
            .filter(|diploma| diploma.hashed_payload.student_full_name_hash == full_name_hash)
            .cloned()
            .collect::<Vec<_>>();
        diplomas.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(diplomas)
    }

    async fn search_by_diploma_number_hash(
        &self,
        diploma_number_hash: &str,
    ) -> Result<Vec<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        let mut diplomas = storage
            .values()
            .filter(|diploma| diploma.hashed_payload.diploma_number_hash == diploma_number_hash)
            .cloned()
            .collect::<Vec<_>>();
        diplomas.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(diplomas)
    }

    async fn search_by_university_code_hash(
        &self,
        university_code_hash: &str,
    ) -> Result<Vec<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        let mut diplomas = storage
            .values()
            .filter(|diploma| diploma.hashed_payload.university_code_hash == university_code_hash)
            .cloned()
            .collect::<Vec<_>>();
        diplomas.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(diplomas)
    }
}

#[async_trait]
impl UserRepository for InMemoryAppRepository {
    async fn create_user(&self, user: User) -> Result<User, AppError> {
        let mut storage = self.users_by_id.write().await;

        if storage
            .values()
            .any(|existing| existing.email.eq_ignore_ascii_case(&user.email))
        {
            return Err(AppError::Conflict("user with this email already exists".into()));
        }

        storage.insert(user.id.0.to_string(), user.clone());
        Ok(user)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let storage = self.users_by_id.read().await;
        Ok(storage
            .values()
            .find(|user| user.email.eq_ignore_ascii_case(email))
            .cloned())
    }

    async fn find_user_by_id(&self, user_id: UserId) -> Result<Option<User>, AppError> {
        let storage = self.users_by_id.read().await;
        Ok(storage.get(&user_id.0.to_string()).cloned())
    }

    async fn update_user(&self, user: User) -> Result<User, AppError> {
        let mut storage = self.users_by_id.write().await;
        storage.insert(user.id.0.to_string(), user.clone());
        Ok(user)
    }
}

#[async_trait]
impl AtsApiKeyRepository for InMemoryAppRepository {
    async fn create_ats_api_key(&self, api_key: AtsApiKey) -> Result<AtsApiKey, AppError> {
        let mut storage = self.ats_api_keys_by_id.write().await;

        if storage
            .values()
            .any(|existing| existing.key_hash == api_key.key_hash)
        {
            return Err(AppError::Conflict("ATS API key already exists".into()));
        }

        storage.insert(api_key.id.0.to_string(), api_key.clone());
        Ok(api_key)
    }

    async fn find_ats_api_key_by_hash(&self, key_hash: &str) -> Result<Option<AtsApiKey>, AppError> {
        let storage = self.ats_api_keys_by_id.read().await;
        Ok(storage
            .values()
            .find(|api_key| api_key.key_hash == key_hash)
            .cloned())
    }

    async fn find_ats_api_key_by_id(&self, api_key_id: AtsApiKeyId) -> Result<Option<AtsApiKey>, AppError> {
        let storage = self.ats_api_keys_by_id.read().await;
        Ok(storage.get(&api_key_id.0.to_string()).cloned())
    }

    async fn list_ats_api_keys_by_hr_user(&self, hr_user_id: UserId) -> Result<Vec<AtsApiKey>, AppError> {
        let storage = self.ats_api_keys_by_id.read().await;
        let mut items = storage
            .values()
            .filter(|api_key| api_key.hr_user_id == hr_user_id)
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(items)
    }

    async fn update_ats_api_key(&self, api_key: AtsApiKey) -> Result<AtsApiKey, AppError> {
        let mut storage = self.ats_api_keys_by_id.write().await;
        storage.insert(api_key.id.0.to_string(), api_key.clone());
        Ok(api_key)
    }
}

#[async_trait]
impl HealthChecker for InMemoryAppRepository {
    async fn is_ready(&self) -> bool {
        true
    }
}
