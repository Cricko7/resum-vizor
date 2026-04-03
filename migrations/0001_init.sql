CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    full_name TEXT NOT NULL,
    student_number TEXT NULL,
    role TEXT NOT NULL,
    university_id UUID NULL,
    university_code TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS diplomas (
    id UUID PRIMARY KEY,
    university_id UUID NOT NULL,
    student_id UUID NOT NULL,
    certificate_id UUID NOT NULL UNIQUE,
    student_account_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    university_code TEXT NOT NULL,
    student_number_last4 TEXT NOT NULL,
    diploma_number_last4 TEXT NOT NULL,
    record_hash TEXT NOT NULL UNIQUE,
    university_signature TEXT NOT NULL,
    signature_algorithm TEXT NOT NULL,
    status TEXT NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    university_code_hash TEXT NOT NULL,
    student_full_name_hash TEXT NOT NULL,
    student_number_hash TEXT NOT NULL,
    student_birth_date_hash TEXT NULL,
    diploma_number_hash TEXT NOT NULL,
    verification_lookup_hash TEXT NOT NULL,
    degree_hash TEXT NOT NULL,
    program_hash TEXT NOT NULL,
    graduation_date_hash TEXT NOT NULL,
    honors_hash TEXT NOT NULL,
    canonical_document_hash TEXT NOT NULL UNIQUE,
    issued_at DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_diplomas_student_account_id ON diplomas(student_account_id);
CREATE INDEX IF NOT EXISTS idx_diplomas_student_full_name_hash ON diplomas(student_full_name_hash);
CREATE INDEX IF NOT EXISTS idx_diplomas_student_number_hash ON diplomas(student_number_hash);
CREATE INDEX IF NOT EXISTS idx_diplomas_diploma_number_hash ON diplomas(diploma_number_hash);
CREATE INDEX IF NOT EXISTS idx_diplomas_university_code_hash ON diplomas(university_code_hash);
CREATE INDEX IF NOT EXISTS idx_diplomas_verification_lookup_hash ON diplomas(verification_lookup_hash);
