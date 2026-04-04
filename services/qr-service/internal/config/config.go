package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

const (
	defaultPort       = "8090"
	defaultStorageDir = "/app/data"
	defaultPublicBase = "http://localhost:8090"
	defaultQRSize     = 512
)

type Config struct {
	Port          string
	StorageDir    string
	APIKey        string
	PublicBaseURL string
	DefaultSize   int
}

func Load() (Config, error) {
	cfg := Config{
		Port:          getenv("QR_SERVICE_PORT", defaultPort),
		StorageDir:    getenv("QR_STORAGE_DIR", defaultStorageDir),
		APIKey:        getenv("QR_SERVICE_API_KEY", ""),
		PublicBaseURL: "",
		DefaultSize:   getenvInt("QR_DEFAULT_SIZE", defaultQRSize),
	}

	if cfg.PublicBaseURL == "" {
		cfg.PublicBaseURL = getenv("QR_PUBLIC_BASE_URL", "http://localhost:"+cfg.Port)
	}

	if strings.TrimSpace(cfg.APIKey) == "" {
		return Config{}, fmt.Errorf("QR_SERVICE_API_KEY is required")
	}
	if strings.TrimSpace(cfg.Port) == "" {
		return Config{}, fmt.Errorf("QR_SERVICE_PORT must not be empty")
	}
	if strings.TrimSpace(cfg.StorageDir) == "" {
		return Config{}, fmt.Errorf("QR_STORAGE_DIR must not be empty")
	}
	if strings.TrimSpace(cfg.PublicBaseURL) == "" {
		return Config{}, fmt.Errorf("QR_PUBLIC_BASE_URL must not be empty")
	}
	if cfg.DefaultSize <= 0 {
		return Config{}, fmt.Errorf("QR_DEFAULT_SIZE must be greater than 0")
	}

	return cfg, nil
}

func getenv(key, fallback string) string {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	return value
}

func getenvInt(key string, fallback int) int {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}

	parsed, err := strconv.Atoi(value)
	if err != nil {
		return fallback
	}
	return parsed
}
