use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{
    hashing::HashedDiplomaPayload,
    ids::{CertificateId, DiplomaId, StudentId, UniversityId, UserId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDiplomaPayload {
    pub university_id: UniversityId,
    pub university_code: String,
    pub student_full_name: String,
    pub student_number: String,
    pub student_birth_date: Option<NaiveDate>,
    pub diploma_number: String,
    pub degree: String,
    pub program_name: String,
    pub graduation_date: NaiveDate,
    pub honors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diploma {
    pub id: DiplomaId,
    pub university_id: UniversityId,
    pub student_id: StudentId,
    pub certificate_id: CertificateId,
    pub student_account_id: Option<UserId>,
    pub university_code: String,
    pub student_number_last4: String,
    pub diploma_number_last4: String,
    pub record_hash: String,
    pub university_signature: String,
    pub signature_algorithm: &'static str,
    pub status: DiplomaStatus,
    pub revoked_at: Option<DateTime<Utc>>,
    pub hashed_payload: HashedDiplomaPayload,
    pub issued_at: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaStatus {
    Active,
    Revoked,
}

impl Diploma {
    pub fn from_payload(
        payload: CreateDiplomaPayload,
        hashed_payload: HashedDiplomaPayload,
        record_hash: String,
        university_signature: String,
    ) -> Self {
        let diploma_number_last4 = payload
            .diploma_number
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let student_number_last4 = payload
            .student_number
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        Self {
            id: DiplomaId::new(),
            university_id: payload.university_id,
            student_id: StudentId::new(),
            certificate_id: CertificateId::new(),
            student_account_id: None,
            university_code: payload.university_code,
            student_number_last4,
            diploma_number_last4,
            record_hash,
            university_signature,
            signature_algorithm: "ed25519",
            status: DiplomaStatus::Active,
            revoked_at: None,
            hashed_payload,
            issued_at: payload.graduation_date,
            created_at: Utc::now(),
        }
    }

    pub fn revoke(&mut self) {
        self.status = DiplomaStatus::Revoked;
        self.revoked_at = Some(Utc::now());
    }

    pub fn restore(&mut self) {
        self.status = DiplomaStatus::Active;
        self.revoked_at = None;
    }

    pub fn assign_student(&mut self, user_id: UserId) {
        self.student_account_id = Some(user_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaVerificationQuery {
    pub student_full_name: String,
    pub student_birth_date: Option<NaiveDate>,
    pub diploma_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaVerificationResult {
    pub found: bool,
    pub diploma_id: Option<DiplomaId>,
    pub certificate_id: Option<CertificateId>,
    pub status: Option<DiplomaStatus>,
}
