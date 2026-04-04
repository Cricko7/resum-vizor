CREATE TABLE IF NOT EXISTS ats_api_keys (
    id UUID PRIMARY KEY,
    hr_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    scope TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    daily_request_limit BIGINT NOT NULL,
    burst_request_limit BIGINT NOT NULL,
    burst_window_seconds BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ NULL,
    revoked_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_ats_api_keys_hr_user_id ON ats_api_keys(hr_user_id);
CREATE INDEX IF NOT EXISTS idx_ats_api_keys_key_hash ON ats_api_keys(key_hash);
