use std::{sync::Arc, time::Duration};

use crate::{
    application::{
        dto::{
            DiplomaImportError, DiplomaImportResponse, DiplomaImportRowResult,
            DiplomaShareLinkResponse, HrRegistrySearchRequest, HrRegistrySearchResponse,
            PublicDiplomaView, RegisterDiplomaRequest, RegistryDiplomaRow,
            StudentDiplomaCard, StudentDiplomaSearchRequest, StudentDiplomaSearchResponse,
            VerifyDiplomaRequest,
        },
        ports::{AppRepository, DiplomaSigner, JwtProvider},
    },
    domain::{
        diploma::{CreateDiplomaPayload, Diploma, DiplomaVerificationQuery, DiplomaVerificationResult},
        hashing::DiplomaHasher,
        ids::{DiplomaId, UniversityId, UserId},
    },
    error::AppError,
    infrastructure::cache::ResponseCache,
};

use super::services_support::{
    intersect_or_replace, normalize_display_name, normalize_identifier, validate_request,
};

const DIPLOMA_CACHE_NAMESPACE: &str = "diploma_read_model";

pub struct DiplomaService {
    repository: Arc<dyn AppRepository>,
    hasher: Arc<dyn DiplomaHasher>,
    signer: Arc<dyn DiplomaSigner>,
    jwt_provider: Arc<dyn JwtProvider>,
    response_cache: Arc<dyn ResponseCache>,
    request_cache_ttl: Duration,
}

impl DiplomaService {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        hasher: Arc<dyn DiplomaHasher>,
        signer: Arc<dyn DiplomaSigner>,
        jwt_provider: Arc<dyn JwtProvider>,
        response_cache: Arc<dyn ResponseCache>,
        request_cache_ttl: Duration,
    ) -> Self {
        Self {
            repository,
            hasher,
            signer,
            jwt_provider,
            response_cache,
            request_cache_ttl,
        }
    }

    pub async fn register_diploma(
        &self,
        university_id: UniversityId,
        university_code: String,
        request: RegisterDiplomaRequest,
    ) -> Result<Diploma, AppError> {
        validate_request(&request)?;

        let payload = CreateDiplomaPayload {
            university_id,
            university_code: normalize_identifier(&university_code),
            student_full_name: normalize_display_name(&request.student_full_name),
            student_number: normalize_identifier(&request.student_number),
            student_birth_date: request.student_birth_date,
            diploma_number: normalize_identifier(&request.diploma_number),
            degree: normalize_display_name(&request.degree),
            program_name: normalize_display_name(&request.program_name),
            graduation_date: request.graduation_date,
            honors: request.honors,
        };

        self.store_signed_diploma(payload).await
    }

    pub async fn import_registry(
        &self,
        university_id: UniversityId,
        university_code: String,
        rows: Vec<RegistryDiplomaRow>,
    ) -> DiplomaImportResponse {
        let mut imported = Vec::new();
        let mut errors = Vec::new();

        for (index, row) in rows.into_iter().enumerate() {
            let row_number = index + 1;
            match self
                .store_signed_diploma(CreateDiplomaPayload {
                    university_id,
                    university_code: normalize_identifier(&university_code),
                    student_full_name: normalize_display_name(&row.student_full_name),
                    student_number: normalize_identifier(&row.student_number),
                    student_birth_date: None,
                    diploma_number: normalize_identifier(&row.diploma_number),
                    degree: "registry_import".to_string(),
                    program_name: normalize_display_name(&row.program_name),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(row.graduation_year, 1, 1)
                        .unwrap_or(chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                    honors: false,
                })
                .await
            {
                Ok(diploma) => imported.push(DiplomaImportRowResult {
                    row_number,
                    diploma_id: diploma.id,
                    certificate_id: diploma.certificate_id,
                    record_hash: diploma.record_hash,
                    university_signature: diploma.university_signature,
                }),
                Err(error) => errors.push(DiplomaImportError {
                    row_number,
                    message: error.to_string(),
                }),
            }
        }

        DiplomaImportResponse {
            imported_count: imported.len(),
            failed_count: errors.len(),
            imported,
            errors,
        }
    }

    pub async fn verify_diploma(
        &self,
        request: VerifyDiplomaRequest,
    ) -> Result<DiplomaVerificationResult, AppError> {
        let query = DiplomaVerificationQuery {
            student_full_name: request.student_full_name,
            student_birth_date: request.student_birth_date,
            diploma_number: request.diploma_number,
        };
        let canonical_hash = self.hasher.hash_verification_query(&query)?;
        let cache_key = self
            .versioned_cache_key("verify_diploma", &canonical_hash)
            .await;

        if let Some(cached) = self.get_cached_json(&cache_key).await {
            return Ok(cached);
        }

        let diploma = self.repository.find_by_canonical_hash(&canonical_hash).await?;
        let result = match diploma {
            Some(diploma) => DiplomaVerificationResult {
                found: true,
                diploma_id: Some(diploma.id),
                certificate_id: Some(diploma.certificate_id),
                status: Some(diploma.status),
            },
            None => DiplomaVerificationResult {
                found: false,
                diploma_id: None,
                certificate_id: None,
                status: None,
            },
        };

        self.set_cached_json(&cache_key, &result).await;
        Ok(result)
    }

    pub async fn list_student_diplomas(&self, student_id: UserId) -> Result<Vec<Diploma>, AppError> {
        self.repository.find_by_student_id(student_id).await
    }

    pub async fn search_student_diplomas(
        &self,
        student_user_id: UserId,
        request: StudentDiplomaSearchRequest,
    ) -> Result<StudentDiplomaSearchResponse, AppError> {
        let has_diploma_number = request
            .diploma_number
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        let has_full_name = request
            .student_full_name
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());

        if !has_diploma_number && !has_full_name {
            return Err(AppError::Validation(
                "provide diploma_number or student_full_name".into(),
            ));
        }

        let student = self
            .repository
            .find_user_by_id(student_user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        let student_number = student
            .student_number
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AppError::Validation("student account must contain student_number".into()))?;
        let student_name_hash = self.hasher.hash_student_name_lookup(&student.full_name)?;
        let student_number_hash = self.hasher.hash_student_number_lookup(student_number)?;

        let mut items = None;

        if let Some(diploma_number) = request.diploma_number.as_deref() {
            if !diploma_number.trim().is_empty() {
                let diploma_hash = self.hasher.hash_diploma_number_lookup(diploma_number)?;
                let matches = self
                    .repository
                    .search_by_diploma_number_hash(&diploma_hash)
                    .await?;
                items = Some(matches);
            }
        }

        if let Some(full_name) = request.student_full_name.as_deref() {
            if !full_name.trim().is_empty() {
                let full_name_hash = self.hasher.hash_student_name_lookup(full_name)?;
                let matches = self
                    .repository
                    .search_by_student_name_hash(&full_name_hash)
                    .await?;
                items = Some(intersect_or_replace(items, matches));
            }
        }

        let items = items.unwrap_or_default();
        let mut owned = Vec::new();
        for mut item in items {
            if item.hashed_payload.student_full_name_hash != student_name_hash
                || item.hashed_payload.student_number_hash != student_number_hash
            {
                continue;
            }

            if let Some(owner_id) = item.student_account_id {
                if owner_id != student_user_id {
                    continue;
                }
            } else {
                item.assign_student(student_user_id);
                item = self.repository.update(item).await?;
            }

            owned.push(StudentDiplomaCard::from(item));
        }

        Ok(StudentDiplomaSearchResponse { items: owned })
    }

    pub async fn generate_diploma_share_link(
        &self,
        student_user_id: UserId,
        diploma_id: DiplomaId,
        base_url: &str,
        ttl_minutes: i64,
    ) -> Result<DiplomaShareLinkResponse, AppError> {
        let diploma = self
            .repository
            .find_by_id(diploma_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if diploma.student_account_id != Some(student_user_id) {
            return Err(AppError::Forbidden(
                "student can share only their own diploma".into(),
            ));
        }

        let token = self
            .jwt_provider
            .issue_diploma_access_token(diploma_id, ttl_minutes)?;

        Ok(DiplomaShareLinkResponse {
            diploma_id,
            expires_in_seconds: ttl_minutes * 60,
            access_url: format!(
                "{}/api/v1/public/diplomas/access/{}",
                base_url.trim_end_matches('/'),
                token
            ),
        })
    }

    pub async fn resolve_public_diploma_view(
        &self,
        token: &str,
    ) -> Result<PublicDiplomaView, AppError> {
        let claims = self.jwt_provider.decode_diploma_access_token(token)?;
        let diploma_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
        let cache_key = self
            .versioned_cache_key("public_diploma_view", &diploma_id.to_string())
            .await;

        if let Some(cached) = self.get_cached_json(&cache_key).await {
            return Ok(cached);
        }

        let diploma = self
            .repository
            .find_by_id(DiplomaId(diploma_id))
            .await?
            .ok_or(AppError::NotFound)?;
        let result = PublicDiplomaView::from(diploma);
        self.set_cached_json(&cache_key, &result).await;
        Ok(result)
    }

    pub async fn search_hr_registry(
        &self,
        request: HrRegistrySearchRequest,
    ) -> Result<HrRegistrySearchResponse, AppError> {
        let has_diploma_number = request
            .diploma_number
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        let has_university_code = request
            .university_code
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());

        if !has_diploma_number && !has_university_code {
            return Err(AppError::Validation(
                "provide diploma_number or university_code".into(),
            ));
        }

        let diploma_hash = if let Some(diploma_number) = request.diploma_number.as_deref() {
            if diploma_number.trim().is_empty() {
                None
            } else {
                Some(self.hasher.hash_diploma_number_lookup(diploma_number)?)
            }
        } else {
            None
        };
        let university_code_hash = if let Some(university_code) = request.university_code.as_deref() {
            if university_code.trim().is_empty() {
                None
            } else {
                Some(self.hasher.hash_university_code_lookup(university_code)?)
            }
        } else {
            None
        };
        let mut cache_fingerprint = Vec::new();
        if let Some(diploma_hash) = diploma_hash.as_ref() {
            cache_fingerprint.push(format!("diploma:{diploma_hash}"));
        }
        if let Some(university_code_hash) = university_code_hash.as_ref() {
            cache_fingerprint.push(format!("university:{university_code_hash}"));
        }
        let cache_key = self
            .versioned_cache_key("hr_registry_search", &cache_fingerprint.join("|"))
            .await;

        if let Some(cached) = self.get_cached_json(&cache_key).await {
            return Ok(cached);
        }

        let mut items = None;

        if let Some(diploma_hash) = diploma_hash.as_ref() {
            let matches = self
                .repository
                .search_by_diploma_number_hash(diploma_hash)
                .await?;
            items = Some(matches);
        }

        if let Some(university_code_hash) = university_code_hash.as_ref() {
            let matches = self
                .repository
                .search_by_university_code_hash(university_code_hash)
                .await?;
            items = Some(intersect_or_replace(items, matches));
        }

        let result = HrRegistrySearchResponse {
            items: items
                .unwrap_or_default()
                .into_iter()
                .map(PublicDiplomaView::from)
                .collect(),
        };

        self.set_cached_json(&cache_key, &result).await;
        Ok(result)
    }

    pub async fn revoke_diploma(
        &self,
        university_id: UniversityId,
        diploma_id: DiplomaId,
    ) -> Result<Diploma, AppError> {
        let mut diploma = self
            .repository
            .find_by_id(diploma_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if diploma.university_id != university_id {
            return Err(AppError::Forbidden(
                "cannot modify diploma from another university".into(),
            ));
        }

        diploma.revoke();
        let updated = self.repository.update(diploma).await?;
        self.invalidate_diploma_cache().await;
        Ok(updated)
    }

    pub async fn restore_diploma(
        &self,
        university_id: UniversityId,
        diploma_id: DiplomaId,
    ) -> Result<Diploma, AppError> {
        let mut diploma = self
            .repository
            .find_by_id(diploma_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if diploma.university_id != university_id {
            return Err(AppError::Forbidden(
                "cannot modify diploma from another university".into(),
            ));
        }

        diploma.restore();
        let updated = self.repository.update(diploma).await?;
        self.invalidate_diploma_cache().await;
        Ok(updated)
    }

    async fn store_signed_diploma(&self, payload: CreateDiplomaPayload) -> Result<Diploma, AppError> {
        let hashed_payload = self.hasher.hash_payload(&payload)?;
        let record_hash = hashed_payload.canonical_document_hash.clone();
        let university_signature = self
            .signer
            .sign_record_hash(payload.university_id, &record_hash)?;
        let diploma = Diploma::from_payload(payload, hashed_payload, record_hash, university_signature);
        let saved = self.repository.save(diploma).await?;
        self.invalidate_diploma_cache().await;
        Ok(saved)
    }

    async fn invalidate_diploma_cache(&self) {
        self.response_cache
            .bump_namespace(DIPLOMA_CACHE_NAMESPACE)
            .await;
    }

    async fn versioned_cache_key(&self, scope: &str, identity: &str) -> String {
        let version = self
            .response_cache
            .namespace_version(DIPLOMA_CACHE_NAMESPACE)
            .await;
        format!("{scope}:v{version}:{identity}")
    }

    async fn get_cached_json<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let cached = self.response_cache.get(key).await?;
        serde_json::from_str(&cached).ok()
    }

    async fn set_cached_json<T>(&self, key: &str, value: &T)
    where
        T: serde::Serialize,
    {
        let Ok(serialized) = serde_json::to_string(value) else {
            return;
        };

        self.response_cache
            .set(key, &serialized, self.request_cache_ttl)
            .await;
    }
}
