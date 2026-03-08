/// TTL-aware LRU query cache with automatic cleanup.
///
/// Expired entries are purged on load AND before every write.
/// Cache file never grows without bound.
/// Atomic writes (temp -> rename) prevent corruption.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_ENTRIES: usize = 200;
const DEFAULT_TTL_SECONDS: i64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: i64,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age > Duration::seconds(self.ttl_seconds)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    entries: HashMap<String, CacheEntry>,
}

pub struct QueryCache {
    entries: HashMap<String, CacheEntry>,
    path: PathBuf,
    ttl_seconds: i64,
}

impl QueryCache {
    /// Load from disk and immediately evict all expired entries.
    pub fn load() -> Self {
        let path = Self::cache_path();
        let ttl_seconds = DEFAULT_TTL_SECONDS;
        let mut entries: HashMap<String, CacheEntry> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<CacheFile>(&s).ok())
            .map(|f| f.entries)
            .unwrap_or_default();
        let before = entries.len();
        entries.retain(|_, v| !v.is_expired());
        let mut cache = Self { entries, path, ttl_seconds };
        // persist cleaned file so next session starts fresh
        if cache.entries.len() != before { let _ = cache.save(); }
        cache
    }

    pub fn get(&mut self, sql: &str) -> Option<&CacheEntry> {
        let key = Self::key(sql);
        if let Some(entry) = self.entries.get(&key) {
            if entry.is_expired() {
                self.entries.remove(&key);
                let _ = self.save();
                return None;
            }
        }
        self.entries.get(&key)
    }

    pub fn insert(
        &mut self, sql: &str, columns: Vec<String>, rows: Vec<Vec<String>>,
        row_count: usize, execution_time_ms: u64,
    ) -> Result<()> {
        // purge expired before adding
        self.entries.retain(|_, v| !v.is_expired());
        // evict oldest if at cap
        if self.entries.len() >= MAX_ENTRIES {
            if let Some(k) = self.entries.iter().min_by_key(|(_, v)| v.cached_at).map(|(k, _)| k.clone()) {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(Self::key(sql), CacheEntry {
            columns, rows, row_count, execution_time_ms,
            cached_at: Utc::now(), ttl_seconds: self.ttl_seconds,
        });
        self.save()
    }

    pub fn invalidate(&mut self, sql: &str) -> Result<()> {
        self.entries.remove(&Self::key(sql));
        self.save()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save()
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    fn key(sql: &str) -> String {
        sql.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase()
    }

    fn cache_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("querywise")
            .join("query_cache.json")
    }

    fn save(&self) -> Result<()> {
        let dir = self.path.parent().unwrap_or(&self.path);
        std::fs::create_dir_all(dir)?;
        let tmp = self.path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(&CacheFile { entries: self.entries.clone() })?;
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    fn mem(ttl: i64) -> QueryCache {
        QueryCache { entries: HashMap::new(), path: PathBuf::from("/tmp/qw_test.json"), ttl_seconds: ttl }
    }

    #[test] fn test_insert_and_get() {
        let mut c = mem(60);
        c.insert("SELECT 1", vec!["c".into()], vec![vec!["1".into()]], 1, 5).unwrap();
        assert!(c.get("SELECT 1").is_some());
    }

    #[test] fn test_expired_removed_on_get() {
        let mut c = mem(0);
        c.insert("SELECT 1", vec![], vec![], 0, 0).unwrap();
        sleep(StdDuration::from_millis(10));
        assert!(c.get("SELECT 1").is_none());
        assert!(c.is_empty());
    }

    #[test] fn test_key_normalises_whitespace() {
        let mut c = mem(60);
        c.insert("SELECT  *  FROM t", vec![], vec![], 0, 0).unwrap();
        assert!(c.get("select * from t").is_some());
    }

    #[test] fn test_clear() {
        let mut c = mem(60);
        c.insert("SELECT 1", vec![], vec![], 0, 0).unwrap();
        c.clear().unwrap();
        assert!(c.is_empty());
    }

    #[test] fn test_invalidate() {
        let mut c = mem(60);
        c.insert("SELECT 1", vec![], vec![], 0, 0).unwrap();
        c.insert("SELECT 2", vec![], vec![], 0, 0).unwrap();
        c.invalidate("SELECT 1").unwrap();
        assert!(c.get("SELECT 1").is_none());
        assert!(c.get("SELECT 2").is_some());
    }
}