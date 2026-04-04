use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::{
    diploma::{Diploma, DiplomaStatus, DiplomaVerificationResult},
    ids::{CertificateId, DiplomaId, UniversityId, UserId},
    user::{User, UserRole},
};

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub student_number: Option<String>,
    pub role: UserRole,
    pub university_id: Option<UniversityId>,
    pub university_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: UserId,
    pub email: String,
    pub full_name: String,
    pub student_number: Option<String>,
    pub role: UserRole,
    pub university_id: Option<UniversityId>,
    pub university_code: Option<String>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            full_name: value.full_name,
            student_number: value.student_number,
            role: value.role,
            university_id: value.university_id,
            university_code: value.university_code,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in_seconds: i64,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterDiplomaRequest {
    pub student_full_name: String,
    pub student_number: String,
    pub student_birth_date: Option<NaiveDate>,
    pub diploma_number: String,
    pub degree: String,
    pub program_name: String,
    pub graduation_date: NaiveDate,
    #[serde(default)]
    pub honors: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterDiplomaResponse {
    pub diploma_id: DiplomaId,
    pub certificate_id: CertificateId,
    pub university_id: UniversityId,
    pub graduation_date: NaiveDate,
    pub diploma_number_last4: String,
    pub record_hash: String,
    pub university_signature: String,
    pub status: DiplomaStatus,
    pub storage_mode: &'static str,
}

impl From<Diploma> for RegisterDiplomaResponse {
    fn from(value: Diploma) -> Self {
        Self {
            diploma_id: value.id,
            certificate_id: value.certificate_id,
            university_id: value.university_id,
            graduation_date: value.issued_at,
            diploma_number_last4: value.diploma_number_last4,
            record_hash: value.record_hash,
            university_signature: value.university_signature,
            status: value.status,
            storage_mode: "hashed_only",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiplomaImportRowResult {
    pub row_number: usize,
    pub diploma_id: DiplomaId,
    pub certificate_id: CertificateId,
    pub record_hash: String,
    pub university_signature: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiplomaImportError {
    pub row_number: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiplomaImportResponse {
    pub imported_count: usize,
    pub failed_count: usize,
    pub imported: Vec<DiplomaImportRowResult>,
    pub errors: Vec<DiplomaImportError>,
}

#[derive(Debug, Clone)]
pub struct RegistryDiplomaRow {
    pub student_full_name: String,
    pub student_number: String,
    pub graduation_year: i32,
    pub program_name: String,
    pub diploma_number: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyDiplomaRequest {
    pub student_full_name: String,
    pub student_birth_date: Option<NaiveDate>,
    pub diploma_number: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyDiplomaResponse {
    pub found: bool,
    pub diploma_id: Option<DiplomaId>,
    pub certificate_id: Option<CertificateId>,
    pub status: Option<DiplomaStatus>,
}

impl From<DiplomaVerificationResult> for VerifyDiplomaResponse {
    fn from(value: DiplomaVerificationResult) -> Self {
        Self {
            found: value.found,
            diploma_id: value.diploma_id,
            certificate_id: value.certificate_id,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiplomaStatusResponse {
    pub diploma_id: DiplomaId,
    pub status: DiplomaStatus,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StudentDiplomaSearchRequest {
    pub diploma_number: Option<String>,
    pub student_full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentDiplomaCard {
    pub diploma_id: DiplomaId,
    pub certificate_id: CertificateId,
    pub university_id: UniversityId,
    pub university_code: String,
    pub student_number_last4: String,
    pub diploma_number_last4: String,
    pub graduation_date: NaiveDate,
    pub program_name_hash: String,
    pub record_hash: String,
    pub university_signature: String,
    pub status: DiplomaStatus,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Diploma> for StudentDiplomaCard {
    fn from(value: Diploma) -> Self {
        Self {
            diploma_id: value.id,
            certificate_id: value.certificate_id,
            university_id: value.university_id,
            university_code: value.university_code,
            student_number_last4: value.student_number_last4,
            diploma_number_last4: value.diploma_number_last4,
            graduation_date: value.issued_at,
            program_name_hash: value.hashed_payload.program_hash,
            record_hash: value.record_hash,
            university_signature: value.university_signature,
            status: value.status,
            revoked_at: value.revoked_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StudentDiplomaSearchResponse {
    pub items: Vec<StudentDiplomaCard>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiplomaShareLinkResponse {
    pub diploma_id: DiplomaId,
    pub expires_in_seconds: i64,
    pub access_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicDiplomaView {
    pub diploma_id: DiplomaId,
    pub certificate_id: CertificateId,
    pub university_id: UniversityId,
    pub university_code: String,
    pub student_number_last4: String,
    pub diploma_number_last4: String,
    pub graduation_date: NaiveDate,
    pub record_hash: String,
    pub university_signature: String,
    pub status: DiplomaStatus,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Diploma> for PublicDiplomaView {
    fn from(value: Diploma) -> Self {
        Self {
            diploma_id: value.id,
            certificate_id: value.certificate_id,
            university_id: value.university_id,
            university_code: value.university_code,
            student_number_last4: value.student_number_last4,
            diploma_number_last4: value.diploma_number_last4,
            graduation_date: value.issued_at,
            record_hash: value.record_hash,
            university_signature: value.university_signature,
            status: value.status,
            revoked_at: value.revoked_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HrRegistrySearchRequest {
    pub diploma_number: Option<String>,
    pub university_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HrRegistrySearchResponse {
    pub items: Vec<PublicDiplomaView>,
}
