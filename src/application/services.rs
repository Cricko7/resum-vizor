use std::sync::Arc;

use crate::{
    application::{
        dto::{
            AuthResponse, ChangePasswordRequest, DiplomaImportError, DiplomaImportResponse,
            DiplomaImportRowResult, DiplomaShareLinkResponse, LoginRequest,
            PublicDiplomaView, RegisterDiplomaRequest, RegisterUserRequest, RegistryDiplomaRow,
            StudentDiplomaCard, StudentDiplomaSearchRequest, StudentDiplomaSearchResponse,
        },
        ports::{AppRepository, DiplomaSigner, JwtProvider, PasswordHasher},
    },
    domain::{
        diploma::{CreateDiplomaPayload, Diploma, DiplomaVerificationQuery, DiplomaVerificationResult},
        hashing::DiplomaHasher,
        ids::{UniversityId, UserId},
        user::{User, UserRole},
    },
    error::AppError,
};

pub struct AuthService {
    repository: Arc<dyn AppRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
    jwt_provider: Arc<dyn JwtProvider>,
    jwt_ttl_minutes: i64,
}

impl AuthService {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        jwt_provider: Arc<dyn JwtProvider>,
        jwt_ttl_minutes: i64,
    ) -> Self {
        Self {
            repository,
            password_hasher,
            jwt_provider,
            jwt_ttl_minutes,
        }
    }

    pub async fn register(&self, request: RegisterUserRequest) -> Result<AuthResponse, AppError> {
        validate_registration(&request)?;

        if self
            .repository
            .find_user_by_email(&normalize_email(&request.email))
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("user with this email already exists".into()));
        }

        if request.role == UserRole::University && request.university_id.is_none() {
            return Err(AppError::Validation(
                "university_id is required for university role".into(),
            ));
        }

        if request.role == UserRole::University
            && request
                .university_code
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(AppError::Validation(
                "university_code is required for university role".into(),
            ));
        }

        if request.role == UserRole::Student
            && request
                .student_number
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(AppError::Validation(
                "student_number is required for student role".into(),
            ));
        }

        let student_number = if request.role == UserRole::Student {
            request
                .student_number
                .as_deref()
                .map(normalize_identifier)
        } else {
            None
        };
        let university_id = if request.role == UserRole::University {
            request.university_id
        } else {
            None
        };
        let university_code = if request.role == UserRole::University {
            request
                .university_code
                .as_deref()
                .map(normalize_identifier)
        } else {
            None
        };

        let user = User::new(
            normalize_email(&request.email),
            self.password_hasher.hash_password(&request.password)?,
            normalize_display_name(&request.full_name),
            student_number,
            request.role,
            university_id,
            university_code,
        );
        let user = self.repository.create_user(user).await?;
        self.build_auth_response(user)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        let email = normalize_email(&request.email);
        let user = self
            .repository
            .find_user_by_email(&email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        self.password_hasher
            .verify_password(&request.password, &user.password_hash)?;

        self.build_auth_response(user)
    }

    pub async fn change_password(
        &self,
        user_id: UserId,
        request: ChangePasswordRequest,
    ) -> Result<(), AppError> {
        validate_new_password(&request.new_password)?;

        if request.current_password == request.new_password {
            return Err(AppError::Validation(
                "new_password must be different from current_password".into(),
            ));
        }

        let mut user = self
            .repository
            .find_user_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        self.password_hasher
            .verify_password(&request.current_password, &user.password_hash)?;

        let new_hash = self.password_hasher.hash_password(&request.new_password)?;
        user.update_password(new_hash);
        self.repository.update_user(user).await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: UserId) -> Result<User, AppError> {
        self.repository
            .find_user_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)
    }

    fn build_auth_response(&self, user: User) -> Result<AuthResponse, AppError> {
        let access_token = self.jwt_provider.issue_token(&user)?;

        Ok(AuthResponse {
            access_token,
            token_type: "Bearer",
            expires_in_seconds: self.jwt_ttl_minutes * 60,
            user: user.into(),
        })
    }
}

pub struct DiplomaService {
    repository: Arc<dyn AppRepository>,
    hasher: Arc<dyn DiplomaHasher>,
    signer: Arc<dyn DiplomaSigner>,
    jwt_provider: Arc<dyn JwtProvider>,
}

impl DiplomaService {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        hasher: Arc<dyn DiplomaHasher>,
        signer: Arc<dyn DiplomaSigner>,
        jwt_provider: Arc<dyn JwtProvider>,
    ) -> Self {
        Self {
            repository,
            hasher,
            signer,
            jwt_provider,
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
        request: crate::application::dto::VerifyDiplomaRequest,
    ) -> Result<DiplomaVerificationResult, AppError> {
        let query = DiplomaVerificationQuery {
            student_full_name: request.student_full_name,
            student_birth_date: request.student_birth_date,
            diploma_number: request.diploma_number,
        };
        let canonical_hash = self.hasher.hash_verification_query(&query)?;
        let diploma = self.repository.find_by_canonical_hash(&canonical_hash).await?;

        Ok(match diploma {
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
        })
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
        diploma_id: crate::domain::ids::DiplomaId,
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
            access_url: format!("{}/api/v1/public/diplomas/access/{}", base_url.trim_end_matches('/'), token),
        })
    }

    pub async fn resolve_public_diploma_view(
        &self,
        token: &str,
    ) -> Result<PublicDiplomaView, AppError> {
        let claims = self.jwt_provider.decode_diploma_access_token(token)?;
        let diploma_id =
            uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
        let diploma = self
            .repository
            .find_by_id(crate::domain::ids::DiplomaId(diploma_id))
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(diploma.into())
    }

    pub async fn search_hr_registry(
        &self,
        request: crate::application::dto::HrRegistrySearchRequest,
    ) -> Result<crate::application::dto::HrRegistrySearchResponse, AppError> {
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

        if let Some(university_code) = request.university_code.as_deref() {
            if !university_code.trim().is_empty() {
                let university_code_hash = self.hasher.hash_university_code_lookup(university_code)?;
                let matches = self
                    .repository
                    .search_by_university_code_hash(&university_code_hash)
                    .await?;
                items = Some(intersect_or_replace(items, matches));
            }
        }

        Ok(crate::application::dto::HrRegistrySearchResponse {
            items: items
                .unwrap_or_default()
                .into_iter()
                .map(PublicDiplomaView::from)
                .collect(),
        })
    }

    pub async fn revoke_diploma(
        &self,
        university_id: UniversityId,
        diploma_id: crate::domain::ids::DiplomaId,
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
        self.repository.update(diploma).await
    }

    pub async fn restore_diploma(
        &self,
        university_id: UniversityId,
        diploma_id: crate::domain::ids::DiplomaId,
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
        self.repository.update(diploma).await
    }

    async fn store_signed_diploma(&self, payload: CreateDiplomaPayload) -> Result<Diploma, AppError> {
        let hashed_payload = self.hasher.hash_payload(&payload)?;
        let record_hash = hashed_payload.canonical_document_hash.clone();
        let university_signature = self
            .signer
            .sign_record_hash(payload.university_id, &record_hash)?;
        let diploma = Diploma::from_payload(payload, hashed_payload, record_hash, university_signature);

        self.repository.save(diploma).await
    }
}

fn validate_request(request: &RegisterDiplomaRequest) -> Result<(), AppError> {
    if request.student_full_name.trim().is_empty() {
        return Err(AppError::Validation("student_full_name is required".into()));
    }

    if request.student_number.trim().is_empty() {
        return Err(AppError::Validation("student_number is required".into()));
    }

    if request.diploma_number.trim().is_empty() {
        return Err(AppError::Validation("diploma_number is required".into()));
    }

    if request.degree.trim().is_empty() {
        return Err(AppError::Validation("degree is required".into()));
    }

    if request.program_name.trim().is_empty() {
        return Err(AppError::Validation("program_name is required".into()));
    }

    Ok(())
}

fn validate_registration(request: &RegisterUserRequest) -> Result<(), AppError> {
    if !request.email.contains('@') {
        return Err(AppError::Validation("email must be valid".into()));
    }

    if request.full_name.trim().is_empty() {
        return Err(AppError::Validation("full_name is required".into()));
    }

    validate_new_password(&request.password)
}

fn validate_new_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters long".into(),
        ));
    }

    Ok(())
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn normalize_display_name(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_identifier(value: &str) -> String {
    value.trim().to_string()
}

fn intersect_or_replace(existing: Option<Vec<Diploma>>, matches: Vec<Diploma>) -> Vec<Diploma> {
    match existing {
        Some(current) => current
            .into_iter()
            .filter(|item| matches.iter().any(|candidate| candidate.id == item.id))
            .collect(),
        None => matches,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use secrecy::SecretString;

    use crate::{
        application::{
            dto::{
                HrRegistrySearchRequest, LoginRequest, RegisterDiplomaRequest, RegisterUserRequest,
                StudentDiplomaSearchRequest,
            },
            ports::AppRepository,
            services::{AuthService, DiplomaService},
        },
        domain::{
            ids::UniversityId,
            user::UserRole,
        },
        error::AppError,
        infrastructure::{
            auth::{ArgonPasswordHasher, JwtService},
            hashing::Blake3DiplomaHasher,
            persistence::in_memory::InMemoryAppRepository,
            signing::UniversityRecordSigner,
        },
    };

    fn secret(value: &str) -> SecretString {
        SecretString::new(value.to_string().into_boxed_str())
    }

    fn build_services() -> (AuthService, DiplomaService, Arc<dyn AppRepository>) {
        let repository: Arc<dyn AppRepository> = Arc::new(InMemoryAppRepository::default());
        let jwt_provider = Arc::new(JwtService::new(&secret("jwt-secret"), 30));
        let auth_service = AuthService::new(
            repository.clone(),
            Arc::new(ArgonPasswordHasher),
            jwt_provider.clone(),
            30,
        );
        let diploma_service = DiplomaService::new(
            repository.clone(),
            Arc::new(Blake3DiplomaHasher::new(secret("hash-secret"))),
            Arc::new(UniversityRecordSigner::new(&secret("sign-secret"))),
            jwt_provider,
        );

        (auth_service, diploma_service, repository)
    }

    #[tokio::test]
    async fn student_registration_requires_student_number() {
        let (auth_service, _, _) = build_services();

        let result = auth_service
            .register(RegisterUserRequest {
                email: "student@example.com".into(),
                password: "superpass".into(),
                full_name: "Ivan Petrov".into(),
                student_number: None,
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await;

        assert!(matches!(result, Err(AppError::Validation(message)) if message.contains("student_number")));
    }

    #[tokio::test]
    async fn search_claims_diploma_only_when_name_and_student_number_match() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        let university = auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-001".into()),
            })
            .await
            .expect("university registration should succeed");

        let _ = university;

        let diploma = diploma_service
            .register_diploma(
                university_id,
                "UNI-001".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Ivan Petrov".into(),
                    student_number: "ST-1001".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0001".into(),
                    degree: "bachelor".into(),
                    program_name: "computer science".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                    honors: false,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let matching_student = auth_service
            .register(RegisterUserRequest {
                email: "ivan@example.com".into(),
                password: "superpass".into(),
                full_name: "Ivan Petrov".into(),
                student_number: Some("ST-1001".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("student registration should succeed");

        let other_student = auth_service
            .register(RegisterUserRequest {
                email: "other@example.com".into(),
                password: "superpass".into(),
                full_name: "Ivan Petrov".into(),
                student_number: Some("ST-9999".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("other student registration should succeed");

        let found = diploma_service
            .search_student_diplomas(
                matching_student.user.id,
                StudentDiplomaSearchRequest {
                    diploma_number: Some("DP-2026-0001".into()),
                    student_full_name: Some("Ivan Petrov".into()),
                },
            )
            .await
            .expect("search should succeed");

        assert_eq!(found.items.len(), 1);
        assert_eq!(found.items[0].diploma_id, diploma.id);

        let not_found = diploma_service
            .search_student_diplomas(
                other_student.user.id,
                StudentDiplomaSearchRequest {
                    diploma_number: Some("DP-2026-0001".into()),
                    student_full_name: Some("Ivan Petrov".into()),
                },
            )
            .await
            .expect("search should succeed");

        assert!(not_found.items.is_empty());
    }

    #[tokio::test]
    async fn share_link_is_allowed_only_for_owner_student() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-001".into()),
            })
            .await
            .expect("university registration should succeed");

        let diploma = diploma_service
            .register_diploma(
                university_id,
                "UNI-001".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Ivan Petrov".into(),
                    student_number: "ST-1001".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0002".into(),
                    degree: "bachelor".into(),
                    program_name: "computer science".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                    honors: true,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let owner = auth_service
            .register(RegisterUserRequest {
                email: "owner@example.com".into(),
                password: "superpass".into(),
                full_name: "Ivan Petrov".into(),
                student_number: Some("ST-1001".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("owner registration should succeed");

        let intruder = auth_service
            .register(RegisterUserRequest {
                email: "intruder@example.com".into(),
                password: "superpass".into(),
                full_name: "Petr Ivanov".into(),
                student_number: Some("ST-2002".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("intruder registration should succeed");

        diploma_service
            .search_student_diplomas(
                owner.user.id,
                StudentDiplomaSearchRequest {
                    diploma_number: Some("DP-2026-0002".into()),
                    student_full_name: Some("Ivan Petrov".into()),
                },
            )
            .await
            .expect("owner should claim diploma");

        let share = diploma_service
            .generate_diploma_share_link(owner.user.id, diploma.id, "http://localhost:8080", 15)
            .await
            .expect("owner should generate share link");

        assert!(share.access_url.contains("/api/v1/public/diplomas/access/"));

        let intruder_result = diploma_service
            .generate_diploma_share_link(
                intruder.user.id,
                diploma.id,
                "http://localhost:8080",
                15,
            )
            .await;

        assert!(matches!(intruder_result, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn share_link_can_be_resolved_back_to_public_view() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-001".into()),
            })
            .await
            .expect("university registration should succeed");

        let diploma = diploma_service
            .register_diploma(
                university_id,
                "UNI-001".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Maria Sidorova".into(),
                    student_number: "ST-2222".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0003".into(),
                    degree: "master".into(),
                    program_name: "data science".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                    honors: false,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let student = auth_service
            .register(RegisterUserRequest {
                email: "maria@example.com".into(),
                password: "superpass".into(),
                full_name: "Maria Sidorova".into(),
                student_number: Some("ST-2222".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("student registration should succeed");

        diploma_service
            .search_student_diplomas(
                student.user.id,
                StudentDiplomaSearchRequest {
                    diploma_number: Some("DP-2026-0003".into()),
                    student_full_name: Some("Maria Sidorova".into()),
                },
            )
            .await
            .expect("search should claim diploma");

        let share = diploma_service
            .generate_diploma_share_link(student.user.id, diploma.id, "http://localhost:8080", 15)
            .await
            .expect("share link should be generated");
        let token = share
            .access_url
            .rsplit('/')
            .next()
            .expect("token should exist in url");

        let public_view = diploma_service
            .resolve_public_diploma_view(token)
            .await
            .expect("public view should resolve");

        assert_eq!(public_view.diploma_id, diploma.id);
        assert_eq!(public_view.university_code, "UNI-001");
    }

    #[tokio::test]
    async fn revoke_and_restore_change_diploma_status() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-001".into()),
            })
            .await
            .expect("university registration should succeed");

        let diploma = diploma_service
            .register_diploma(
                university_id,
                "UNI-001".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Oleg Smirnov".into(),
                    student_number: "ST-3333".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0004".into(),
                    degree: "bachelor".into(),
                    program_name: "economics".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                    honors: false,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let revoked = diploma_service
            .revoke_diploma(university_id, diploma.id)
            .await
            .expect("revoke should succeed");
        assert_eq!(format!("{:?}", revoked.status), "Revoked");
        assert!(revoked.revoked_at.is_some());

        let restored = diploma_service
            .restore_diploma(university_id, diploma.id)
            .await
            .expect("restore should succeed");
        assert_eq!(format!("{:?}", restored.status), "Active");
        assert!(restored.revoked_at.is_none());
    }

    #[tokio::test]
    async fn hr_search_finds_by_university_code_and_diploma_number() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-777".into()),
            })
            .await
            .expect("university registration should succeed");

        diploma_service
            .register_diploma(
                university_id,
                "UNI-777".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Anna Volkova".into(),
                    student_number: "ST-4444".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0042".into(),
                    degree: "master".into(),
                    program_name: "management".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                    honors: false,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let by_code = diploma_service
            .search_hr_registry(HrRegistrySearchRequest {
                diploma_number: None,
                university_code: Some("UNI-777".into()),
            })
            .await
            .expect("hr search by code should succeed");
        assert_eq!(by_code.items.len(), 1);

        let by_number = diploma_service
            .search_hr_registry(HrRegistrySearchRequest {
                diploma_number: Some("DP-2026-0042".into()),
                university_code: None,
            })
            .await
            .expect("hr search by diploma number should succeed");
        assert_eq!(by_number.items.len(), 1);
    }

    #[tokio::test]
    async fn hr_search_with_both_filters_returns_empty_when_one_filter_does_not_match() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-777".into()),
            })
            .await
            .expect("university registration should succeed");

        diploma_service
            .register_diploma(
                university_id,
                "UNI-777".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Anna Volkova".into(),
                    student_number: "ST-4444".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0042".into(),
                    degree: "master".into(),
                    program_name: "management".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                    honors: false,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let result = diploma_service
            .search_hr_registry(HrRegistrySearchRequest {
                diploma_number: Some("DP-DOES-NOT-EXIST".into()),
                university_code: Some("UNI-777".into()),
            })
            .await
            .expect("hr search should succeed");

        assert!(result.items.is_empty());
    }

    #[tokio::test]
    async fn student_search_with_both_filters_returns_empty_when_one_filter_does_not_match() {
        let (auth_service, diploma_service, _) = build_services();
        let university_id = UniversityId::new();

        auth_service
            .register(RegisterUserRequest {
                email: "uni@example.com".into(),
                password: "superpass".into(),
                full_name: "Test University".into(),
                student_number: None,
                role: UserRole::University,
                university_id: Some(university_id),
                university_code: Some("UNI-001".into()),
            })
            .await
            .expect("university registration should succeed");

        diploma_service
            .register_diploma(
                university_id,
                "UNI-001".into(),
                RegisterDiplomaRequest {
                    student_full_name: "Ivan Petrov".into(),
                    student_number: "ST-1001".into(),
                    student_birth_date: None,
                    diploma_number: "DP-2026-0001".into(),
                    degree: "bachelor".into(),
                    program_name: "computer science".into(),
                    graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                    honors: false,
                },
            )
            .await
            .expect("diploma registration should succeed");

        let student = auth_service
            .register(RegisterUserRequest {
                email: "ivan@example.com".into(),
                password: "superpass".into(),
                full_name: "Ivan Petrov".into(),
                student_number: Some("ST-1001".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("student registration should succeed");

        let result = diploma_service
            .search_student_diplomas(
                student.user.id,
                StudentDiplomaSearchRequest {
                    diploma_number: Some("DP-DOES-NOT-EXIST".into()),
                    student_full_name: Some("Ivan Petrov".into()),
                },
            )
            .await
            .expect("student search should succeed");

        assert!(result.items.is_empty());
    }

    #[tokio::test]
    async fn login_succeeds_after_registration() {
        let (auth_service, _, _) = build_services();

        auth_service
            .register(RegisterUserRequest {
                email: "student@example.com".into(),
                password: "superpass".into(),
                full_name: "Test Student".into(),
                student_number: Some("ST-5555".into()),
                role: UserRole::Student,
                university_id: None,
                university_code: None,
            })
            .await
            .expect("registration should succeed");

        let login = auth_service
            .login(LoginRequest {
                email: "student@example.com".into(),
                password: "superpass".into(),
            })
            .await
            .expect("login should succeed");

        assert_eq!(login.user.email, "student@example.com");
        assert_eq!(login.token_type, "Bearer");
    }
}
