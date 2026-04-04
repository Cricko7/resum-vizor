use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityScheme},
};

use crate::{
    application::dto::{
        AtsApiKeyListResponse, AtsApiKeySummary, AtsVerificationDecision, AtsVerifyRequest,
        AtsVerifyResponse, AuthResponse, ChangePasswordRequest, CreateAtsApiKeyRequest,
        CreateAtsApiKeyResponse, CreateDiplomaQrRequest, DeleteDiplomaQrResponse,
        DiplomaImportError, DiplomaImportResponse, DiplomaImportRowResult, DiplomaQrResponse,
        DiplomaShareLinkResponse, DiplomaStatusResponse, HrRegistrySearchRequest,
        HrRegistrySearchResponse, LoginRequest, PublicDiplomaView, RegisterDiplomaRequest,
        RegisterDiplomaResponse, RegisterUserRequest, StudentDiplomaCard,
        StudentDiplomaSearchRequest, StudentDiplomaSearchResponse, UserResponse,
        VerifyDiplomaRequest, VerifyDiplomaResponse,
    },
    domain::{
        ats::IntegrationApiScope,
        diploma::DiplomaStatus,
        ids::{AtsApiKeyId, CertificateId, DiplomaId, UniversityId, UserId},
        qr::{QrCodeStatus, QrImageFormat},
        user::UserRole,
    },
    error::ErrorBody,
    http::{auth, common, hr, student, university},
};

#[derive(Debug, ToSchema)]
pub struct ImportDiplomasMultipartRequest {
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register,
        auth::login,
        auth::change_password,
        auth::me,
        university::register_diploma,
        university::import_diplomas,
        university::revoke_diploma,
        university::restore_diploma,
        student::profile,
        student::search_my_diplomas,
        student::generate_share_link,
        student::create_or_get_qr,
        student::get_qr,
        student::delete_qr,
        student::get_qr_content,
        hr::verify_diploma,
        hr::search_registry,
        hr::automation_verify,
        hr::create_ats_api_key,
        hr::list_ats_api_keys,
        hr::revoke_ats_api_key,
        hr::ats_verify,
        common::health_check,
        common::liveness_check,
        common::readiness_check,
        common::public_diploma_access
    ),
    components(
        schemas(
            RegisterUserRequest,
            LoginRequest,
            ChangePasswordRequest,
            UserResponse,
            AuthResponse,
            RegisterDiplomaRequest,
            RegisterDiplomaResponse,
            DiplomaImportRowResult,
            DiplomaImportError,
            DiplomaImportResponse,
            VerifyDiplomaRequest,
            VerifyDiplomaResponse,
            DiplomaStatusResponse,
            PublicDiplomaView,
            StudentDiplomaSearchRequest,
            StudentDiplomaCard,
            StudentDiplomaSearchResponse,
            DiplomaShareLinkResponse,
            CreateDiplomaQrRequest,
            DiplomaQrResponse,
            DeleteDiplomaQrResponse,
            HrRegistrySearchRequest,
            HrRegistrySearchResponse,
            CreateAtsApiKeyRequest,
            AtsApiKeySummary,
            CreateAtsApiKeyResponse,
            AtsApiKeyListResponse,
            AtsVerifyRequest,
            AtsVerifyResponse,
            AtsVerificationDecision,
            ImportDiplomasMultipartRequest,
            common::HealthResponse,
            common::ReadinessResponse,
            common::ReadinessChecks,
            ErrorBody,
            UserRole,
            DiplomaStatus,
            QrCodeStatus,
            QrImageFormat,
            IntegrationApiScope,
            UniversityId,
            UserId,
            DiplomaId,
            CertificateId,
            AtsApiKeyId
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Registration and JWT authentication"),
        (name = "University", description = "University diploma management"),
        (name = "Student", description = "Student diploma access"),
        (name = "HR", description = "HR verification and integration key management"),
        (name = "Integrations", description = "Machine-to-machine verification endpoints"),
        (name = "Public", description = "Public temporary diploma access"),
        (name = "Operations", description = "Health and operational endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "api_key_auth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
        );
    }
}
