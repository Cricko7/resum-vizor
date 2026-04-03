use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as PasswordHasherTrait, PasswordVerifier, SaltString,
    },
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::{
    application::ports::{JwtProvider, PasswordHasher},
    domain::{ids::DiplomaId, user::User},
    error::AppError,
};

pub struct ArgonPasswordHasher;

impl PasswordHasher for ArgonPasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AppError::Internal)
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), AppError> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(|_| AppError::Unauthorized)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub role: crate::domain::user::UserRole,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaAccessClaims {
    pub sub: String,
    pub kind: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    ttl_minutes: i64,
}

impl JwtService {
    pub fn new(secret: &SecretString, ttl_minutes: i64) -> Self {
        let bytes = secret.expose_secret().as_bytes();
        Self {
            encoding_key: EncodingKey::from_secret(bytes),
            decoding_key: DecodingKey::from_secret(bytes),
            ttl_minutes,
        }
    }
}

impl JwtProvider for JwtService {
    fn issue_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = JwtClaims {
            sub: user.id.0.to_string(),
            email: user.email.clone(),
            role: user.role,
            iat: now.timestamp(),
            exp: (now + Duration::minutes(self.ttl_minutes)).timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|_| AppError::Internal)
    }

    fn decode_token(&self, token: &str) -> Result<JwtClaims, AppError> {
        decode::<JwtClaims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized)
    }

    fn issue_diploma_access_token(
        &self,
        diploma_id: DiplomaId,
        ttl_minutes: i64,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = DiplomaAccessClaims {
            sub: diploma_id.0.to_string(),
            kind: "diploma_access".to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(ttl_minutes)).timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|_| AppError::Internal)
    }

    fn decode_diploma_access_token(
        &self,
        token: &str,
    ) -> Result<DiplomaAccessClaims, AppError> {
        let claims = decode::<DiplomaAccessClaims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized)?;

        if claims.kind != "diploma_access" {
            return Err(AppError::Unauthorized);
        }

        Ok(claims)
    }
}
