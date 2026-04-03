use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::ids::{UniversityId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    University,
    Student,
    Hr,
}

impl UserRole {
    pub fn as_header_value(&self) -> &'static str {
        match self {
            Self::University => "university",
            Self::Student => "student",
            Self::Hr => "hr",
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_header_value())
    }
}

impl std::str::FromStr for UserRole {
    type Err = crate::error::AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "university" => Ok(Self::University),
            "student" => Ok(Self::Student),
            "hr" => Ok(Self::Hr),
            _ => Err(crate::error::AppError::Validation(
                "role must be one of: university, student, hr".into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub student_number: Option<String>,
    pub role: UserRole,
    pub university_id: Option<UniversityId>,
    pub university_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: String,
        password_hash: String,
        full_name: String,
        student_number: Option<String>,
        role: UserRole,
        university_id: Option<UniversityId>,
        university_code: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            email,
            password_hash,
            full_name,
            student_number,
            role,
            university_id,
            university_code,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_password(&mut self, password_hash: String) {
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
    }
}
