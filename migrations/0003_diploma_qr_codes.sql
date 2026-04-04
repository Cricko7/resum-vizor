CREATE TABLE IF NOT EXISTS diploma_qr_codes (
    diploma_id UUID PRIMARY KEY REFERENCES diplomas(id) ON DELETE CASCADE,
    student_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    certificate_id UUID NOT NULL REFERENCES diplomas(certificate_id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    payload_url TEXT NOT NULL,
    format TEXT NOT NULL,
    size INTEGER NOT NULL,
    ttl_seconds BIGINT NOT NULL,
    status TEXT NOT NULL,
    external_job_id TEXT NULL,
    external_qr_id TEXT NULL,
    external_download_url TEXT NULL,
    expires_at TIMESTAMPTZ NULL,
    error_message TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_diploma_qr_codes_student_user_id
    ON diploma_qr_codes(student_user_id);

CREATE INDEX IF NOT EXISTS idx_diploma_qr_codes_external_qr_id
    ON diploma_qr_codes(external_qr_id);

CREATE INDEX IF NOT EXISTS idx_diploma_qr_codes_external_job_id
    ON diploma_qr_codes(external_job_id);
