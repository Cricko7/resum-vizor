use async_trait::async_trait;

use crate::{
    domain::{
        ats::AtsApiKey,
        diploma::Diploma,
        ids::{AtsApiKeyId, CertificateId, DiplomaId, UniversityId, UserId},
        qr::{CreateQrJobPayload, DiplomaQrCode, ExternalQrJob, ExternalQrMetadata, QrBinaryContent},
        user::User,
    },
    error::AppError,
};

#[async_trait]
pub trait DiplomaRepository: Send + Sync {
    async fn save(&self, diploma: Diploma) -> Result<Diploma, AppError>;
    async fn find_by_student_id(&self, student_id: UserId) -> Result<Vec<Diploma>, AppError>;
    async fn find_by_certificate_id(
        &self,
        certificate_id: CertificateId,
    ) -> Result<Option<Diploma>, AppError>;
    async fn find_by_canonical_hash(&self, canonical_hash: &str) -> Result<Option<Diploma>, AppError>;
    async fn find_by_id(&self, diploma_id: DiplomaId) -> Result<Option<Diploma>, AppError>;
    async fn update(&self, diploma: Diploma) -> Result<Diploma, AppError>;
    async fn search_by_student_name_hash(&self, full_name_hash: &str) -> Result<Vec<Diploma>, AppError>;
    async fn search_by_diploma_number_hash(
        &self,
        diploma_number_hash: &str,
    ) -> Result<Vec<Diploma>, AppError>;
    async fn search_by_university_code_hash(
        &self,
        university_code_hash: &str,
    ) -> Result<Vec<Diploma>, AppError>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: User) -> Result<User, AppError>;
    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_user_by_id(&self, user_id: UserId) -> Result<Option<User>, AppError>;
    async fn update_user(&self, user: User) -> Result<User, AppError>;
}

#[async_trait]
pub trait AtsApiKeyRepository: Send + Sync {
    async fn create_ats_api_key(&self, api_key: AtsApiKey) -> Result<AtsApiKey, AppError>;
    async fn find_ats_api_key_by_hash(&self, key_hash: &str) -> Result<Option<AtsApiKey>, AppError>;
    async fn find_ats_api_key_by_id(&self, api_key_id: AtsApiKeyId) -> Result<Option<AtsApiKey>, AppError>;
    async fn list_ats_api_keys_by_hr_user(&self, hr_user_id: UserId) -> Result<Vec<AtsApiKey>, AppError>;
    async fn update_ats_api_key(&self, api_key: AtsApiKey) -> Result<AtsApiKey, AppError>;
}

#[async_trait]
pub trait DiplomaQrCodeRepository: Send + Sync {
    async fn upsert_diploma_qr_code(&self, qr_code: DiplomaQrCode) -> Result<DiplomaQrCode, AppError>;
    async fn find_diploma_qr_code_by_diploma_id(
        &self,
        diploma_id: DiplomaId,
    ) -> Result<Option<DiplomaQrCode>, AppError>;
    async fn delete_diploma_qr_code_by_diploma_id(&self, diploma_id: DiplomaId) -> Result<(), AppError>;
}

pub trait AppRepository:
    DiplomaRepository + UserRepository + AtsApiKeyRepository + DiplomaQrCodeRepository
{
}

impl<T> AppRepository for T where
    T: DiplomaRepository + UserRepository + AtsApiKeyRepository + DiplomaQrCodeRepository + ?Sized
{
}

pub trait PasswordHasher: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, AppError>;
    fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), AppError>;
}

pub trait JwtProvider: Send + Sync {
    fn issue_token(&self, user: &User) -> Result<String, AppError>;
    fn decode_token(&self, token: &str) -> Result<crate::infrastructure::auth::JwtClaims, AppError>;
    fn issue_diploma_access_token(
        &self,
        diploma_id: DiplomaId,
        ttl_minutes: i64,
    ) -> Result<String, AppError>;
    fn decode_diploma_access_token(
        &self,
        token: &str,
    ) -> Result<crate::infrastructure::auth::DiplomaAccessClaims, AppError>;
}

pub trait DiplomaSigner: Send + Sync {
    fn sign_record_hash(
        &self,
        university_id: UniversityId,
        record_hash: &str,
    ) -> Result<String, AppError>;
}

pub trait AtsKeyManager: Send + Sync {
    fn generate_api_key(&self) -> Result<String, AppError>;
    fn hash_api_key(&self, api_key: &str) -> Result<String, AppError>;
    fn key_prefix(&self, api_key: &str) -> String;
}

#[async_trait]
pub trait QrGateway: Send + Sync {
    async fn create_job(&self, payload: CreateQrJobPayload) -> Result<ExternalQrJob, AppError>;
    async fn get_job(&self, job_id: &str) -> Result<ExternalQrJob, AppError>;
    async fn get_qr(&self, qr_id: &str) -> Result<ExternalQrMetadata, AppError>;
    async fn get_qr_content(&self, qr_id: &str) -> Result<QrBinaryContent, AppError>;
    async fn delete_qr(&self, qr_id: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn is_ready(&self) -> bool;
}
