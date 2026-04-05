package qr

import (
	"os"
	"path/filepath"
	"testing"
)

func TestWriteFileCreatesNonEmptyPNG(t *testing.T) {
	tempDir := t.TempDir()
	filePath := filepath.Join(tempDir, "qr.png")

	if err := WriteFile("https://resume-vizor.test/diploma/123", "png", 256, filePath); err != nil {
		t.Fatalf("WriteFile returned error: %v", err)
	}

	info, err := os.Stat(filePath)
	if err != nil {
		t.Fatalf("stat failed: %v", err)
	}
	if info.Size() == 0 {
		t.Fatal("generated png file is empty")
	}

	content, err := os.ReadFile(filePath)
	if err != nil {
		t.Fatalf("read failed: %v", err)
	}
	if len(content) < 4 {
		t.Fatalf("generated png is unexpectedly short: %d bytes", len(content))
	}
	if string(content[:4]) != "\x89PNG" {
		t.Fatalf("generated file does not start with png signature: %v", content[:4])
	}
}

func TestWriteFileCreatesNonEmptySVG(t *testing.T) {
	tempDir := t.TempDir()
	filePath := filepath.Join(tempDir, "qr.svg")

	if err := WriteFile("https://resume-vizor.test/diploma/123", "svg", 256, filePath); err != nil {
		t.Fatalf("WriteFile returned error: %v", err)
	}

	content, err := os.ReadFile(filePath)
	if err != nil {
		t.Fatalf("read failed: %v", err)
	}
	if len(content) == 0 {
		t.Fatal("generated svg file is empty")
	}
	if string(content[:5]) != "<?xml" {
		t.Fatalf("generated svg does not start with xml declaration: %q", string(content[:5]))
	}
}
