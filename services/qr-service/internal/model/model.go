package model

import "time"

type CreateJobRequest struct {
	ExternalID     string `json:"external_id"`
	PayloadURL     string `json:"payload_url"`
	Format         string `json:"format"`
	Size           int    `json:"size"`
	TTLSeconds     int64  `json:"ttl_seconds"`
	IdempotencyKey string `json:"idempotency_key"`
}

type JobResponse struct {
	JobID  string  `json:"job_id"`
	Status string  `json:"status"`
	QRID   *string `json:"qr_id"`
	Error  *string `json:"error"`
}

type QrMetadataResponse struct {
	QRID        string     `json:"qr_id"`
	ExternalID  string     `json:"external_id"`
	Status      string     `json:"status"`
	Format      string     `json:"format"`
	Size        int        `json:"size"`
	ExpiresAt   *time.Time `json:"expires_at"`
	DownloadURL *string    `json:"download_url"`
	CreatedAt   time.Time  `json:"created_at"`
}

type DeleteResponse struct {
	Deleted bool `json:"deleted"`
}

type QrRecord struct {
	QRID        string
	ExternalID  string
	Status      string
	Format      string
	Size        int
	ExpiresAt   *time.Time
	DownloadURL *string
	CreatedAt   time.Time
	FileName    string
}
