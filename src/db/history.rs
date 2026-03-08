use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY_ENTRIES: usize = 1000;
const HISTORY_FILE: &str = "history.json";

/// A single entry in the query history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The original user input (natural language or raw SQL)
    pub input: String,
    /// The final SQL that was executed (may differ from input after AI translation)
    pub sql: String,
    /// Whether the query succeeded
    pub success: bool,
    /// Number of rows returned
    pub row_count: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Timestamp of execution
    pub timestamp: DateTime<Local>,
    /// Database type this was run against
    pub db_type: String,
}

/// Persistent query history backed by a JSON file in the user's config directory.
///
/// Supports:
/// - Appending entries
/// - Fuzzy search across inputs and SQL
/// - Reverse chronological navigation (like shell Ctrl+R)
/// - Deduplication of consecutive identical inputs
pub struct QueryHistory {
    entries: Vec<HistoryEntry>,
    path: PathBuf,
    /// Current position in reverse-navigation (-1 = not browsing)
    nav_index: Option<usize>,
}

impl QueryHistory {
    /// Load history from disk, or create a new empty history if the file doesn't exist.
    pub fn load() -> Self {
        let path = Self::history_path();
        let entries = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            entries,
            path,
            nav_index: None,
        }
    }

    /// Append a new entry and persist to disk.
    /// Skips if the input is identical to the most recent entry (deduplication).
    pub fn push(&mut self, entry: HistoryEntry) -> Result<()> {
        // Deduplicate consecutive identical inputs
        if let Some(last) = self.entries.last() {
            if last.input == entry.input {
                return Ok(());
            }
        }

        self.entries.push(entry);

        // Trim to max size (keep most recent)
        if self.entries.len() > MAX_HISTORY_ENTRIES {
            let drain_count = self.entries.len() - MAX_HISTORY_ENTRIES;
            self.entries.drain(0..drain_count);
        }

        self.save()?;
        self.nav_index = None; // Reset navigation on new entry
        Ok(())
    }

    /// Fuzzy-search history by input or SQL content.
    /// Returns entries in reverse chronological order (most recent first).
    /// Scoring: exact substring match > word match > character match.
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        if query.is_empty() {
            return self.entries.iter().rev().take(50).collect();
        }

        let q = query.to_lowercase();
        let mut scored: Vec<(usize, &HistoryEntry)> = self
            .entries
            .iter()
            .filter_map(|e| {
                let input_lower = e.input.to_lowercase();
                let sql_lower = e.sql.to_lowercase();
                let score = Self::fuzzy_score(&input_lower, &q)
                    .max(Self::fuzzy_score(&sql_lower, &q));
                if score > 0 {
                    Some((score, e))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score desc, then by recency (index desc)
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, e)| e).take(50).collect()
    }

    /// Navigate backward through history (older entries).
    /// Returns the input string at the new position, or None if at the start.
    pub fn navigate_back(&mut self, current_input: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.nav_index {
            None => {
                // Start navigation from most recent
                self.entries.len().saturating_sub(1)
            }
            Some(0) => return None, // Already at oldest
            Some(i) => i.saturating_sub(1),
        };

        // Skip entries identical to current input when starting
        if self.nav_index.is_none() && self.entries[new_index].input == current_input {
            if new_index == 0 {
                return None;
            }
            self.nav_index = Some(new_index.saturating_sub(1));
        } else {
            self.nav_index = Some(new_index);
        }

        self.nav_index.map(|i| self.entries[i].input.as_str())
    }

    /// Navigate forward through history (newer entries).
    /// Returns None when back at the current (unsaved) input.
    pub fn navigate_forward(&mut self) -> Option<&str> {
        match self.nav_index {
            None => None,
            Some(i) if i + 1 >= self.entries.len() => {
                self.nav_index = None;
                None // Signal: restore original input
            }
            Some(i) => {
                self.nav_index = Some(i + 1);
                Some(self.entries[i + 1].input.as_str())
            }
        }
    }

    /// Reset navigation state (called when user types new input).
    pub fn reset_navigation(&mut self) {
        self.nav_index = None;
    }

    /// Returns true if we're currently in history navigation mode.
    pub fn is_navigating(&self) -> bool {
        self.nav_index.is_some()
    }

    /// Total number of history entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return the most recent N entries (for display in UI).
    pub fn recent(&self, n: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    fn history_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("querywise")
            .join(HISTORY_FILE)
    }

    /// Simple fuzzy scoring:
    /// - 100 points for exact substring match
    /// - 10 points per matching word
    /// - 1 point per matching character (in order)
    fn fuzzy_score(haystack: &str, needle: &str) -> usize {
        if haystack.contains(needle) {
            return 100 + (100 - needle.len().min(100)); // Prefer shorter matches
        }

        let mut score = 0usize;

        // Word-level matching
        for word in needle.split_whitespace() {
            if haystack.contains(word) {
                score += 10;
            }
        }

        // Character-level sequential matching
        let mut hay_chars = haystack.chars();
        for c in needle.chars() {
            if hay_chars.any(|h| h == c) {
                score += 1;
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn make_entry(input: &str, sql: &str, success: bool) -> HistoryEntry {
        HistoryEntry {
            input: input.to_string(),
            sql: sql.to_string(),
            success,
            row_count: 0,
            execution_time_ms: 10,
            timestamp: Local::now(),
            db_type: "sqlite".to_string(),
        }
    }

    #[test]
    fn test_fuzzy_score_exact_match() {
        let score = QueryHistory::fuzzy_score("select * from users", "users");
        assert!(score >= 100);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        let score = QueryHistory::fuzzy_score("select * from orders", "xyz123");
        assert_eq!(score, 0);
    }

    #[test]
    fn test_deduplication() {
        let mut h = QueryHistory {
            entries: Vec::new(),
            path: PathBuf::from("/tmp/test_history.json"),
            nav_index: None,
        };
        h.push(make_entry("show users", "SELECT * FROM users", true)).ok();
        h.push(make_entry("show users", "SELECT * FROM users", true)).ok();
        assert_eq!(h.len(), 1, "Consecutive duplicates should be deduplicated");
    }

    #[test]
    fn test_navigation_back_and_forward() {
        let mut h = QueryHistory {
            entries: vec![
                make_entry("query 1", "SELECT 1", true),
                make_entry("query 2", "SELECT 2", true),
                make_entry("query 3", "SELECT 3", true),
            ],
            path: PathBuf::from("/tmp/test_nav.json"),
            nav_index: None,
        };

        assert_eq!(h.navigate_back(""), Some("query 3"));
        assert_eq!(h.navigate_back(""), Some("query 2"));
        assert_eq!(h.navigate_forward(), Some("query 3"));
        assert_eq!(h.navigate_forward(), None); // back to current
    }

    #[test]
    fn test_search_returns_relevant_results() {
        let mut h = QueryHistory {
            entries: vec![
                make_entry("show all users", "SELECT * FROM users", true),
                make_entry("count orders today", "SELECT COUNT(*) FROM orders WHERE date = TODAY()", true),
                make_entry("find active users", "SELECT * FROM users WHERE active = 1", true),
            ],
            path: PathBuf::from("/tmp/test_search.json"),
            nav_index: None,
        };

        let results = h.search("users");
        assert!(!results.is_empty());
        assert!(results.iter().all(|e| e.input.contains("users") || e.sql.contains("users")));
    }

    #[test]
    fn test_max_history_size() {
        let mut h = QueryHistory {
            entries: Vec::new(),
            path: PathBuf::from("/tmp/test_max.json"),
            nav_index: None,
        };
        for i in 0..=MAX_HISTORY_ENTRIES + 10 {
            let _ = h.push(make_entry(&format!("query {}", i), "SELECT 1", true));
        }
        assert!(h.len() <= MAX_HISTORY_ENTRIES);
    }
}
