use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use chrono::Utc;
use redis::{AsyncCommands, aio::ConnectionManager};
use secrecy::ExposeSecret;
use tokio::sync::RwLock;

use crate::config::RedisSettings;

#[async_trait]
pub trait HrRateLimiter: Send + Sync {
    async fn allow(&self, key: &str, max_requests: usize, window: Duration) -> bool;
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct SimpleRateLimiter {
    daily_buckets: Arc<RwLock<HashMap<String, usize>>>,
    rolling_buckets: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl SimpleRateLimiter {
    pub fn new() -> Self {
        Self {
            daily_buckets: Arc::new(RwLock::new(HashMap::new())),
            rolling_buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl HrRateLimiter for SimpleRateLimiter {
    async fn allow(&self, key: &str, max_requests: usize, window: Duration) -> bool {
        if is_daily_window(window) {
            let mut buckets = self.daily_buckets.write().await;
            let bucket_key = daily_bucket_key(key);
            let bucket = buckets.entry(bucket_key).or_default();

            if *bucket >= max_requests {
                return false;
            }

            *bucket += 1;
            return true;
        }

        let mut buckets = self.rolling_buckets.write().await;
        let now = Instant::now();
        let bucket_key = rolling_bucket_key(key, max_requests, window);
        let bucket = buckets.entry(bucket_key).or_default();
        bucket.retain(|instant| now.duration_since(*instant) <= window);

        if bucket.len() >= max_requests {
            return false;
        }

        bucket.push(now);
        true
    }

    fn backend_name(&self) -> &'static str {
        "in_memory"
    }
}

#[derive(Clone)]
pub struct RedisRateLimiter {
    manager: ConnectionManager,
    prefix: String,
}

impl RedisRateLimiter {
    pub async fn connect(settings: &RedisSettings) -> anyhow::Result<Self> {
        let client = redis::Client::open(settings.url.expose_secret())?;
        let manager = client.get_connection_manager().await?;

        Ok(Self {
            manager,
            prefix: settings.rate_limit_prefix.clone(),
        })
    }

    fn key(&self, key: &str, max_requests: usize, window: Duration) -> String {
        if is_daily_window(window) {
            return format!(
                "{}:daily:{}:{}:{}",
                self.prefix,
                current_utc_day_key(),
                max_requests,
                key,
            );
        }

        format!(
            "{}:rolling:{}:{}:{}",
            self.prefix,
            window.as_secs(),
            max_requests,
            key,
        )
    }
}

#[async_trait]
impl HrRateLimiter for RedisRateLimiter {
    async fn allow(&self, key: &str, max_requests: usize, window: Duration) -> bool {
        let redis_key = self.key(key, max_requests, window);
        let mut connection = self.manager.clone();

        let requests: isize = match connection.incr(&redis_key, 1).await {
            Ok(value) => value,
            Err(_) => return false,
        };

        if requests == 1 {
            let ttl_seconds = if is_daily_window(window) {
                seconds_until_next_utc_day()
            } else {
                window.as_secs().max(1)
            };
            let _: Result<bool, _> = connection.expire(&redis_key, ttl_seconds as i64).await;
        }

        requests as usize <= max_requests
    }

    fn backend_name(&self) -> &'static str {
        "redis"
    }
}

fn daily_bucket_key(key: &str) -> String {
    bucket_key_for_day(key, current_utc_day_key())
}

fn rolling_bucket_key(key: &str, max_requests: usize, window: Duration) -> String {
    format!("{}:{}:{}", key, max_requests, window.as_secs())
}

fn current_utc_day_key() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

fn bucket_key_for_day(key: &str, day_key: String) -> String {
    format!("{}:{}", day_key, key)
}

fn is_daily_window(window: Duration) -> bool {
    window.as_secs() >= 86_400
}

fn seconds_until_next_utc_day() -> u64 {
    let now = Utc::now();
    let tomorrow = now.date_naive().succ_opt().unwrap_or(now.date_naive());
    let midnight = tomorrow
        .and_hms_opt(0, 0, 0)
        .unwrap_or_else(|| now.naive_utc());
    let seconds = (midnight - now.naive_utc()).num_seconds().max(1);
    seconds as u64
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{HrRateLimiter, SimpleRateLimiter, bucket_key_for_day};

    #[tokio::test]
    async fn denies_requests_after_daily_limit() {
        let limiter = SimpleRateLimiter::new();

        assert!(limiter.allow("client-1", 2, Duration::from_millis(30)).await);
        assert!(limiter.allow("client-1", 2, Duration::from_millis(30)).await);
        assert!(!limiter.allow("client-1", 2, Duration::from_millis(30)).await);
    }

    #[test]
    fn bucket_key_changes_between_days() {
        let first = bucket_key_for_day("client-1", "2026-04-04".to_string());
        let second = bucket_key_for_day("client-1", "2026-04-05".to_string());

        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn denies_requests_after_burst_limit_and_allows_after_window() {
        let limiter = SimpleRateLimiter::new();

        assert!(limiter.allow("client-1", 2, Duration::from_millis(30)).await);
        assert!(limiter.allow("client-1", 2, Duration::from_millis(30)).await);
        assert!(!limiter.allow("client-1", 2, Duration::from_millis(30)).await);

        tokio::time::sleep(Duration::from_millis(40)).await;

        assert!(limiter.allow("client-1", 2, Duration::from_millis(30)).await);
    }
}
