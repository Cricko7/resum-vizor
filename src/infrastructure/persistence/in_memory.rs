use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    application::ports::{DiplomaRepository, HealthChecker, UserRepository},
    domain::{
        diploma::Diploma,
        ids::{CertificateId, DiplomaId, UserId},
        user::User,
    },
    error::AppError,
};

#[derive(Debug, Default)]
pub struct InMemoryAppRepository {
    diplomas_by_id: Arc<RwLock<HashMap<String, Diploma>>>,
    users_by_id: Arc<RwLock<HashMap<String, User>>>,
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
        Ok(storage
            .values()
            .filter(|diploma| diploma.student_account_id == Some(student_id))
            .cloned()
            .collect())
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
        Ok(storage
            .values()
            .filter(|diploma| diploma.hashed_payload.student_full_name_hash == full_name_hash)
            .cloned()
            .collect())
    }

    async fn search_by_diploma_number_hash(
        &self,
        diploma_number_hash: &str,
    ) -> Result<Vec<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        Ok(storage
            .values()
            .filter(|diploma| diploma.hashed_payload.diploma_number_hash == diploma_number_hash)
            .cloned()
            .collect())
    }

    async fn search_by_university_code_hash(
        &self,
        university_code_hash: &str,
    ) -> Result<Vec<Diploma>, AppError> {
        let storage = self.diplomas_by_id.read().await;
        Ok(storage
            .values()
            .filter(|diploma| diploma.hashed_payload.university_code_hash == university_code_hash)
            .cloned()
            .collect())
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
impl HealthChecker for InMemoryAppRepository {
    async fn is_ready(&self) -> bool {
        true
    }
}
