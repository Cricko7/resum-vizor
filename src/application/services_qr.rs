use std::{sync::Arc, time::Duration};

use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::{
    application::{
        dto::{CreateDiplomaQrRequest, DeleteDiplomaQrResponse, DiplomaQrResponse},
        ports::{AppRepository, QrGateway},
    },
    domain::{
        ids::DiplomaId,
        qr::{CreateQrJobPayload, DiplomaQrCode, QrBinaryContent, QrCodeStatus, QrImageFormat},
    },
    error::AppError,
    infrastructure::cache::ResponseCache,
};

use super::services_diploma::DiplomaService;

const DEFAULT_QR_SIZE: u32 = 512;
const MIN_QR_SIZE: u32 = 128;
const MAX_QR_SIZE: u32 = 2048;
const QR_CACHE_NAMESPACE: &str = "qr_read_model";
const DEFAULT_QR_CACHE_TTL_SECONDS: u64 = 300;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedQrContent {
    content_type: String,
    body_base64: String,
}

pub struct QrService {
    repository: Arc<dyn AppRepository>,
    diploma_service: Arc<DiplomaService>,
    qr_gateway: Arc<dyn QrGateway>,
    response_cache: Arc<dyn ResponseCache>,
    base_url: String,
    diploma_link_ttl_minutes: i64,
}

impl QrService {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        diploma_service: Arc<DiplomaService>,
        qr_gateway: Arc<dyn QrGateway>,
        response_cache: Arc<dyn ResponseCache>,
        base_url: String,
        diploma_link_ttl_minutes: i64,
    ) -> Self {
        Self {
            repository,
            diploma_service,
            qr_gateway,
            response_cache,
            base_url,
            diploma_link_ttl_minutes,
        }
    }

    pub async fn create_or_get_diploma_qr(
        &self,
        student_user_id: crate::domain::ids::UserId,
        diploma_id: DiplomaId,
        request: CreateDiplomaQrRequest,
    ) -> Result<DiplomaQrResponse, AppError> {
        let diploma = self.ensure_owned_diploma(student_user_id, diploma_id).await?;

        if let Some(existing) = self
            .repository
            .find_diploma_qr_code_by_diploma_id(diploma_id)
            .await?
        {
            let refreshed = self.refresh_record(existing).await?;
            if !request.force_regenerate
                && matches!(refreshed.status, QrCodeStatus::Pending | QrCodeStatus::Ready)
                && !refreshed.is_expired()
            {
                return Ok(DiplomaQrResponse::from_record(&self.base_url, refreshed));
            }

            if let Some(qr_id) = refreshed.external_qr_id.as_deref() {
                let _ = self.qr_gateway.delete_qr(qr_id).await;
            }
        }

        let format = request.format.unwrap_or(QrImageFormat::Png);
        let size = request.size.unwrap_or(DEFAULT_QR_SIZE);
        self.validate_size(size)?;

        let share_link = self
            .diploma_service
            .generate_diploma_share_link(
                student_user_id,
                diploma_id,
                &self.base_url,
                self.diploma_link_ttl_minutes,
            )
            .await?;
        let payload = CreateQrJobPayload {
            external_id: diploma.certificate_id.0.to_string(),
            payload_url: share_link.access_url,
            format,
            size,
            ttl_seconds: share_link.expires_in_seconds,
            idempotency_key: format!("diploma-qr:{}:{}", diploma.id.0, chrono::Utc::now().timestamp()),
        };

        let created_job = self.qr_gateway.create_job(payload.clone()).await?;
        let mut record = DiplomaQrCode::pending(
            diploma.id,
            student_user_id,
            diploma.certificate_id,
            &payload,
            created_job,
        );
        record = self.materialize_if_ready(record).await?;
        let saved = self.repository.upsert_diploma_qr_code(record).await?;
        self.invalidate_qr_cache().await;

        Ok(DiplomaQrResponse::from_record(&self.base_url, saved))
    }

    pub async fn get_diploma_qr(
        &self,
        student_user_id: crate::domain::ids::UserId,
        diploma_id: DiplomaId,
    ) -> Result<DiplomaQrResponse, AppError> {
        let _ = self.ensure_owned_diploma(student_user_id, diploma_id).await?;
        let record = self
            .repository
            .find_diploma_qr_code_by_diploma_id(diploma_id)
            .await?
            .ok_or(AppError::NotFound)?;
        let record = self.refresh_record(record).await?;
        Ok(DiplomaQrResponse::from_record(&self.base_url, record))
    }

    pub async fn get_diploma_qr_content(
        &self,
        student_user_id: crate::domain::ids::UserId,
        diploma_id: DiplomaId,
    ) -> Result<QrBinaryContent, AppError> {
        let _ = self.ensure_owned_diploma(student_user_id, diploma_id).await?;
        let record = self
            .repository
            .find_diploma_qr_code_by_diploma_id(diploma_id)
            .await?
            .ok_or(AppError::NotFound)?;
        let record = self.refresh_record(record).await?;

        if !record.is_ready() || record.is_expired() {
            return Err(AppError::Conflict(
                "qr code is not ready yet or has expired".into(),
            ));
        }

        let qr_id = record
            .external_qr_id
            .clone()
            .ok_or(AppError::NotFound)?;
        let cache_key = self
            .versioned_cache_key("qr_content", &format!("{}:{qr_id}", diploma_id.0))
            .await;

        if let Some(cached) = self.get_cached_qr_content(&cache_key).await {
            return Ok(cached);
        }

        let content = self.qr_gateway.get_qr_content(&qr_id).await?;
        let ttl = self.qr_cache_ttl(&record);
        self.set_cached_qr_content(&cache_key, &content, ttl).await;
        Ok(content)
    }

    pub async fn delete_diploma_qr(
        &self,
        student_user_id: crate::domain::ids::UserId,
        diploma_id: DiplomaId,
    ) -> Result<DeleteDiplomaQrResponse, AppError> {
        let _ = self.ensure_owned_diploma(student_user_id, diploma_id).await?;

        if let Some(record) = self
            .repository
            .find_diploma_qr_code_by_diploma_id(diploma_id)
            .await?
        {
            if let Some(qr_id) = record.external_qr_id.as_deref() {
                let _ = self.qr_gateway.delete_qr(qr_id).await;
            }
            self.repository
                .delete_diploma_qr_code_by_diploma_id(diploma_id)
                .await?;
            self.invalidate_qr_cache().await;
        }

        Ok(DeleteDiplomaQrResponse {
            diploma_id,
            deleted: true,
        })
    }

    async fn ensure_owned_diploma(
        &self,
        student_user_id: crate::domain::ids::UserId,
        diploma_id: DiplomaId,
    ) -> Result<crate::domain::diploma::Diploma, AppError> {
        let diploma = self
            .repository
            .find_by_id(diploma_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if diploma.student_account_id != Some(student_user_id) {
            return Err(AppError::Forbidden(
                "student can manage qr only for their own diploma".into(),
            ));
        }

        Ok(diploma)
    }

    async fn refresh_record(&self, mut record: DiplomaQrCode) -> Result<DiplomaQrCode, AppError> {
        if record.status == QrCodeStatus::Pending {
            if let Some(job_id) = record.external_job_id.clone() {
                let job = self.qr_gateway.get_job(&job_id).await?;
                let previous_status = record.status;
                record.mark_from_job(job);
                record = self.materialize_if_ready(record).await?;
                record = self.repository.upsert_diploma_qr_code(record).await?;
                if previous_status != record.status {
                    self.invalidate_qr_cache().await;
                }
            }
        } else if record.is_ready() {
            if let Some(qr_id) = record.external_qr_id.clone() {
                let metadata = self.qr_gateway.get_qr(&qr_id).await?;
                record.mark_ready(metadata);
                record = self.repository.upsert_diploma_qr_code(record).await?;
            }
        }

        if record.is_expired() && record.status == QrCodeStatus::Ready {
            record.status = QrCodeStatus::Expired;
            record = self.repository.upsert_diploma_qr_code(record).await?;
            self.invalidate_qr_cache().await;
        }

        Ok(record)
    }

    async fn materialize_if_ready(&self, mut record: DiplomaQrCode) -> Result<DiplomaQrCode, AppError> {
        if record.status == QrCodeStatus::Ready {
            if let Some(qr_id) = record.external_qr_id.clone() {
                let metadata = self.qr_gateway.get_qr(&qr_id).await?;
                record.mark_ready(metadata);
            }
        }

        Ok(record)
    }

    fn validate_size(&self, size: u32) -> Result<(), AppError> {
        if !(MIN_QR_SIZE..=MAX_QR_SIZE).contains(&size) {
            return Err(AppError::Validation(format!(
                "size must be between {MIN_QR_SIZE} and {MAX_QR_SIZE}"
            )));
        }

        Ok(())
    }

    async fn invalidate_qr_cache(&self) {
        self.response_cache.bump_namespace(QR_CACHE_NAMESPACE).await;
    }

    async fn versioned_cache_key(&self, scope: &str, identity: &str) -> String {
        let version = self.response_cache.namespace_version(QR_CACHE_NAMESPACE).await;
        format!("{scope}:v{version}:{identity}")
    }

    async fn get_cached_qr_content(&self, key: &str) -> Option<QrBinaryContent> {
        let cached = self.response_cache.get(key).await?;
        let cached: CachedQrContent = serde_json::from_str(&cached).ok()?;
        let bytes = STANDARD.decode(cached.body_base64).ok()?;

        Some(QrBinaryContent {
            content_type: cached.content_type,
            bytes,
        })
    }

    async fn set_cached_qr_content(&self, key: &str, content: &QrBinaryContent, ttl: Duration) {
        let payload = CachedQrContent {
            content_type: content.content_type.clone(),
            body_base64: STANDARD.encode(&content.bytes),
        };

        let Ok(serialized) = serde_json::to_string(&payload) else {
            return;
        };

        self.response_cache.set(key, &serialized, ttl).await;
    }

    fn qr_cache_ttl(&self, record: &DiplomaQrCode) -> Duration {
        let fallback = Duration::from_secs(DEFAULT_QR_CACHE_TTL_SECONDS);
        let Some(expires_at) = record.expires_at else {
            return fallback;
        };
        let now = chrono::Utc::now();
        let remaining = expires_at - now;
        if remaining <= chrono::Duration::zero() {
            return Duration::from_secs(1);
        }

        remaining.to_std().unwrap_or(fallback)
    }
}
