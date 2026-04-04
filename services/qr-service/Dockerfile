FROM golang:1.24.2-alpine AS builder

WORKDIR /src

COPY services/qr-service/go.mod ./
RUN go mod download

COPY services/qr-service/ ./

RUN CGO_ENABLED=0 GOOS=linux go build -o /out/qr-service ./main.go

FROM alpine:3.21

WORKDIR /app

RUN adduser -D -u 10001 appuser && mkdir -p /app/data && chown -R appuser:appuser /app

COPY --from=builder /out/qr-service /usr/local/bin/qr-service

USER appuser

ENV QR_SERVICE_PORT=8090
ENV QR_STORAGE_DIR=/app/data

EXPOSE 8090

CMD ["qr-service"]
