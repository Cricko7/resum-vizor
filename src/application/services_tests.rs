use std::sync::Arc;

use secrecy::SecretString;

use crate::{
    application::{
        dto::{
            AtsVerifyRequest, CreateAtsApiKeyRequest, HrRegistrySearchRequest, LoginRequest,
            RegisterDiplomaRequest, RegisterUserRequest, StudentDiplomaSearchRequest,
        },
        ports::AppRepository,
        services::{AtsService, AuthService, DiplomaService},
    },
    domain::{
        ats::IntegrationApiScope,
        ids::UniversityId,
        user::UserRole,
    },
    error::AppError,
    infrastructure::{
        api_keys::Blake3AtsKeyManager,
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

fn build_services_with_ats() -> (AuthService, DiplomaService, AtsService, Arc<dyn AppRepository>) {
    let (auth_service, diploma_service, repository) = build_services();
    let ats_service = AtsService::new(
        repository.clone(),
        Arc::new(Blake3AtsKeyManager::new(&secret("ats-secret"))),
        10,
        30,
        20,
        40,
    );

    (auth_service, diploma_service, ats_service, repository)
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
        .generate_diploma_share_link(intruder.user.id, diploma.id, "http://localhost:8080", 15)
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
async fn ats_api_key_can_verify_registry_via_machine_to_machine_flow() {
    let (auth_service, diploma_service, ats_service, _) = build_services_with_ats();
    let university_id = UniversityId::new();

    auth_service
        .register(RegisterUserRequest {
            email: "uni@example.com".into(),
            password: "superpass".into(),
            full_name: "Test University".into(),
            student_number: None,
            role: UserRole::University,
            university_id: Some(university_id),
            university_code: Some("UNI-900".into()),
        })
        .await
        .expect("university registration should succeed");

    diploma_service
        .register_diploma(
            university_id,
            "UNI-900".into(),
            RegisterDiplomaRequest {
                student_full_name: "Alice Hr".into(),
                student_number: "ST-7777".into(),
                student_birth_date: None,
                diploma_number: "DP-ATS-0001".into(),
                degree: "master".into(),
                program_name: "analytics".into(),
                graduation_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                honors: false,
            },
        )
        .await
        .expect("diploma registration should succeed");

    let hr = auth_service
        .register(RegisterUserRequest {
            email: "hr@example.com".into(),
            password: "superpass".into(),
            full_name: "Recruiter".into(),
            student_number: None,
            role: UserRole::Hr,
            university_id: None,
            university_code: None,
        })
        .await
        .expect("hr registration should succeed");

    let created_key = ats_service
        .create_api_key(
            hr.user.id,
            CreateAtsApiKeyRequest {
                name: "Greenhouse prod".into(),
                scope: IntegrationApiScope::Combined,
            },
        )
        .await
        .expect("api key should be created");

    let client = ats_service
        .authorize_api_key_for_ats(&created_key.api_key)
        .await
        .expect("api key should authenticate");

    let response = ats_service
        .verify_for_ats(
            &client.name,
            AtsVerifyRequest {
                diploma_number: Some("DP-ATS-0001".into()),
                university_code: Some("UNI-900".into()),
                candidate_reference: Some("candidate-42".into()),
                resume_reference: Some("resume-42".into()),
            },
            &diploma_service,
        )
        .await
        .expect("ats verify should succeed");

    assert!(response.verified);
    assert_eq!(response.match_count, 1);
    assert_eq!(response.integration_name, "Greenhouse prod");
    assert!(response.risk_flags.is_empty());
    assert_eq!(created_key.daily_request_limit, 1_000);
    assert_eq!(created_key.burst_request_limit, 40);
    assert_eq!(created_key.burst_window_seconds, 10);
}

#[tokio::test]
async fn revoked_ats_api_key_is_rejected() {
    let (auth_service, _, ats_service, _) = build_services_with_ats();

    let hr = auth_service
        .register(RegisterUserRequest {
            email: "hr@example.com".into(),
            password: "superpass".into(),
            full_name: "Recruiter".into(),
            student_number: None,
            role: UserRole::Hr,
            university_id: None,
            university_code: None,
        })
        .await
        .expect("hr registration should succeed");

    let created_key = ats_service
        .create_api_key(
            hr.user.id,
            CreateAtsApiKeyRequest {
                name: "Huntflow".into(),
                scope: IntegrationApiScope::AtsOnly,
            },
        )
        .await
        .expect("api key should be created");

    ats_service
        .revoke_api_key(hr.user.id, created_key.api_key_id)
        .await
        .expect("api key should be revoked");

    let result = ats_service.authorize_api_key_for_ats(&created_key.api_key).await;
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn automation_only_key_is_rejected_by_ats_scope_check() {
    let (auth_service, _, ats_service, _) = build_services_with_ats();

    let hr = auth_service
        .register(RegisterUserRequest {
            email: "hr@example.com".into(),
            password: "superpass".into(),
            full_name: "Recruiter".into(),
            student_number: None,
            role: UserRole::Hr,
            university_id: None,
            university_code: None,
        })
        .await
        .expect("hr registration should succeed");

    let created_key = ats_service
        .create_api_key(
            hr.user.id,
            CreateAtsApiKeyRequest {
                name: "Automation only".into(),
                scope: IntegrationApiScope::HrAutomationOnly,
            },
        )
        .await
        .expect("api key should be created");

    let result = ats_service.authorize_api_key_for_ats(&created_key.api_key).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));

    let automation_result = ats_service
        .authorize_api_key_for_hr_automation(&created_key.api_key)
        .await
        .expect("automation scope should be allowed");
    assert_eq!(automation_result.daily_request_limit, 1_000);
    assert_eq!(automation_result.burst_request_limit, 20);
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
