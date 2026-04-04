# syntax=docker/dockerfile:1

FROM rust:1.91-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/resume-vizor /usr/local/bin/resume-vizor
COPY migrations ./migrations

EXPOSE 8080

CMD ["resume-vizor"]

