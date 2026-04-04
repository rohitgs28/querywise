// src/db/bookmarks.rs
//
// Query bookmarks for QueryWise.
//
// Save frequently used queries with labels for quick recall.
// Persisted to ~/.config/querywise/bookmarks.json.
// Addresses GitHub issue #6.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_BOOKMARKS: usize = 100;

/// A single bookmarked query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// User-defined label for the bookmark
    pub label: String,
    /// The SQL query or natural language question
    pub query: String,
    /// Optional database/connection identifier
    pub database: Option<String>,
    /// When the bookmark was created
    pub created_at: DateTime<Utc>,
    /// How many times the bookmark has been used
    pub use_count: u64,
    /// When it was last used
    pub last_used: Option<DateTime<Utc>>,
}

impl Bookmark {
    pub fn new(label: String, query: String, database: Option<String>) -> Self {
        Self {
            label,
            query,
            database,
            created_at: Utc::now(),
            use_count: 0,
            last_used: None,
        }
    }

    /// Record a use of this bookmark.
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = Some(Utc::now());
    }
}

/// Manager for query bookmarks with persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl BookmarkManager {
    /// Create a new BookmarkManager that persists to the default config directory.
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("querywise");

        std::fs::create_dir_all(&config_dir)
            .context("Could not create config directory")?;

        let file_path = config_dir.join("bookmarks.json");
        let mut manager = Self {
            bookmarks: Vec::new(),
            file_path: Some(file_path.clone()),
        };

        // Load existing bookmarks
        if file_path.exists() {
            let data = std::fs::read_to_string(&file_path)
                .context("Could not read bookmarks file")?;
            if let Ok(saved) = serde_json::from_str::<Vec<Bookmark>>(&data) {
                manager.bookmarks = saved;
            }
        }

        Ok(manager)
    }

    /// Create an in-memory manager (for testing).
    pub fn in_memory() -> Self {
        Self {
            bookmarks: Vec::new(),
            file_path: None,
        }
    }

    /// Add a new bookmark. Returns an error if the label already exists.
    pub fn add(&mut self, label: String, query: String, database: Option<String>) -> Result<()> {
        if self.bookmarks.len() >= MAX_BOOKMARKS {
            anyhow::bail!("Maximum of {} bookmarks reached. Remove some first.", MAX_BOOKMARKS);
        }

        if self.bookmarks.iter().any(|b| b.label == label) {
            anyhow::bail!("Bookmark with label '{}' already exists. Use a different label or remove it first.", label);
        }

        let bookmark = Bookmark::new(label, query, database);
        self.bookmarks.push(bookmark);
        self.save()?;
        Ok(())
    }

    /// Remove a bookmark by label.
    pub fn remove(&mut self, label: &str) -> Result<Bookmark> {
        let idx = self.bookmarks.iter().position(|b| b.label == label)
            .context(format!("No bookmark with label '{}'", label))?;

        let removed = self.bookmarks.remove(idx);
        self.save()?;
        Ok(removed)
    }

    /// Get a bookmark by label and record its use.
    pub fn get_and_use(&mut self, label: &str) -> Option<String> {
        if let Some(bookmark) = self.bookmarks.iter_mut().find(|b| b.label == label) {
            bookmark.record_use();
            let query = bookmark.query.clone();
            let _ = self.save(); // best-effort save
            Some(query)
        } else {
            None
        }
    }

    /// Get a bookmark by index (for TUI list selection) and record its use.
    pub fn get_by_index(&mut self, index: usize) -> Option<String> {
        if let Some(bookmark) = self.bookmarks.get_mut(index) {
            bookmark.record_use();
            let query = bookmark.query.clone();
            let _ = self.save();
            Some(query)
        } else {
            None
        }
    }

    /// List all bookmarks.
    pub fn list(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    /// Search bookmarks by label or query content.
    pub fn search(&self, query: &str) -> Vec<&Bookmark> {
        let query_lower = query.to_lowercase();
        self.bookmarks.iter()
            .filter(|b| {
                b.label.to_lowercase().contains(&query_lower)
                    || b.query.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get bookmarks sorted by most frequently used.
    pub fn most_used(&self, limit: usize) -> Vec<&Bookmark> {
        let mut sorted: Vec<&Bookmark> = self.bookmarks.iter().collect();
        sorted.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        sorted.truncate(limit);
        sorted
    }

    /// Get the number of bookmarks.
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Format bookmarks for TUI display.
    pub fn format_list(&self) -> Vec<String> {
        if self.bookmarks.is_empty() {
            return vec!["  No bookmarks saved. Use Ctrl+B to bookmark a query.".to_string()];
        }

        let mut lines = Vec::new();
        for (i, b) in self.bookmarks.iter().enumerate() {
            let truncated_query = if b.query.len() > 60 {
                format!("{}...", &b.query[..57])
            } else {
                b.query.clone()
            };
            lines.push(format!(
                "  [{}] {} — {} (used {} times)",
                i + 1,
                b.label,
                truncated_query,
                b.use_count,
            ));
        }
        lines
    }

    /// Save bookmarks to disk.
    fn save(&self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            let data = serde_json::to_string_pretty(&self.bookmarks)
                .context("Could not serialize bookmarks")?;
            std::fs::write(path, data)
                .context("Could not write bookmarks file")?;
        }
        Ok(())
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::in_memory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_list() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("active-users".into(), "SELECT * FROM users WHERE active = true".into(), None).unwrap();
        assert_eq!(mgr.count(), 1);
        assert_eq!(mgr.list()[0].label, "active-users");
    }

    #[test]
    fn test_duplicate_label_rejected() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("test".into(), "SELECT 1".into(), None).unwrap();
        assert!(mgr.add("test".into(), "SELECT 2".into(), None).is_err());
    }

    #[test]
    fn test_remove() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("to-remove".into(), "SELECT 1".into(), None).unwrap();
        let removed = mgr.remove("to-remove").unwrap();
        assert_eq!(removed.label, "to-remove");
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut mgr = BookmarkManager::in_memory();
        assert!(mgr.remove("nope").is_err());
    }

    #[test]
    fn test_get_and_use() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("test".into(), "SELECT 1".into(), None).unwrap();

        let query = mgr.get_and_use("test");
        assert_eq!(query, Some("SELECT 1".to_string()));
        assert_eq!(mgr.list()[0].use_count, 1);

        mgr.get_and_use("test");
        assert_eq!(mgr.list()[0].use_count, 2);
    }

    #[test]
    fn test_search() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("users-active".into(), "SELECT * FROM users WHERE active = true".into(), None).unwrap();
        mgr.add("orders-total".into(), "SELECT SUM(total) FROM orders".into(), None).unwrap();

        let results = mgr.search("users");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "users-active");

        let results = mgr.search("SELECT");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_most_used() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("a".into(), "SELECT 1".into(), None).unwrap();
        mgr.add("b".into(), "SELECT 2".into(), None).unwrap();

        mgr.get_and_use("a");
        mgr.get_and_use("b");
        mgr.get_and_use("b");
        mgr.get_and_use("b");

        let top = mgr.most_used(1);
        assert_eq!(top[0].label, "b");
        assert_eq!(top[0].use_count, 3);
    }

    #[test]
    fn test_max_bookmarks() {
        let mut mgr = BookmarkManager::in_memory();
        for i in 0..MAX_BOOKMARKS {
            mgr.add(format!("bm-{}", i), format!("SELECT {}", i), None).unwrap();
        }
        assert!(mgr.add("one-too-many".into(), "SELECT 999".into(), None).is_err());
    }

    #[test]
    fn test_format_list_empty() {
        let mgr = BookmarkManager::in_memory();
        let lines = mgr.format_list();
        assert!(lines[0].contains("No bookmarks"));
    }

    #[test]
    fn test_get_by_index() {
        let mut mgr = BookmarkManager::in_memory();
        mgr.add("first".into(), "SELECT 1".into(), None).unwrap();
        mgr.add("second".into(), "SELECT 2".into(), None).unwrap();

        assert_eq!(mgr.get_by_index(0), Some("SELECT 1".to_string()));
        assert_eq!(mgr.get_by_index(1), Some("SELECT 2".to_string()));
        assert_eq!(mgr.get_by_index(99), None);
    }
}
