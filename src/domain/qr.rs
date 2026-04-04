use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::ids::{CertificateId, DiplomaId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QrImageFormat {
    Png,
    Svg,
}

impl QrImageFormat {
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Svg => "image/svg+xml",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QrCodeStatus {
    Pending,
    Ready,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQrJobPayload {
    pub external_id: String,
    pub payload_url: String,
    pub format: QrImageFormat,
    pub size: u32,
    pub ttl_seconds: i64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalQrJob {
    pub job_id: String,
    pub status: QrCodeStatus,
    pub qr_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalQrMetadata {
    pub qr_id: String,
    pub external_id: String,
    pub status: QrCodeStatus,
    pub format: QrImageFormat,
    pub size: u32,
    pub expires_at: Option<DateTime<Utc>>,
    pub download_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct QrBinaryContent {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaQrCode {
    pub diploma_id: DiplomaId,
    pub student_user_id: UserId,
    pub certificate_id: CertificateId,
    pub external_id: String,
    pub payload_url: String,
    pub format: QrImageFormat,
    pub size: u32,
    pub ttl_seconds: i64,
    pub status: QrCodeStatus,
    pub external_job_id: Option<String>,
    pub external_qr_id: Option<String>,
    pub external_download_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DiplomaQrCode {
    pub fn pending(
        diploma_id: DiplomaId,
        student_user_id: UserId,
        certificate_id: CertificateId,
        payload: &CreateQrJobPayload,
        job: ExternalQrJob,
    ) -> Self {
        let ExternalQrJob {
            job_id,
            status,
            qr_id,
            error,
        } = job;
        let now = Utc::now();
        let mut value = Self {
            diploma_id,
            student_user_id,
            certificate_id,
            external_id: payload.external_id.clone(),
            payload_url: payload.payload_url.clone(),
            format: payload.format,
            size: payload.size,
            ttl_seconds: payload.ttl_seconds,
            status: QrCodeStatus::Pending,
            external_job_id: Some(job_id),
            external_qr_id: qr_id,
            external_download_url: None,
            expires_at: Some(now + chrono::Duration::seconds(payload.ttl_seconds)),
            error_message: error,
            created_at: now,
            updated_at: now,
        };

        value.apply_job_status(status);
        value
    }

    pub fn mark_from_job(&mut self, job: ExternalQrJob) {
        let ExternalQrJob {
            job_id,
            status,
            qr_id,
            error,
        } = job;
        self.external_job_id = Some(job_id);
        self.external_qr_id = qr_id;
        self.error_message = error;
        self.apply_job_status(status);
        self.updated_at = Utc::now();
    }

    pub fn mark_ready(&mut self, metadata: ExternalQrMetadata) {
        self.status = metadata.status;
        self.external_qr_id = Some(metadata.qr_id);
        self.external_id = metadata.external_id;
        self.format = metadata.format;
        self.size = metadata.size;
        self.expires_at = metadata.expires_at;
        self.external_download_url = metadata.download_url;
        self.error_message = None;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, message: String) {
        self.status = QrCodeStatus::Failed;
        self.error_message = Some(message);
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|value| value <= Utc::now())
            || self.status == QrCodeStatus::Expired
    }

    pub fn is_ready(&self) -> bool {
        self.status == QrCodeStatus::Ready && self.external_qr_id.is_some()
    }

    fn apply_job_status(&mut self, status: QrCodeStatus) {
        self.status = status;
        if matches!(status, QrCodeStatus::Failed) && self.error_message.is_none() {
            self.error_message = Some("qr generation failed".to_string());
        }
    }
}
