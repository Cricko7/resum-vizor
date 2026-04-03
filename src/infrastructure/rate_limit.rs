use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SimpleRateLimiter {
    max_requests: usize,
    window: Duration,
    buckets: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl SimpleRateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn allow(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();
        let bucket = buckets.entry(key.to_string()).or_default();
        bucket.retain(|instant| now.duration_since(*instant) <= self.window);

        if bucket.len() >= self.max_requests {
            return false;
        }

        bucket.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::SimpleRateLimiter;

    #[tokio::test]
    async fn denies_requests_after_limit_and_allows_after_window() {
        let limiter = SimpleRateLimiter::new(2, Duration::from_millis(30));

        assert!(limiter.allow("client-1").await);
        assert!(limiter.allow("client-1").await);
        assert!(!limiter.allow("client-1").await);

        tokio::time::sleep(Duration::from_millis(40)).await;

        assert!(limiter.allow("client-1").await);
    }
}
