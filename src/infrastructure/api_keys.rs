use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};
use secrecy::ExposeSecret;

use crate::{
    application::ports::AtsKeyManager,
    error::AppError,
};

pub struct Blake3AtsKeyManager {
    hash_key: [u8; 32],
}

impl Blake3AtsKeyManager {
    pub fn new(secret: &secrecy::SecretString) -> Self {
        let seed = blake3::hash(secret.expose_secret().as_bytes());
        Self {
            hash_key: *seed.as_bytes(),
        }
    }
}

impl AtsKeyManager for Blake3AtsKeyManager {
    fn generate_api_key(&self) -> Result<String, AppError> {
        let mut random = [0u8; 24];
        OsRng.fill_bytes(&mut random);
        Ok(format!("rv_ats_{}", URL_SAFE_NO_PAD.encode(random)))
    }

    fn hash_api_key(&self, api_key: &str) -> Result<String, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::Validation("api key must not be empty".into()));
        }

        Ok(blake3::keyed_hash(&self.hash_key, api_key.trim().as_bytes()).to_hex().to_string())
    }

    fn key_prefix(&self, api_key: &str) -> String {
        api_key.trim().chars().take(12).collect()
    }
}
