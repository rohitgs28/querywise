//! Connection string manager for saved database connections.
//!
//! Persists named connection strings to a JSON file in the user's
//! config directory so they can quickly reconnect without retyping URLs.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A saved database connection with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConnection {
    /// Display name (e.g., "prod-postgres", "local-sqlite")
    pub name: String,
    /// Connection URL (e.g., "postgres://user:pass@host/db")
    pub url: String,
    /// Database type inferred from the URL scheme
    pub db_type: String,
    /// When this connection was first saved
    pub created_at: DateTime<Utc>,
    /// When this connection was last used
    pub last_used: Option<DateTime<Utc>>,
    /// Number of times this connection has been used
    pub use_count: u64,
}

/// Manages persistent storage of named database connections.
pub struct ConnectionManager {
    connections: BTreeMap<String, SavedConnection>,
    path: PathBuf,
}

impl ConnectionManager {
    /// Load saved connections from disk, or create an empty manager.
    pub fn load() -> Result<Self> {
        let path = Self::storage_path()?;
        let connections = if path.exists() {
            let data = std::fs::read_to_string(&path)
                .context("reading connections file")?;
            serde_json::from_str(&data)
                .context("parsing connections file")?
        } else {
            BTreeMap::new()
        };
        Ok(Self { connections, path })
    }

    /// Save a new connection or update an existing one.
    pub fn save(&mut self, name: &str, url: &str) -> Result<()> {
        let db_type = Self::infer_db_type(url);
        let entry = self.connections.entry(name.to_string())
            .or_insert_with(|| SavedConnection {
                name: name.to_string(),
                url: url.to_string(),
                db_type: db_type.clone(),
                created_at: Utc::now(),
                last_used: None,
                use_count: 0,
            });
        entry.url = url.to_string();
        entry.db_type = db_type;
        self.persist()
    }

    /// Record that a connection was used (updates last_used and use_count).
    pub fn record_use(&mut self, name: &str) -> Result<()> {
        if let Some(conn) = self.connections.get_mut(name) {
            conn.last_used = Some(Utc::now());
            conn.use_count += 1;
            self.persist()?;
        }
        Ok(())
    }

    /// Remove a saved connection by name.
    pub fn remove(&mut self, name: &str) -> Result<bool> {
        let existed = self.connections.remove(name).is_some();
        if existed {
            self.persist()?;
        }
        Ok(existed)
    }

    /// Get a connection by name.
    pub fn get(&self, name: &str) -> Option<&SavedConnection> {
        self.connections.get(name)
    }

    /// List all saved connections, sorted by name.
    pub fn list(&self) -> Vec<&SavedConnection> {
        self.connections.values().collect()
    }

    /// List connections sorted by most recently used.
    pub fn list_recent(&self) -> Vec<&SavedConnection> {
        let mut conns: Vec<_> = self.connections.values().collect();
        conns.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        conns
    }

    /// Search connections by name prefix (for autocomplete).
    pub fn search(&self, prefix: &str) -> Vec<&SavedConnection> {
        let lower = prefix.to_lowercase();
        self.connections.values()
            .filter(|c| c.name.to_lowercase().starts_with(&lower))
            .collect()
    }

    /// Infer the database type from a connection URL.
    fn infer_db_type(url: &str) -> String {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            "PostgreSQL".to_string()
        } else if url.starts_with("mysql://") {
            "MySQL".to_string()
        } else if url.starts_with("sqlite:") {
            "SQLite".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Path to the connections storage file.
    fn storage_path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("could not find config directory"))?
            .join("querywise");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("connections.json"))
    }

    /// Write connections to disk atomically (write to temp, then rename).
    fn persist(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.connections)
            .context("serializing connections")?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, &data)
            .context("writing temp connections file")?;
        std::fs::rename(&tmp, &self.path)
            .context("renaming connections file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_manager() -> ConnectionManager {
        let dir = env::temp_dir().join(format!("qw-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        ConnectionManager {
            connections: BTreeMap::new(),
            path: dir.join("connections.json"),
        }
    }

    #[test]
    fn test_save_and_get() {
        let mut mgr = test_manager();
        mgr.save("local", "postgres://localhost/mydb").unwrap();

        let conn = mgr.get("local").unwrap();
        assert_eq!(conn.name, "local");
        assert_eq!(conn.db_type, "PostgreSQL");
    }

    #[test]
    fn test_infer_db_types() {
        assert_eq!(ConnectionManager::infer_db_type("postgres://host/db"), "PostgreSQL");
        assert_eq!(ConnectionManager::infer_db_type("postgresql://host/db"), "PostgreSQL");
        assert_eq!(ConnectionManager::infer_db_type("mysql://host/db"), "MySQL");
        assert_eq!(ConnectionManager::infer_db_type("sqlite:./data.db"), "SQLite");
        assert_eq!(ConnectionManager::infer_db_type("unknown://host"), "Unknown");
    }

    #[test]
    fn test_record_use() {
        let mut mgr = test_manager();
        mgr.save("dev", "sqlite::memory:").unwrap();
        mgr.record_use("dev").unwrap();
        mgr.record_use("dev").unwrap();

        let conn = mgr.get("dev").unwrap();
        assert_eq!(conn.use_count, 2);
        assert!(conn.last_used.is_some());
    }

    #[test]
    fn test_remove() {
        let mut mgr = test_manager();
        mgr.save("tmp", "sqlite::memory:").unwrap();
        assert!(mgr.remove("tmp").unwrap());
        assert!(mgr.get("tmp").is_none());
        assert!(!mgr.remove("nonexistent").unwrap());
    }

    #[test]
    fn test_list_and_search() {
        let mut mgr = test_manager();
        mgr.save("prod-pg", "postgres://prod/db").unwrap();
        mgr.save("dev-pg", "postgres://dev/db").unwrap();
        mgr.save("local-sqlite", "sqlite:./dev.db").unwrap();

        assert_eq!(mgr.list().len(), 3);
        assert_eq!(mgr.search("prod").len(), 1);
        assert_eq!(mgr.search("dev").len(), 1);
        assert_eq!(mgr.search("").len(), 3);
    }

    #[test]
    fn test_persistence() {
        let dir = env::temp_dir().join(format!("qw-persist-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("connections.json");

        // Save
        {
            let mut mgr = ConnectionManager { connections: BTreeMap::new(), path: path.clone() };
            mgr.save("test", "postgres://host/db").unwrap();
        }

        // Reload
        let data = std::fs::read_to_string(&path).unwrap();
        let conns: BTreeMap<String, SavedConnection> = serde_json::from_str(&data).unwrap();
        assert!(conns.contains_key("test"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
