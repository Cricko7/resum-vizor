use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{
    ids::{UniversityId, UserId},
    user::{User, UserRole},
};

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub student_number: Option<String>,
    pub role: UserRole,
    pub university_id: Option<UniversityId>,
    pub university_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: UserId,
    pub email: String,
    pub full_name: String,
    pub student_number: Option<String>,
    pub role: UserRole,
    pub university_id: Option<UniversityId>,
    pub university_code: Option<String>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            full_name: value.full_name,
            student_number: value.student_number,
            role: value.role,
            university_id: value.university_id,
            university_code: value.university_code,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in_seconds: i64,
    pub user: UserResponse,
}
