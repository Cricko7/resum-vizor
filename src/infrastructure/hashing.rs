use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    domain::{
        diploma::{CreateDiplomaPayload, DiplomaVerificationQuery},
        hashing::{DiplomaHasher, HashedDiplomaPayload},
    },
    error::AppError,
};

pub struct Blake3DiplomaHasher {
    secret: SecretString,
}

impl Blake3DiplomaHasher {
    pub fn new(secret: SecretString) -> Self {
        Self { secret }
    }

    fn derive_key(&self) -> [u8; 32] {
        *blake3::hash(self.secret.expose_secret().as_bytes()).as_bytes()
    }

    fn hash_field(&self, field_name: &str, value: &str) -> String {
        let key = self.derive_key();
        let canonical = format!("{field_name}:{value}");
        let digest = blake3::keyed_hash(&key, canonical.as_bytes());
        URL_SAFE_NO_PAD.encode(digest.as_bytes())
    }
}

impl DiplomaHasher for Blake3DiplomaHasher {
    fn hash_payload(&self, payload: &CreateDiplomaPayload) -> Result<HashedDiplomaPayload, AppError> {
        let student_full_name = normalize_text(&payload.student_full_name);
        let student_number = normalize_identifier(&payload.student_number);
        let university_code = normalize_identifier(&payload.university_code);
        let diploma_number = normalize_identifier(&payload.diploma_number);
        let degree = normalize_text(&payload.degree);
        let program_name = normalize_text(&payload.program_name);
        let birth_date = payload
            .student_birth_date
            .map(|date| date.format("%Y-%m-%d").to_string());
        let graduation_date = payload.graduation_date.format("%Y-%m-%d").to_string();
        let honors = if payload.honors { "true" } else { "false" };

        let mut canonical_document = vec![
            format!("university_id={}", payload.university_id.0),
            format!("university_code={university_code}"),
            format!("student_full_name={student_full_name}"),
            format!("student_number={student_number}"),
            format!("diploma_number={diploma_number}"),
            format!("degree={degree}"),
            format!("program_name={program_name}"),
            format!("graduation_date={graduation_date}"),
            format!("honors={honors}"),
        ];
        canonical_document.push(format!(
            "student_birth_date={}",
            birth_date.clone().unwrap_or_default()
        ));

        let mut verification_document = vec![
            format!("student_full_name={student_full_name}"),
            format!("diploma_number={diploma_number}"),
        ];
        verification_document.push(format!(
            "student_birth_date={}",
            birth_date.clone().unwrap_or_default()
        ));

        Ok(HashedDiplomaPayload {
            university_code_hash: self.hash_field("university_code", &university_code),
            student_full_name_hash: self.hash_field("student_full_name", &student_full_name),
            student_number_hash: self.hash_field("student_number", &student_number),
            student_birth_date_hash: birth_date
                .as_ref()
                .map(|value| self.hash_field("student_birth_date", value)),
            diploma_number_hash: self.hash_field("diploma_number", &diploma_number),
            verification_lookup_hash: self.hash_field(
                "verification_document",
                &verification_document.join("|"),
            ),
            degree_hash: self.hash_field("degree", &degree),
            program_hash: self.hash_field("program_name", &program_name),
            graduation_date_hash: self.hash_field("graduation_date", &graduation_date),
            honors_hash: self.hash_field("honors", honors),
            canonical_document_hash: self.hash_field("canonical_document", &canonical_document.join("|")),
        })
    }

    fn hash_verification_query(&self, query: &DiplomaVerificationQuery) -> Result<String, AppError> {
        let student_full_name = normalize_text(&query.student_full_name);
        let diploma_number = normalize_identifier(&query.diploma_number);
        let birth_date = query
            .student_birth_date
            .map(|date| date.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        let canonical_document = [
            format!("student_full_name={student_full_name}"),
            format!("student_birth_date={birth_date}"),
            format!("diploma_number={diploma_number}"),
        ]
        .join("|");

        Ok(self.hash_field("verification_document", &canonical_document))
    }

    fn hash_student_name_lookup(&self, full_name: &str) -> Result<String, AppError> {
        Ok(self.hash_field("student_full_name", &normalize_text(full_name)))
    }

    fn hash_student_number_lookup(&self, student_number: &str) -> Result<String, AppError> {
        Ok(self.hash_field(
            "student_number",
            &normalize_identifier(student_number),
        ))
    }

    fn hash_diploma_number_lookup(&self, diploma_number: &str) -> Result<String, AppError> {
        Ok(self.hash_field(
            "diploma_number",
            &normalize_identifier(diploma_number),
        ))
    }

    fn hash_university_code_lookup(&self, university_code: &str) -> Result<String, AppError> {
        Ok(self.hash_field(
            "university_code",
            &normalize_identifier(university_code),
        ))
    }
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn normalize_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|char| char.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase()
}
