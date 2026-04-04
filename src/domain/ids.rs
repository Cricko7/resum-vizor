use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

typed_id!(UniversityId);
typed_id!(StudentId);
typed_id!(DiplomaId);
typed_id!(CertificateId);
typed_id!(UserId);
typed_id!(AtsApiKeyId);
typed_id!(QrCodeId);
typed_id!(QrJobId);
