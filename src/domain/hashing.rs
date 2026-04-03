use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedDiplomaPayload {
    pub university_code_hash: String,
    pub student_full_name_hash: String,
    pub student_number_hash: String,
    pub student_birth_date_hash: Option<String>,
    pub diploma_number_hash: String,
    pub verification_lookup_hash: String,
    pub degree_hash: String,
    pub program_hash: String,
    pub graduation_date_hash: String,
    pub honors_hash: String,
    pub canonical_document_hash: String,
}

pub trait DiplomaHasher: Send + Sync {
    fn hash_payload(
        &self,
        payload: &crate::domain::diploma::CreateDiplomaPayload,
    ) -> Result<HashedDiplomaPayload, AppError>;
    fn hash_verification_query(
        &self,
        query: &crate::domain::diploma::DiplomaVerificationQuery,
    ) -> Result<String, AppError>;
    fn hash_student_name_lookup(&self, full_name: &str) -> Result<String, AppError>;
    fn hash_student_number_lookup(&self, student_number: &str) -> Result<String, AppError>;
    fn hash_diploma_number_lookup(&self, diploma_number: &str) -> Result<String, AppError>;
    fn hash_university_code_lookup(&self, university_code: &str) -> Result<String, AppError>;
}
