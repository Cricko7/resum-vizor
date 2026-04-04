package service

import (
	"fmt"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/uuid"

	"resume-vizor-qr-service/internal/config"
	"resume-vizor-qr-service/internal/model"
	"resume-vizor-qr-service/internal/qr"
	"resume-vizor-qr-service/internal/storage"
)

type Service struct {
	cfg   config.Config
	store *storage.MemoryStore
}

func New(cfg config.Config, store *storage.MemoryStore) *Service {
	return &Service{
		cfg:   cfg,
		store: store,
	}
}

func (s *Service) CreateJob(req model.CreateJobRequest) model.JobResponse {
	if key := strings.TrimSpace(req.IdempotencyKey); key != "" {
		if job, ok := s.store.FindJobByIdempotencyKey(key); ok {
			return job
		}
	}

	format := qr.NormalizeFormat(req.Format)
	size := req.Size
	if size <= 0 {
		size = s.cfg.DefaultSize
	}

	jobID := uuid.NewString()
	qrID := uuid.NewString()
	fileName := fmt.Sprintf("%s.%s", qrID, format)
	filePath := filepath.Join(s.cfg.StorageDir, fileName)
	now := time.Now().UTC()

	if err := qr.WriteFile(req.PayloadURL, format, size, filePath); err != nil {
		message := err.Error()
		job := model.JobResponse{
			JobID:  jobID,
			Status: "failed",
			QRID:   nil,
			Error:  &message,
		}
		s.store.SaveJob(req.IdempotencyKey, job)
		return job
	}

	var expiresAt *time.Time
	if req.TTLSeconds > 0 {
		value := now.Add(time.Duration(req.TTLSeconds) * time.Second)
		expiresAt = &value
	}

	downloadURL := fmt.Sprintf("%s/api/v1/qr/%s/content", strings.TrimRight(s.cfg.PublicBaseURL, "/"), qrID)
	job := model.JobResponse{
		JobID:  jobID,
		Status: "ready",
		QRID:   &qrID,
		Error:  nil,
	}
	record := model.QrRecord{
		QRID:        qrID,
		ExternalID:  req.ExternalID,
		Status:      "ready",
		Format:      format,
		Size:        size,
		ExpiresAt:   expiresAt,
		DownloadURL: &downloadURL,
		CreatedAt:   now,
		FileName:    fileName,
	}

	s.store.SaveJob(req.IdempotencyKey, job)
	s.store.SaveQR(record)
	return job
}

func (s *Service) GetJob(jobID string) (model.JobResponse, bool) {
	return s.store.GetJob(jobID)
}

func (s *Service) GetQRMetadata(qrID string) (model.QrMetadataResponse, bool) {
	record, ok := s.store.GetQR(qrID)
	if !ok {
		return model.QrMetadataResponse{}, false
	}

	record = s.refreshExpiration(record)
	return model.QrMetadataResponse{
		QRID:        record.QRID,
		ExternalID:  record.ExternalID,
		Status:      record.Status,
		Format:      record.Format,
		Size:        record.Size,
		ExpiresAt:   record.ExpiresAt,
		DownloadURL: record.DownloadURL,
		CreatedAt:   record.CreatedAt,
	}, true
}

func (s *Service) GetQRFile(qrID string) (model.QrRecord, string, bool) {
	record, ok := s.store.GetQR(qrID)
	if !ok {
		return model.QrRecord{}, "", false
	}

	record = s.refreshExpiration(record)
	if record.Status == "expired" {
		return model.QrRecord{}, "", false
	}

	return record, filepath.Join(s.cfg.StorageDir, record.FileName), true
}

func (s *Service) DeleteQR(qrID string) (string, bool) {
	record, ok := s.store.DeleteQR(qrID)
	if !ok {
		return "", false
	}

	return filepath.Join(s.cfg.StorageDir, record.FileName), true
}

func (s *Service) refreshExpiration(record model.QrRecord) model.QrRecord {
	if record.ExpiresAt != nil && time.Now().UTC().After(*record.ExpiresAt) && record.Status != "expired" {
		record.Status = "expired"
		s.store.UpdateQR(record)
	}

	return record
}
