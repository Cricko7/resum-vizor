package main

import (
	"log"
	"net/http"
	"os"

	"resume-vizor-qr-service/internal/config"
	"resume-vizor-qr-service/internal/httpapi"
	"resume-vizor-qr-service/internal/service"
	"resume-vizor-qr-service/internal/storage"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("failed to load config: %v", err)
	}

	if err := os.MkdirAll(cfg.StorageDir, os.ModePerm); err != nil {
		log.Fatalf("failed to create storage dir: %v", err)
	}

	store := storage.NewMemoryStore()
	qrService := service.New(cfg, store)
	router := httpapi.NewRouter(cfg, qrService)

	addr := ":" + cfg.Port
	log.Printf("QR service started on %s", addr)
	log.Fatal(http.ListenAndServe(addr, router))
}
