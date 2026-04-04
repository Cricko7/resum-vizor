use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{
    diploma::{Diploma, DiplomaStatus},
    ids::{CertificateId, DiplomaId, UniversityId},
};

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StudentDiplomaSearchRequest {
    pub diploma_number: Option<String>,
    pub student_full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StudentDiplomaCard {
    pub diploma_id: DiplomaId,
    pub certificate_id: CertificateId,
    pub university_id: UniversityId,
    pub university_code: String,
    pub student_number_last4: String,
    pub diploma_number_last4: String,
    pub graduation_date: chrono::NaiveDate,
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StudentDiplomaSearchResponse {
    pub items: Vec<StudentDiplomaCard>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiplomaShareLinkResponse {
    pub diploma_id: DiplomaId,
    pub expires_in_seconds: i64,
    pub access_url: String,
}
