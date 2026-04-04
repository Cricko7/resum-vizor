use anyhow::{Context, anyhow};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub security: SecuritySettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub port: u16,
    pub log_level: String,
    pub base_url: String,
    pub hr_api_rate_limit_requests: usize,
    pub hr_api_rate_limit_window_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettings {
    pub diploma_hash_key: SecretString,
    pub jwt_secret: SecretString,
    pub jwt_ttl_minutes: i64,
    pub university_signing_key: SecretString,
    pub diploma_link_ttl_minutes: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: SecretString,
    pub max_connections: u32,
}

impl Settings {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let port = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("APP_PORT must be a valid u16")?;

        let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".to_string());
        let base_url =
            std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let hr_api_rate_limit_requests = std::env::var("HR_API_RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<usize>()
            .context("HR_API_RATE_LIMIT_REQUESTS must be a valid usize")?;
        let hr_api_rate_limit_window_seconds = std::env::var("HR_API_RATE_LIMIT_WINDOW_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .context("HR_API_RATE_LIMIT_WINDOW_SECONDS must be a valid u64")?;
        let diploma_hash_key = std::env::var("DIPLOMA_HASH_KEY")
            .map(|value| SecretString::new(value.into_boxed_str()))
            .map_err(|_| anyhow!("DIPLOMA_HASH_KEY is required"))?;
        let database_url = std::env::var("DATABASE_URL")
            .map(|value| SecretString::new(value.into_boxed_str()))
            .map_err(|_| anyhow!("DATABASE_URL is required"))?;
        let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a valid u32")?;
        let jwt_secret = std::env::var("JWT_SECRET")
            .map(|value| SecretString::new(value.into_boxed_str()))
            .map_err(|_| anyhow!("JWT_SECRET is required"))?;
        let jwt_ttl_minutes = std::env::var("JWT_TTL_MINUTES")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<i64>()
            .context("JWT_TTL_MINUTES must be a valid i64")?;
        let university_signing_key = std::env::var("UNIVERSITY_SIGNING_KEY")
            .map(|value| SecretString::new(value.into_boxed_str()))
            .map_err(|_| anyhow!("UNIVERSITY_SIGNING_KEY is required"))?;
        let diploma_link_ttl_minutes = std::env::var("DIPLOMA_LINK_TTL_MINUTES")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<i64>()
            .context("DIPLOMA_LINK_TTL_MINUTES must be a valid i64")?;

        if base_url.trim().is_empty() {
            return Err(anyhow!("APP_BASE_URL must not be empty"));
        }

        if hr_api_rate_limit_requests == 0 {
            return Err(anyhow!("HR_API_RATE_LIMIT_REQUESTS must be greater than 0"));
        }

        if hr_api_rate_limit_window_seconds == 0 {
            return Err(anyhow!(
                "HR_API_RATE_LIMIT_WINDOW_SECONDS must be greater than 0"
            ));
        }

        if database_max_connections == 0 {
            return Err(anyhow!("DATABASE_MAX_CONNECTIONS must be greater than 0"));
        }

        if jwt_ttl_minutes <= 0 {
            return Err(anyhow!("JWT_TTL_MINUTES must be greater than 0"));
        }

        if diploma_link_ttl_minutes <= 0 {
            return Err(anyhow!("DIPLOMA_LINK_TTL_MINUTES must be greater than 0"));
        }

        Ok(Self {
            server: ServerSettings {
                port,
                log_level,
                base_url,
                hr_api_rate_limit_requests,
                hr_api_rate_limit_window_seconds,
            },
            database: DatabaseSettings {
                url: database_url,
                max_connections: database_max_connections,
            },
            security: SecuritySettings {
                diploma_hash_key,
                jwt_secret,
                jwt_ttl_minutes,
                university_signing_key,
                diploma_link_ttl_minutes,
            },
        })
    }
}
