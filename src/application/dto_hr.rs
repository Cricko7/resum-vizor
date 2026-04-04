use serde::{Deserialize, Serialize};

use super::dto_diploma::PublicDiplomaView;

#[derive(Debug, Clone, Deserialize)]
pub struct HrRegistrySearchRequest {
    pub diploma_number: Option<String>,
    pub university_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HrRegistrySearchResponse {
    pub items: Vec<PublicDiplomaView>,
}
