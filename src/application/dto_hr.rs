use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::dto_diploma::PublicDiplomaView;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct HrRegistrySearchRequest {
    pub diploma_number: Option<String>,
    pub university_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HrRegistrySearchResponse {
    pub items: Vec<PublicDiplomaView>,
}
