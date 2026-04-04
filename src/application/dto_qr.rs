use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{
    ids::DiplomaId,
    qr::{DiplomaQrCode, QrCodeStatus, QrImageFormat},
};

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateDiplomaQrRequest {
    #[serde(default)]
    pub format: Option<QrImageFormat>,
    #[serde(default)]
    pub size: Option<u32>,
    #[serde(default)]
    pub force_regenerate: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiplomaQrResponse {
    pub diploma_id: DiplomaId,
    pub status: QrCodeStatus,
    pub job_id: Option<String>,
    pub qr_id: Option<String>,
    pub format: QrImageFormat,
    pub size: u32,
    pub expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub status_url: String,
    pub content_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DiplomaQrResponse {
    pub fn from_record(base_url: &str, value: DiplomaQrCode) -> Self {
        let base_url = base_url.trim_end_matches('/');
        let status_url = format!("{base_url}/api/v1/student/diplomas/{}/qr", value.diploma_id.0);
        let content_url = value
            .is_ready()
            .then(|| format!("{base_url}/api/v1/student/diplomas/{}/qr/content", value.diploma_id.0));

        Self {
            diploma_id: value.diploma_id,
            status: value.status,
            job_id: value.external_job_id,
            qr_id: value.external_qr_id,
            format: value.format,
            size: value.size,
            expires_at: value.expires_at,
            error_message: value.error_message,
            status_url,
            content_url,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteDiplomaQrResponse {
    pub diploma_id: DiplomaId,
    pub deleted: bool,
}
