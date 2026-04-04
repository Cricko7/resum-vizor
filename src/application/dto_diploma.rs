use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{
    diploma::{Diploma, DiplomaStatus, DiplomaVerificationResult},
    ids::{CertificateId, DiplomaId, UniversityId},
};

#[derive(Debug, Clone, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiplomaImportRowResult {
    pub row_number: usize,
    pub diploma_id: DiplomaId,
    pub certificate_id: CertificateId,
    pub record_hash: String,
    pub university_signature: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiplomaImportError {
    pub row_number: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct VerifyDiplomaRequest {
    pub student_full_name: String,
    pub student_birth_date: Option<NaiveDate>,
    pub diploma_number: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiplomaStatusResponse {
    pub diploma_id: DiplomaId,
    pub status: DiplomaStatus,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
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
