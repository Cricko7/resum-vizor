use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::UniversityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct University {
    pub id: UniversityId,
    pub external_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
