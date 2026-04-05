package httpapi

import (
	"encoding/json"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/go-chi/chi/v5"

	"resume-vizor-qr-service/internal/config"
	"resume-vizor-qr-service/internal/model"
	"resume-vizor-qr-service/internal/qr"
	"resume-vizor-qr-service/internal/service"
)

func NewRouter(cfg config.Config, qrService *service.Service) http.Handler {
	r := chi.NewRouter()
	r.Use(apiKeyMiddleware(cfg.APIKey))

	r.Post("/api/v1/qr/jobs", func(w http.ResponseWriter, r *http.Request) {
		var req model.CreateJobRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid json", http.StatusBadRequest)
			return
		}

		if strings.TrimSpace(req.ExternalID) == "" || strings.TrimSpace(req.PayloadURL) == "" {
			http.Error(w, "external_id and payload_url are required", http.StatusBadRequest)
			return
		}

		job := qrService.CreateJob(req)
		writeJSON(w, http.StatusCreated, job)
	})

	r.Get("/api/v1/qr/jobs/{job_id}", func(w http.ResponseWriter, r *http.Request) {
		job, ok := qrService.GetJob(chi.URLParam(r, "job_id"))
		if !ok {
			http.Error(w, "not found", http.StatusNotFound)
			return
		}

		writeJSON(w, http.StatusOK, job)
	})

	r.Get("/api/v1/qr/{qr_id}", func(w http.ResponseWriter, r *http.Request) {
		record, ok := qrService.GetQRMetadata(chi.URLParam(r, "qr_id"))
		if !ok {
			http.Error(w, "not found", http.StatusNotFound)
			return
		}

		writeJSON(w, http.StatusOK, record)
	})

	r.Get("/api/v1/qr/{qr_id}/content", func(w http.ResponseWriter, r *http.Request) {
		record, filePath, ok := qrService.GetQRFile(chi.URLParam(r, "qr_id"))
		if !ok {
			http.Error(w, "not found", http.StatusNotFound)
			return
		}

		if _, err := os.Stat(filePath); os.IsNotExist(err) {
			http.Error(w, "not found", http.StatusNotFound)
			return
		}

		content, err := os.ReadFile(filePath)
		if err != nil {
			http.Error(w, "failed to read qr file", http.StatusInternalServerError)
			return
		}
		if len(content) == 0 {
			http.Error(w, "empty qr file", http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", qr.ContentTypeForFormat(record.Format))
		w.Header().Set("Content-Length", strconv.Itoa(len(content)))
		w.Header().Set("Cache-Control", "private, max-age=60")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write(content)
	})

	r.Delete("/api/v1/qr/{qr_id}", func(w http.ResponseWriter, r *http.Request) {
		filePath, ok := qrService.DeleteQR(chi.URLParam(r, "qr_id"))
		if !ok {
			http.Error(w, "not found", http.StatusNotFound)
			return
		}

		_ = os.Remove(filePath)
		writeJSON(w, http.StatusOK, model.DeleteResponse{Deleted: true})
	})

	return r
}

func apiKeyMiddleware(expected string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Header.Get("x-api-key") != expected {
				http.Error(w, "unauthorized", http.StatusUnauthorized)
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

func writeJSON(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(value)
}
