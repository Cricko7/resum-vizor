use async_trait::async_trait;

use crate::{
    domain::{
        diploma::Diploma,
        ids::{CertificateId, DiplomaId, UniversityId, UserId},
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

pub trait AppRepository: DiplomaRepository + UserRepository {}

impl<T> AppRepository for T where T: DiplomaRepository + UserRepository + ?Sized {}

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

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn is_ready(&self) -> bool;
}
