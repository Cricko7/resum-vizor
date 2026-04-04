use std::time::Duration;

use async_trait::async_trait;
use reqwest::StatusCode;
use secrecy::ExposeSecret;

use crate::{
    application::ports::QrGateway,
    config::QrServiceSettings,
    domain::qr::{CreateQrJobPayload, ExternalQrJob, ExternalQrMetadata, QrBinaryContent},
    error::AppError,
};

#[derive(Clone)]
pub struct HttpQrGateway {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl HttpQrGateway {
    pub fn new(settings: &QrServiceSettings) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(settings.connect_timeout_seconds))
            .timeout(Duration::from_secs(settings.request_timeout_seconds))
            .build()
            .map_err(|_| AppError::Internal)?;

        Ok(Self {
            client,
            base_url: settings.base_url.trim_end_matches('/').to_string(),
            api_key: settings.api_key.expose_secret().to_string(),
        })
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{}{}", self.base_url, path))
            .header("x-api-key", &self.api_key)
    }
}

pub struct DisabledQrGateway;

#[async_trait]
impl QrGateway for DisabledQrGateway {
    async fn create_job(&self, _payload: CreateQrJobPayload) -> Result<ExternalQrJob, AppError> {
        Err(AppError::ServiceUnavailable(
            "qr integration is not configured".into(),
        ))
    }

    async fn get_job(&self, _job_id: &str) -> Result<ExternalQrJob, AppError> {
        Err(AppError::ServiceUnavailable(
            "qr integration is not configured".into(),
        ))
    }

    async fn get_qr(&self, _qr_id: &str) -> Result<ExternalQrMetadata, AppError> {
        Err(AppError::ServiceUnavailable(
            "qr integration is not configured".into(),
        ))
    }

    async fn get_qr_content(&self, _qr_id: &str) -> Result<QrBinaryContent, AppError> {
        Err(AppError::ServiceUnavailable(
            "qr integration is not configured".into(),
        ))
    }

    async fn delete_qr(&self, _qr_id: &str) -> Result<(), AppError> {
        Err(AppError::ServiceUnavailable(
            "qr integration is not configured".into(),
        ))
    }
}

#[async_trait]
impl QrGateway for HttpQrGateway {
    async fn create_job(&self, payload: CreateQrJobPayload) -> Result<ExternalQrJob, AppError> {
        let response = self
            .request(reqwest::Method::POST, "/api/v1/qr/jobs")
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        parse_json_response(response).await
    }

    async fn get_job(&self, job_id: &str) -> Result<ExternalQrJob, AppError> {
        let response = self
            .request(reqwest::Method::GET, &format!("/api/v1/qr/jobs/{job_id}"))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        parse_json_response(response).await
    }

    async fn get_qr(&self, qr_id: &str) -> Result<ExternalQrMetadata, AppError> {
        let response = self
            .request(reqwest::Method::GET, &format!("/api/v1/qr/{qr_id}"))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        parse_json_response(response).await
    }

    async fn get_qr_content(&self, qr_id: &str) -> Result<QrBinaryContent, AppError> {
        let response = self
            .request(reqwest::Method::GET, &format!("/api/v1/qr/{qr_id}/content"))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = response.status();
        if !status.is_success() {
            return Err(map_upstream_status(status));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(map_reqwest_error)?
            .to_vec();

        Ok(QrBinaryContent { content_type, bytes })
    }

    async fn delete_qr(&self, qr_id: &str) -> Result<(), AppError> {
        let response = self
            .request(reqwest::Method::DELETE, &format!("/api/v1/qr/{qr_id}"))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if response.status() == StatusCode::NOT_FOUND || response.status().is_success() {
            return Ok(());
        }

        Err(map_upstream_status(response.status()))
    }
}

async fn parse_json_response<T>(response: reqwest::Response) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        return Err(map_upstream_status(status));
    }

    response.json::<T>().await.map_err(|error| {
        if error.is_timeout() {
            AppError::ServiceUnavailable("qr service request timed out".into())
        } else {
            AppError::Upstream("qr service returned invalid payload".into())
        }
    })
}

fn map_reqwest_error(error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::ServiceUnavailable("qr service request timed out".into())
    } else if error.is_connect() {
        AppError::ServiceUnavailable("qr service is unavailable".into())
    } else {
        AppError::Upstream("qr service request failed".into())
    }
}

fn map_upstream_status(status: StatusCode) -> AppError {
    if status.is_server_error() {
        AppError::ServiceUnavailable(format!(
            "qr service is temporarily unavailable ({status})"
        ))
    } else {
        AppError::Upstream(format!("qr service returned status {status}"))
    }
}
