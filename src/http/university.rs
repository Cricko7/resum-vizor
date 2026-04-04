use std::io::Cursor;

use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use calamine::{Data, Reader, Xlsx};

use crate::{
    application::dto::{
        DiplomaImportResponse, DiplomaStatusResponse, RegisterDiplomaRequest,
        RegisterDiplomaResponse, RegistryDiplomaRow,
    },
    domain::ids::DiplomaId,
    error::AppError,
    http::{AppState, middleware::AuthenticatedUser},
};

pub async fn register_diploma(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Json(payload): Json<RegisterDiplomaRequest>,
) -> Result<Json<RegisterDiplomaResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    let university_id = user
        .university_id
        .ok_or_else(|| AppError::Forbidden("university account must be linked to university_id".into()))?;
    let university_code = user
        .university_code
        .ok_or_else(|| AppError::Forbidden("university account must be linked to university_code".into()))?;

    let diploma = state
        .diploma_service
        .register_diploma(university_id, university_code, payload)
        .await?;
    Ok(Json(diploma.into()))
}

pub async fn import_diplomas(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<DiplomaImportResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    let university_id = user
        .university_id
        .ok_or_else(|| AppError::Forbidden("university account must be linked to university_id".into()))?;
    let university_code = user
        .university_code
        .ok_or_else(|| AppError::Forbidden("university account must be linked to university_code".into()))?;

    let mut filename = None;
    let mut file_bytes = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::Validation("invalid multipart payload".into()))? {
        if field.name() == Some("file") {
            filename = field.file_name().map(|value| value.to_string());
            file_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::Validation("failed to read uploaded file".into()))?,
            );
            break;
        }
    }

    let filename = filename.ok_or_else(|| AppError::Validation("multipart field 'file' is required".into()))?;
    let bytes = file_bytes.ok_or_else(|| AppError::Validation("uploaded file is empty".into()))?;
    let rows = parse_registry_file(&filename, &bytes)?;
    let response = state
        .diploma_service
        .import_registry(university_id, university_code, rows)
        .await;

    Ok(Json(response))
}

pub async fn revoke_diploma(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
) -> Result<Json<DiplomaStatusResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    let university_id = user
        .university_id
        .ok_or_else(|| AppError::Forbidden("university account must be linked to university_id".into()))?;

    let diploma = state
        .diploma_service
        .revoke_diploma(university_id, diploma_id)
        .await?;

    Ok(Json(DiplomaStatusResponse {
        diploma_id: diploma.id,
        status: diploma.status,
        revoked_at: diploma.revoked_at,
    }))
}

pub async fn restore_diploma(
    State(state): State<AppState>,
    authenticated: AuthenticatedUser,
    Path(diploma_id): Path<DiplomaId>,
) -> Result<Json<DiplomaStatusResponse>, AppError> {
    let user = state.auth_service.get_user(authenticated.user_id).await?;
    let university_id = user
        .university_id
        .ok_or_else(|| AppError::Forbidden("university account must be linked to university_id".into()))?;

    let diploma = state
        .diploma_service
        .restore_diploma(university_id, diploma_id)
        .await?;

    Ok(Json(DiplomaStatusResponse {
        diploma_id: diploma.id,
        status: diploma.status,
        revoked_at: diploma.revoked_at,
    }))
}

fn parse_registry_file(filename: &str, bytes: &[u8]) -> Result<Vec<RegistryDiplomaRow>, AppError> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".csv") {
        parse_csv(bytes)
    } else if lower.ends_with(".xlsx") {
        parse_xlsx(bytes)
    } else {
        Err(AppError::Validation(
            "unsupported file format, expected .csv or .xlsx".into(),
        ))
    }
}

fn parse_csv(bytes: &[u8]) -> Result<Vec<RegistryDiplomaRow>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(Cursor::new(bytes));
    let mut rows = Vec::new();

    for (index, result) in reader.records().enumerate() {
        let record = result.map_err(|_| AppError::Validation("failed to parse csv registry".into()))?;
        if index == 0 && is_registry_header(&record) {
            continue;
        }
        rows.push(RegistryDiplomaRow {
            student_full_name: required_csv_value(&record, 0, "fio")?,
            student_number: required_csv_value(&record, 1, "student_number")?,
            graduation_year: parse_year(&required_csv_value(&record, 2, "year")?)?,
            program_name: required_csv_value(&record, 3, "specialnost")?,
            diploma_number: required_csv_value(&record, 4, "diploma_number")?,
        });
    }

    Ok(rows)
}

fn parse_xlsx(bytes: &[u8]) -> Result<Vec<RegistryDiplomaRow>, AppError> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook =
        Xlsx::new(cursor).map_err(|_| AppError::Validation("failed to parse xlsx registry".into()))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::Validation("xlsx file does not contain worksheets".into()))?
        .map_err(|_| AppError::Validation("failed to read first worksheet".into()))?;

    let mut rows = Vec::new();
    for row in range.rows().skip(1) {
        rows.push(RegistryDiplomaRow {
            student_full_name: required_excel_value(row, 0, "fio")?,
            student_number: required_excel_value(row, 1, "student_number")?,
            graduation_year: parse_year(&required_excel_value(row, 2, "year")?)?,
            program_name: required_excel_value(row, 3, "specialnost")?,
            diploma_number: required_excel_value(row, 4, "diploma_number")?,
        });
    }

    Ok(rows)
}

fn is_registry_header(record: &csv::StringRecord) -> bool {
    let values = [
        record.get(0),
        record.get(1),
        record.get(2),
        record.get(3),
        record.get(4),
    ];

    values[0].map(normalize_header_cell) == Some("fio".to_string())
        && values[1].map(normalize_header_cell) == Some("student_number".to_string())
        && values[2].map(normalize_header_cell) == Some("year".to_string())
        && values[3].map(normalize_header_cell) == Some("specialnost".to_string())
        && values[4].map(normalize_header_cell) == Some("diploma_number".to_string())
}

fn required_csv_value(record: &csv::StringRecord, index: usize, field: &str) -> Result<String, AppError> {
    let value = record
        .get(index)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Validation(format!("csv field '{field}' is required")))?;

    Ok(value.to_string())
}

fn required_excel_value(row: &[Data], index: usize, field: &str) -> Result<String, AppError> {
    let value = row
        .get(index)
        .map(cell_to_string)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Validation(format!("excel field '{field}' is required")))?;

    Ok(value.trim().to_string())
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(value) => value.clone(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                (*value as i64).to_string()
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        _ => String::new(),
    }
}

fn parse_year(value: &str) -> Result<i32, AppError> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|_| AppError::Validation(format!("invalid graduation year: {value}")))
}

fn normalize_header_cell(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::parse_csv;

    #[test]
    fn csv_import_supports_documented_header_row() {
        let csv = b"fio,student_number,year,specialnost,diploma_number\nIvan Petrov,ST-1001,2026,Computer Science,DP-2026-0001\n";

        let rows = parse_csv(csv).expect("csv with header should parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].student_full_name, "Ivan Petrov");
        assert_eq!(rows[0].student_number, "ST-1001");
        assert_eq!(rows[0].graduation_year, 2026);
    }

    #[test]
    fn csv_import_without_header_is_still_supported() {
        let csv = b"Ivan Petrov,ST-1001,2026,Computer Science,DP-2026-0001\n";

        let rows = parse_csv(csv).expect("csv without header should parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].diploma_number, "DP-2026-0001");
    }
}
