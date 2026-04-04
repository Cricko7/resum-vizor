package qr

import (
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/skip2/go-qrcode"
)

func NormalizeFormat(value string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "svg":
		return "svg"
	default:
		return "png"
	}
}

func ContentTypeForFormat(format string) string {
	if format == "svg" {
		return "image/svg+xml"
	}
	return "image/png"
}

func WriteFile(payloadURL, format string, size int, path string) error {
	if format == "svg" {
		return writeSVGFile(payloadURL, size, path)
	}
	return qrcode.WriteFile(payloadURL, qrcode.Medium, size, path)
}

func writeSVGFile(payloadURL string, size int, path string) error {
	code, err := qrcode.New(payloadURL, qrcode.Medium)
	if err != nil {
		return err
	}

	bitmap := code.Bitmap()
	if len(bitmap) == 0 {
		return fmt.Errorf("empty bitmap")
	}

	cellSize := 8
	if size > 0 {
		cellSize = size / len(bitmap)
		if cellSize <= 0 {
			cellSize = 1
		}
	}

	canvasSize := len(bitmap) * cellSize
	var builder strings.Builder
	builder.WriteString(`<?xml version="1.0" encoding="UTF-8"?>`)
	builder.WriteString("\n")
	builder.WriteString(`<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="`)
	builder.WriteString(strconv.Itoa(canvasSize))
	builder.WriteString(`" height="`)
	builder.WriteString(strconv.Itoa(canvasSize))
	builder.WriteString(`" viewBox="0 0 `)
	builder.WriteString(strconv.Itoa(canvasSize))
	builder.WriteString(" ")
	builder.WriteString(strconv.Itoa(canvasSize))
	builder.WriteString(`">`)
	builder.WriteString("\n")
	builder.WriteString(`<rect width="100%" height="100%" fill="#ffffff"/>`)
	builder.WriteString("\n")

	for y, row := range bitmap {
		for x, dark := range row {
			if !dark {
				continue
			}
			builder.WriteString(`<rect x="`)
			builder.WriteString(strconv.Itoa(x * cellSize))
			builder.WriteString(`" y="`)
			builder.WriteString(strconv.Itoa(y * cellSize))
			builder.WriteString(`" width="`)
			builder.WriteString(strconv.Itoa(cellSize))
			builder.WriteString(`" height="`)
			builder.WriteString(strconv.Itoa(cellSize))
			builder.WriteString(`" fill="#000000"/>`)
			builder.WriteString("\n")
		}
	}

	builder.WriteString(`</svg>`)
	return os.WriteFile(path, []byte(builder.String()), 0o644)
}
