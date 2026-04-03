use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signer, SigningKey};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    application::ports::DiplomaSigner,
    domain::ids::UniversityId,
    error::AppError,
};

pub struct UniversityRecordSigner {
    master_secret: SecretString,
}

impl UniversityRecordSigner {
    pub fn new(secret: &SecretString) -> Self {
        Self {
            master_secret: secret.clone(),
        }
    }

    fn signing_key_for_university(&self, university_id: UniversityId) -> SigningKey {
        let seed_input = format!(
            "{}:{}",
            self.master_secret.expose_secret(),
            university_id.0
        );
        let seed = blake3::hash(seed_input.as_bytes());
        SigningKey::from_bytes(seed.as_bytes())
    }
}

impl DiplomaSigner for UniversityRecordSigner {
    fn sign_record_hash(
        &self,
        university_id: UniversityId,
        record_hash: &str,
    ) -> Result<String, AppError> {
        let signing_key = self.signing_key_for_university(university_id);
        let signature = signing_key.sign(record_hash.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(signature.to_bytes()))
    }
}
