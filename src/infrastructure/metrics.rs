use std::{sync::OnceLock, time::Instant};

use axum::{extract::MatchedPath, http::Request, middleware::Next, response::Response};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, TextEncoder,
    register_histogram_vec, register_int_counter_vec, register_int_gauge,
};

fn http_requests_total() -> &'static IntCounterVec {
    static METRIC: OnceLock<IntCounterVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_counter_vec!(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "path", "status"]
        )
        .expect("http_requests_total metric registration must succeed")
    })
}

fn http_request_duration_seconds() -> &'static HistogramVec {
    static METRIC: OnceLock<HistogramVec> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_histogram_vec!(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds"
            ),
            &["method", "path", "status"]
        )
        .expect("http_request_duration_seconds metric registration must succeed")
    })
}

fn http_requests_in_flight() -> &'static IntGauge {
    static METRIC: OnceLock<IntGauge> = OnceLock::new();
    METRIC.get_or_init(|| {
        register_int_gauge!(
            "http_requests_in_flight",
            "Current number of in-flight HTTP requests"
        )
        .expect("http_requests_in_flight metric registration must succeed")
    })
}

pub async fn metrics_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    http_requests_in_flight().inc();
    let started_at = Instant::now();

    let method = request.method().as_str().to_string();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path())
        .to_string();

    let response = next.run(request).await;
    let status = response.status().as_u16().to_string();
    let elapsed = started_at.elapsed().as_secs_f64();

    http_requests_total()
        .with_label_values(&[&method, &path, &status])
        .inc();
    http_request_duration_seconds()
        .with_label_values(&[&method, &path, &status])
        .observe(elapsed);
    http_requests_in_flight().dec();

    response
}

pub fn render_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("prometheus encoder must succeed");
    String::from_utf8(buffer).expect("prometheus metrics must be valid utf-8")
}
