package storage

import (
	"sync"

	"resume-vizor-qr-service/internal/model"
)

type MemoryStore struct {
	mu                 sync.RWMutex
	jobsByID           map[string]model.JobResponse
	qrByID             map[string]model.QrRecord
	jobIDByIdempotency map[string]string
}

func NewMemoryStore() *MemoryStore {
	return &MemoryStore{
		jobsByID:           make(map[string]model.JobResponse),
		qrByID:             make(map[string]model.QrRecord),
		jobIDByIdempotency: make(map[string]string),
	}
}

func (s *MemoryStore) FindJobByIdempotencyKey(key string) (model.JobResponse, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	jobID, ok := s.jobIDByIdempotency[key]
	if !ok {
		return model.JobResponse{}, false
	}

	job, ok := s.jobsByID[jobID]
	return job, ok
}

func (s *MemoryStore) SaveJob(idempotencyKey string, job model.JobResponse) {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.jobsByID[job.JobID] = job
	if idempotencyKey != "" {
		s.jobIDByIdempotency[idempotencyKey] = job.JobID
	}
}

func (s *MemoryStore) GetJob(jobID string) (model.JobResponse, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	job, ok := s.jobsByID[jobID]
	return job, ok
}

func (s *MemoryStore) SaveQR(record model.QrRecord) {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.qrByID[record.QRID] = record
}

func (s *MemoryStore) GetQR(qrID string) (model.QrRecord, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	record, ok := s.qrByID[qrID]
	return record, ok
}

func (s *MemoryStore) UpdateQR(record model.QrRecord) {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.qrByID[record.QRID] = record
}

func (s *MemoryStore) DeleteQR(qrID string) (model.QrRecord, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()

	record, ok := s.qrByID[qrID]
	if !ok {
		return model.QrRecord{}, false
	}

	delete(s.qrByID, qrID)
	for jobID, job := range s.jobsByID {
		if job.QRID != nil && *job.QRID == qrID {
			delete(s.jobsByID, jobID)
		}
	}

	return record, true
}
