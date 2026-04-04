use std::sync::Arc;

use crate::{
    application::{
        dto::{AuthResponse, ChangePasswordRequest, LoginRequest, RegisterUserRequest},
        ports::{AppRepository, JwtProvider, PasswordHasher},
    },
    domain::{
        ids::UserId,
        user::{User, UserRole},
    },
    error::AppError,
};

use super::services_support::{
    normalize_display_name, normalize_email, normalize_identifier, validate_new_password,
    validate_registration,
};

pub struct AuthService {
    repository: Arc<dyn AppRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
    jwt_provider: Arc<dyn JwtProvider>,
    jwt_ttl_minutes: i64,
}

impl AuthService {
    pub fn new(
        repository: Arc<dyn AppRepository>,
        password_hasher: Arc<dyn PasswordHasher>,
        jwt_provider: Arc<dyn JwtProvider>,
        jwt_ttl_minutes: i64,
    ) -> Self {
        Self {
            repository,
            password_hasher,
            jwt_provider,
            jwt_ttl_minutes,
        }
    }

    pub async fn register(&self, request: RegisterUserRequest) -> Result<AuthResponse, AppError> {
        validate_registration(&request)?;

        if self
            .repository
            .find_user_by_email(&normalize_email(&request.email))
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("user with this email already exists".into()));
        }

        if request.role == UserRole::University && request.university_id.is_none() {
            return Err(AppError::Validation(
                "university_id is required for university role".into(),
            ));
        }

        if request.role == UserRole::University
            && request
                .university_code
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(AppError::Validation(
                "university_code is required for university role".into(),
            ));
        }

        if request.role == UserRole::Student
            && request
                .student_number
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(AppError::Validation(
                "student_number is required for student role".into(),
            ));
        }

        let student_number = if request.role == UserRole::Student {
            request.student_number.as_deref().map(normalize_identifier)
        } else {
            None
        };
        let university_id = if request.role == UserRole::University {
            request.university_id
        } else {
            None
        };
        let university_code = if request.role == UserRole::University {
            request.university_code.as_deref().map(normalize_identifier)
        } else {
            None
        };

        let user = User::new(
            normalize_email(&request.email),
            self.password_hasher.hash_password(&request.password)?,
            normalize_display_name(&request.full_name),
            student_number,
            request.role,
            university_id,
            university_code,
        );
        let user = self.repository.create_user(user).await?;
        self.build_auth_response(user)
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        let email = normalize_email(&request.email);
        let user = self
            .repository
            .find_user_by_email(&email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        self.password_hasher
            .verify_password(&request.password, &user.password_hash)?;

        self.build_auth_response(user)
    }

    pub async fn change_password(
        &self,
        user_id: UserId,
        request: ChangePasswordRequest,
    ) -> Result<(), AppError> {
        validate_new_password(&request.new_password)?;

        if request.current_password == request.new_password {
            return Err(AppError::Validation(
                "new_password must be different from current_password".into(),
            ));
        }

        let mut user = self
            .repository
            .find_user_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        self.password_hasher
            .verify_password(&request.current_password, &user.password_hash)?;

        let new_hash = self.password_hasher.hash_password(&request.new_password)?;
        user.update_password(new_hash);
        self.repository.update_user(user).await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: UserId) -> Result<User, AppError> {
        self.repository
            .find_user_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)
    }

    fn build_auth_response(&self, user: User) -> Result<AuthResponse, AppError> {
        let access_token = self.jwt_provider.issue_token(&user)?;

        Ok(AuthResponse {
            access_token,
            token_type: "Bearer",
            expires_in_seconds: self.jwt_ttl_minutes * 60,
            user: user.into(),
        })
    }
}
