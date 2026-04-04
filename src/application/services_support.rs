use crate::{
    application::dto::{RegisterDiplomaRequest, RegisterUserRequest},
    domain::diploma::Diploma,
    error::AppError,
};

pub(super) fn validate_request(request: &RegisterDiplomaRequest) -> Result<(), AppError> {
    if request.student_full_name.trim().is_empty() {
        return Err(AppError::Validation("student_full_name is required".into()));
    }

    if request.student_number.trim().is_empty() {
        return Err(AppError::Validation("student_number is required".into()));
    }

    if request.diploma_number.trim().is_empty() {
        return Err(AppError::Validation("diploma_number is required".into()));
    }

    if request.degree.trim().is_empty() {
        return Err(AppError::Validation("degree is required".into()));
    }

    if request.program_name.trim().is_empty() {
        return Err(AppError::Validation("program_name is required".into()));
    }

    Ok(())
}

pub(super) fn validate_registration(request: &RegisterUserRequest) -> Result<(), AppError> {
    if !request.email.contains('@') {
        return Err(AppError::Validation("email must be valid".into()));
    }

    if request.full_name.trim().is_empty() {
        return Err(AppError::Validation("full_name is required".into()));
    }

    validate_new_password(&request.password)
}

pub(super) fn validate_new_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters long".into(),
        ));
    }

    Ok(())
}

pub(super) fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub(super) fn normalize_display_name(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn normalize_identifier(value: &str) -> String {
    value.trim().to_string()
}

pub(super) fn intersect_or_replace(
    existing: Option<Vec<Diploma>>,
    matches: Vec<Diploma>,
) -> Vec<Diploma> {
    match existing {
        Some(current) => current
            .into_iter()
            .filter(|item| matches.iter().any(|candidate| candidate.id == item.id))
            .collect(),
        None => matches,
    }
}
