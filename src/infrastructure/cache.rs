use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use redis::{AsyncCommands, aio::ConnectionManager};
use secrecy::ExposeSecret;
use tokio::sync::RwLock;

use crate::config::RedisSettings;

#[async_trait]
pub trait ResponseCache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str, ttl: Duration);
    async fn namespace_version(&self, namespace: &str) -> u64;
    async fn bump_namespace(&self, namespace: &str);
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
struct CachedEntry {
    value: String,
    expires_at: Instant,
}

#[derive(Debug)]
pub struct InMemoryResponseCache {
    entries: Arc<RwLock<HashMap<String, CachedEntry>>>,
    namespaces: Arc<RwLock<HashMap<String, u64>>>,
}

impl InMemoryResponseCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            namespaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ResponseCache for InMemoryResponseCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut entries = self.entries.write().await;
        let entry = entries.get(key)?.clone();

        if Instant::now() >= entry.expires_at {
            entries.remove(key);
            return None;
        }

        Some(entry.value)
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) {
        let expires_at = Instant::now() + ttl.max(Duration::from_millis(1));
        let entry = CachedEntry {
            value: value.to_string(),
            expires_at,
        };

        self.entries.write().await.insert(key.to_string(), entry);
    }

    async fn namespace_version(&self, namespace: &str) -> u64 {
        self.namespaces
            .read()
            .await
            .get(namespace)
            .copied()
            .unwrap_or(0)
    }

    async fn bump_namespace(&self, namespace: &str) {
        let mut namespaces = self.namespaces.write().await;
        let version = namespaces.entry(namespace.to_string()).or_default();
        *version += 1;
    }

    fn backend_name(&self) -> &'static str {
        "in_memory"
    }
}

#[derive(Clone)]
pub struct RedisResponseCache {
    manager: ConnectionManager,
    prefix: String,
}

impl RedisResponseCache {
    pub async fn connect(settings: &RedisSettings) -> anyhow::Result<Self> {
        let client = redis::Client::open(settings.url.expose_secret())?;
        let manager = client.get_connection_manager().await?;

        Ok(Self {
            manager,
            prefix: settings.cache_prefix.clone(),
        })
    }

    fn value_key(&self, key: &str) -> String {
        format!("{}:value:{}", self.prefix, key)
    }

    fn namespace_key(&self, namespace: &str) -> String {
        format!("{}:namespace:{}", self.prefix, namespace)
    }
}

#[async_trait]
impl ResponseCache for RedisResponseCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut connection = self.manager.clone();
        let redis_key = self.value_key(key);
        connection.get(redis_key).await.ok()
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) {
        let mut connection = self.manager.clone();
        let redis_key = self.value_key(key);
        let _: Result<(), _> = connection
            .set_ex(redis_key, value, ttl.as_secs().max(1))
            .await;
    }

    async fn namespace_version(&self, namespace: &str) -> u64 {
        let mut connection = self.manager.clone();
        let redis_key = self.namespace_key(namespace);
        connection.get(redis_key).await.unwrap_or(0)
    }

    async fn bump_namespace(&self, namespace: &str) {
        let mut connection = self.manager.clone();
        let redis_key = self.namespace_key(namespace);
        let _: Result<u64, _> = connection.incr(redis_key, 1).await;
    }

    fn backend_name(&self) -> &'static str {
        "redis"
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{InMemoryResponseCache, ResponseCache};

    #[tokio::test]
    async fn returns_cached_value_before_expiry() {
        let cache = InMemoryResponseCache::new();
        cache.set("key-1", "{\"ok\":true}", Duration::from_millis(50)).await;

        assert_eq!(cache.get("key-1").await, Some("{\"ok\":true}".to_string()));
    }

    #[tokio::test]
    async fn expires_cached_value_after_ttl() {
        let cache = InMemoryResponseCache::new();
        cache.set("key-1", "value", Duration::from_millis(20)).await;

        tokio::time::sleep(Duration::from_millis(30)).await;

        assert_eq!(cache.get("key-1").await, None);
    }

    #[tokio::test]
    async fn bumps_namespace_version() {
        let cache = InMemoryResponseCache::new();

        assert_eq!(cache.namespace_version("diplomas").await, 0);
        cache.bump_namespace("diplomas").await;
        assert_eq!(cache.namespace_version("diplomas").await, 1);
    }
}
