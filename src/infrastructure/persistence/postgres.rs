use async_trait::async_trait;
use secrecy::ExposeSecret;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

use crate::{
    application::ports::{
        AtsApiKeyRepository, DiplomaQrCodeRepository, DiplomaRepository, HealthChecker,
        UserRepository,
    },
    config::DatabaseSettings,
    domain::{
        ats::{AtsApiKey, IntegrationApiScope},
        diploma::{Diploma, DiplomaStatus},
        hashing::HashedDiplomaPayload,
        ids::{AtsApiKeyId, CertificateId, DiplomaId, StudentId, UniversityId, UserId},
        qr::{DiplomaQrCode, QrCodeStatus, QrImageFormat},
        user::{User, UserRole},
    },
    error::AppError,
};

#[derive(Clone, Debug)]
pub struct PostgresAppRepository {
    pool: PgPool,
}

impl PostgresAppRepository {
    pub async fn connect(settings: &DatabaseSettings) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(settings.url.expose_secret())
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[async_trait]
impl DiplomaRepository for PostgresAppRepository {
    async fn save(&self, diploma: Diploma) -> Result<Diploma, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO diplomas (
                id, university_id, student_id, certificate_id, student_account_id, university_code,
                student_number_last4, diploma_number_last4, record_hash, university_signature,
                signature_algorithm, status, revoked_at, university_code_hash, student_full_name_hash,
                student_number_hash, student_birth_date_hash, diploma_number_hash,
                verification_lookup_hash, degree_hash, program_hash, graduation_date_hash,
                honors_hash, canonical_document_hash, issued_at, created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17, $18,
                $19, $20, $21, $22,
                $23, $24, $25, $26
            )
            "#,
        )
        .bind(diploma.id.0)
        .bind(diploma.university_id.0)
        .bind(diploma.student_id.0)
        .bind(diploma.certificate_id.0)
        .bind(diploma.student_account_id.map(|value| value.0))
        .bind(&diploma.university_code)
        .bind(&diploma.student_number_last4)
        .bind(&diploma.diploma_number_last4)
        .bind(&diploma.record_hash)
        .bind(&diploma.university_signature)
        .bind(diploma.signature_algorithm)
        .bind(diploma_status_to_db(diploma.status))
        .bind(diploma.revoked_at)
        .bind(&diploma.hashed_payload.university_code_hash)
        .bind(&diploma.hashed_payload.student_full_name_hash)
        .bind(&diploma.hashed_payload.student_number_hash)
        .bind(&diploma.hashed_payload.student_birth_date_hash)
        .bind(&diploma.hashed_payload.diploma_number_hash)
        .bind(&diploma.hashed_payload.verification_lookup_hash)
        .bind(&diploma.hashed_payload.degree_hash)
        .bind(&diploma.hashed_payload.program_hash)
        .bind(&diploma.hashed_payload.graduation_date_hash)
        .bind(&diploma.hashed_payload.honors_hash)
        .bind(&diploma.hashed_payload.canonical_document_hash)
        .bind(diploma.issued_at)
        .bind(diploma.created_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(diploma),
            Err(error) if is_unique_violation(&error) => {
                Err(AppError::Conflict("diploma already registered".into()))
            }
            Err(_) => Err(AppError::Internal),
        }
    }

    async fn find_by_student_id(&self, student_id: UserId) -> Result<Vec<Diploma>, AppError> {
        let rows = sqlx::query("SELECT * FROM diplomas WHERE student_account_id = $1 ORDER BY created_at DESC")
            .bind(student_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        rows.into_iter().map(row_to_diploma).collect()
    }

    async fn find_by_certificate_id(
        &self,
        certificate_id: CertificateId,
    ) -> Result<Option<Diploma>, AppError> {
        let row = sqlx::query("SELECT * FROM diplomas WHERE certificate_id = $1")
            .bind(certificate_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_diploma).transpose()
    }

    async fn find_by_canonical_hash(&self, canonical_hash: &str) -> Result<Option<Diploma>, AppError> {
        let row = sqlx::query("SELECT * FROM diplomas WHERE verification_lookup_hash = $1")
            .bind(canonical_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_diploma).transpose()
    }

    async fn find_by_id(&self, diploma_id: DiplomaId) -> Result<Option<Diploma>, AppError> {
        let row = sqlx::query("SELECT * FROM diplomas WHERE id = $1")
            .bind(diploma_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_diploma).transpose()
    }

    async fn update(&self, diploma: Diploma) -> Result<Diploma, AppError> {
        sqlx::query(
            r#"
            UPDATE diplomas
            SET student_account_id = $2, status = $3, revoked_at = $4
            WHERE id = $1
            "#,
        )
        .bind(diploma.id.0)
        .bind(diploma.student_account_id.map(|value| value.0))
        .bind(diploma_status_to_db(diploma.status))
        .bind(diploma.revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|_| AppError::Internal)?;

        Ok(diploma)
    }

    async fn search_by_student_name_hash(&self, full_name_hash: &str) -> Result<Vec<Diploma>, AppError> {
        search_diplomas(&self.pool, "student_full_name_hash", full_name_hash).await
    }

    async fn search_by_diploma_number_hash(
        &self,
        diploma_number_hash: &str,
    ) -> Result<Vec<Diploma>, AppError> {
        search_diplomas(&self.pool, "diploma_number_hash", diploma_number_hash).await
    }

    async fn search_by_university_code_hash(
        &self,
        university_code_hash: &str,
    ) -> Result<Vec<Diploma>, AppError> {
        search_diplomas(&self.pool, "university_code_hash", university_code_hash).await
    }
}

#[async_trait]
impl UserRepository for PostgresAppRepository {
    async fn create_user(&self, user: User) -> Result<User, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (
                id, email, password_hash, full_name, student_number, role,
                university_id, university_code, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.student_number)
        .bind(user.role.to_string())
        .bind(user.university_id.map(|value| value.0))
        .bind(&user.university_code)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(user),
            Err(error) if is_unique_violation(&error) => {
                Err(AppError::Conflict("user with this email already exists".into()))
            }
            Err(_) => Err(AppError::Internal),
        }
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_user).transpose()
    }

    async fn find_user_by_id(&self, user_id: UserId) -> Result<Option<User>, AppError> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(user_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_user).transpose()
    }

    async fn update_user(&self, user: User) -> Result<User, AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET email = $2, password_hash = $3, full_name = $4, student_number = $5,
                role = $6, university_id = $7, university_code = $8, updated_at = $9
            WHERE id = $1
            "#,
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.student_number)
        .bind(user.role.to_string())
        .bind(user.university_id.map(|value| value.0))
        .bind(&user.university_code)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|_| AppError::Internal)?;

        Ok(user)
    }
}

#[async_trait]
impl AtsApiKeyRepository for PostgresAppRepository {
    async fn create_ats_api_key(&self, api_key: AtsApiKey) -> Result<AtsApiKey, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO ats_api_keys (
                id, hr_user_id, name, scope, key_prefix, key_hash, daily_request_limit,
                burst_request_limit, burst_window_seconds,
                created_at, updated_at, last_used_at, revoked_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(api_key.id.0)
        .bind(api_key.hr_user_id.0)
        .bind(&api_key.name)
        .bind(integration_scope_to_db(api_key.scope))
        .bind(&api_key.key_prefix)
        .bind(&api_key.key_hash)
        .bind(api_key.daily_request_limit as i64)
        .bind(api_key.burst_request_limit as i64)
        .bind(api_key.burst_window_seconds as i64)
        .bind(api_key.created_at)
        .bind(api_key.updated_at)
        .bind(api_key.last_used_at)
        .bind(api_key.revoked_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(api_key),
            Err(error) if is_unique_violation(&error) => {
                Err(AppError::Conflict("ATS API key already exists".into()))
            }
            Err(_) => Err(AppError::Internal),
        }
    }

    async fn find_ats_api_key_by_hash(&self, key_hash: &str) -> Result<Option<AtsApiKey>, AppError> {
        let row = sqlx::query("SELECT * FROM ats_api_keys WHERE key_hash = $1")
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_ats_api_key).transpose()
    }

    async fn find_ats_api_key_by_id(&self, api_key_id: AtsApiKeyId) -> Result<Option<AtsApiKey>, AppError> {
        let row = sqlx::query("SELECT * FROM ats_api_keys WHERE id = $1")
            .bind(api_key_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_ats_api_key).transpose()
    }

    async fn list_ats_api_keys_by_hr_user(&self, hr_user_id: UserId) -> Result<Vec<AtsApiKey>, AppError> {
        let rows = sqlx::query("SELECT * FROM ats_api_keys WHERE hr_user_id = $1 ORDER BY created_at DESC")
            .bind(hr_user_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        rows.into_iter().map(row_to_ats_api_key).collect()
    }

    async fn update_ats_api_key(&self, api_key: AtsApiKey) -> Result<AtsApiKey, AppError> {
        sqlx::query(
            r#"
            UPDATE ats_api_keys
            SET name = $2, scope = $3, key_prefix = $4, key_hash = $5,
                daily_request_limit = $6, burst_request_limit = $7, burst_window_seconds = $8,
                updated_at = $9, last_used_at = $10, revoked_at = $11
            WHERE id = $1
            "#,
        )
        .bind(api_key.id.0)
        .bind(&api_key.name)
        .bind(integration_scope_to_db(api_key.scope))
        .bind(&api_key.key_prefix)
        .bind(&api_key.key_hash)
        .bind(api_key.daily_request_limit as i64)
        .bind(api_key.burst_request_limit as i64)
        .bind(api_key.burst_window_seconds as i64)
        .bind(api_key.updated_at)
        .bind(api_key.last_used_at)
        .bind(api_key.revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|_| AppError::Internal)?;

        Ok(api_key)
    }
}

#[async_trait]
impl DiplomaQrCodeRepository for PostgresAppRepository {
    async fn upsert_diploma_qr_code(&self, qr_code: DiplomaQrCode) -> Result<DiplomaQrCode, AppError> {
        sqlx::query(
            r#"
            INSERT INTO diploma_qr_codes (
                diploma_id, student_user_id, certificate_id, external_id, payload_url, format,
                size, ttl_seconds, status, external_job_id, external_qr_id, external_download_url,
                expires_at, error_message, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16
            )
            ON CONFLICT (diploma_id)
            DO UPDATE SET
                student_user_id = EXCLUDED.student_user_id,
                certificate_id = EXCLUDED.certificate_id,
                external_id = EXCLUDED.external_id,
                payload_url = EXCLUDED.payload_url,
                format = EXCLUDED.format,
                size = EXCLUDED.size,
                ttl_seconds = EXCLUDED.ttl_seconds,
                status = EXCLUDED.status,
                external_job_id = EXCLUDED.external_job_id,
                external_qr_id = EXCLUDED.external_qr_id,
                external_download_url = EXCLUDED.external_download_url,
                expires_at = EXCLUDED.expires_at,
                error_message = EXCLUDED.error_message,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(qr_code.diploma_id.0)
        .bind(qr_code.student_user_id.0)
        .bind(qr_code.certificate_id.0)
        .bind(&qr_code.external_id)
        .bind(&qr_code.payload_url)
        .bind(qr_image_format_to_db(qr_code.format))
        .bind(qr_code.size as i32)
        .bind(qr_code.ttl_seconds)
        .bind(qr_status_to_db(qr_code.status))
        .bind(&qr_code.external_job_id)
        .bind(&qr_code.external_qr_id)
        .bind(&qr_code.external_download_url)
        .bind(qr_code.expires_at)
        .bind(&qr_code.error_message)
        .bind(qr_code.created_at)
        .bind(qr_code.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|_| AppError::Internal)?;

        Ok(qr_code)
    }

    async fn find_diploma_qr_code_by_diploma_id(
        &self,
        diploma_id: DiplomaId,
    ) -> Result<Option<DiplomaQrCode>, AppError> {
        let row = sqlx::query("SELECT * FROM diploma_qr_codes WHERE diploma_id = $1")
            .bind(diploma_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;

        row.map(row_to_diploma_qr_code).transpose()
    }

    async fn delete_diploma_qr_code_by_diploma_id(&self, diploma_id: DiplomaId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM diploma_qr_codes WHERE diploma_id = $1")
            .bind(diploma_id.0)
            .execute(&self.pool)
            .await
            .map_err(|_| AppError::Internal)?;
        Ok(())
    }
}

#[async_trait]
impl HealthChecker for PostgresAppRepository {
    async fn is_ready(&self) -> bool {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }
}

async fn search_diplomas(pool: &PgPool, column: &str, value: &str) -> Result<Vec<Diploma>, AppError> {
    let query = match column {
        "student_full_name_hash" => {
            sqlx::query("SELECT * FROM diplomas WHERE student_full_name_hash = $1 ORDER BY created_at DESC")
        }
        "diploma_number_hash" => {
            sqlx::query("SELECT * FROM diplomas WHERE diploma_number_hash = $1 ORDER BY created_at DESC")
        }
        "university_code_hash" => {
            sqlx::query("SELECT * FROM diplomas WHERE university_code_hash = $1 ORDER BY created_at DESC")
        }
        _ => return Err(AppError::Internal),
    };

    let rows = query.bind(value).fetch_all(pool).await.map_err(|_| AppError::Internal)?;

    rows.into_iter().map(row_to_diploma).collect()
}

fn row_to_user(row: sqlx::postgres::PgRow) -> Result<User, AppError> {
    let role = row.get::<String, _>("role");
    Ok(User {
        id: UserId(row.get("id")),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        full_name: row.get("full_name"),
        student_number: row.get("student_number"),
        role: parse_user_role(&role)?,
        university_id: row.get::<Option<uuid::Uuid>, _>("university_id").map(UniversityId),
        university_code: row.get("university_code"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn row_to_diploma(row: sqlx::postgres::PgRow) -> Result<Diploma, AppError> {
    let status = row.get::<String, _>("status");
    let signature_algorithm: String = row.get("signature_algorithm");
    let signature_algorithm = match signature_algorithm.as_str() {
        "ed25519" => "ed25519",
        _ => return Err(AppError::Internal),
    };

    Ok(Diploma {
        id: DiplomaId(row.get("id")),
        university_id: UniversityId(row.get("university_id")),
        student_id: StudentId(row.get("student_id")),
        certificate_id: CertificateId(row.get("certificate_id")),
        student_account_id: row
            .get::<Option<uuid::Uuid>, _>("student_account_id")
            .map(UserId),
        university_code: row.get("university_code"),
        student_number_last4: row.get("student_number_last4"),
        diploma_number_last4: row.get("diploma_number_last4"),
        record_hash: row.get("record_hash"),
        university_signature: row.get("university_signature"),
        signature_algorithm,
        status: parse_diploma_status(&status)?,
        revoked_at: row.get("revoked_at"),
        hashed_payload: HashedDiplomaPayload {
            university_code_hash: row.get("university_code_hash"),
            student_full_name_hash: row.get("student_full_name_hash"),
            student_number_hash: row.get("student_number_hash"),
            student_birth_date_hash: row.get("student_birth_date_hash"),
            diploma_number_hash: row.get("diploma_number_hash"),
            verification_lookup_hash: row.get("verification_lookup_hash"),
            degree_hash: row.get("degree_hash"),
            program_hash: row.get("program_hash"),
            graduation_date_hash: row.get("graduation_date_hash"),
            honors_hash: row.get("honors_hash"),
            canonical_document_hash: row.get("canonical_document_hash"),
        },
        issued_at: row.get("issued_at"),
        created_at: row.get("created_at"),
    })
}

fn row_to_ats_api_key(row: sqlx::postgres::PgRow) -> Result<AtsApiKey, AppError> {
    Ok(AtsApiKey {
        id: AtsApiKeyId(row.get("id")),
        hr_user_id: UserId(row.get("hr_user_id")),
        name: row.get("name"),
        scope: parse_integration_scope(&row.get::<String, _>("scope"))?,
        key_prefix: row.get("key_prefix"),
        key_hash: row.get("key_hash"),
        daily_request_limit: row.get::<i64, _>("daily_request_limit") as usize,
        burst_request_limit: row.get::<i64, _>("burst_request_limit") as usize,
        burst_window_seconds: row.get::<i64, _>("burst_window_seconds") as u64,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        last_used_at: row.get("last_used_at"),
        revoked_at: row.get("revoked_at"),
    })
}

fn row_to_diploma_qr_code(row: sqlx::postgres::PgRow) -> Result<DiplomaQrCode, AppError> {
    Ok(DiplomaQrCode {
        diploma_id: DiplomaId(row.get("diploma_id")),
        student_user_id: UserId(row.get("student_user_id")),
        certificate_id: CertificateId(row.get("certificate_id")),
        external_id: row.get("external_id"),
        payload_url: row.get("payload_url"),
        format: parse_qr_image_format(&row.get::<String, _>("format"))?,
        size: row.get::<i32, _>("size") as u32,
        ttl_seconds: row.get("ttl_seconds"),
        status: parse_qr_status(&row.get::<String, _>("status"))?,
        external_job_id: row.get("external_job_id"),
        external_qr_id: row.get("external_qr_id"),
        external_download_url: row.get("external_download_url"),
        expires_at: row.get("expires_at"),
        error_message: row.get("error_message"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_user_role(value: &str) -> Result<UserRole, AppError> {
    match value {
        "university" => Ok(UserRole::University),
        "student" => Ok(UserRole::Student),
        "hr" => Ok(UserRole::Hr),
        _ => Err(AppError::Internal),
    }
}

fn parse_diploma_status(value: &str) -> Result<DiplomaStatus, AppError> {
    match value {
        "active" => Ok(DiplomaStatus::Active),
        "revoked" => Ok(DiplomaStatus::Revoked),
        _ => Err(AppError::Internal),
    }
}

fn parse_integration_scope(value: &str) -> Result<IntegrationApiScope, AppError> {
    match value {
        "ats_only" => Ok(IntegrationApiScope::AtsOnly),
        "hr_automation_only" => Ok(IntegrationApiScope::HrAutomationOnly),
        "combined" => Ok(IntegrationApiScope::Combined),
        _ => Err(AppError::Internal),
    }
}

fn parse_qr_status(value: &str) -> Result<QrCodeStatus, AppError> {
    match value {
        "pending" => Ok(QrCodeStatus::Pending),
        "ready" => Ok(QrCodeStatus::Ready),
        "failed" => Ok(QrCodeStatus::Failed),
        "expired" => Ok(QrCodeStatus::Expired),
        _ => Err(AppError::Internal),
    }
}

fn parse_qr_image_format(value: &str) -> Result<QrImageFormat, AppError> {
    match value {
        "png" => Ok(QrImageFormat::Png),
        "svg" => Ok(QrImageFormat::Svg),
        _ => Err(AppError::Internal),
    }
}

fn diploma_status_to_db(status: DiplomaStatus) -> &'static str {
    match status {
        DiplomaStatus::Active => "active",
        DiplomaStatus::Revoked => "revoked",
    }
}

fn integration_scope_to_db(scope: IntegrationApiScope) -> &'static str {
    match scope {
        IntegrationApiScope::AtsOnly => "ats_only",
        IntegrationApiScope::HrAutomationOnly => "hr_automation_only",
        IntegrationApiScope::Combined => "combined",
    }
}

fn qr_status_to_db(status: QrCodeStatus) -> &'static str {
    match status {
        QrCodeStatus::Pending => "pending",
        QrCodeStatus::Ready => "ready",
        QrCodeStatus::Failed => "failed",
        QrCodeStatus::Expired => "expired",
    }
}

fn qr_image_format_to_db(format: QrImageFormat) -> &'static str {
    match format {
        QrImageFormat::Png => "png",
        QrImageFormat::Svg => "svg",
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|db_error| db_error.code())
        .is_some_and(|code| code == "23505")
}
