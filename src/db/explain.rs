//! Query execution plan analysis for Postgres, MySQL, and SQLite.
//!
//! Generates the appropriate EXPLAIN syntax per database engine,
//! parses the output into a structured format, and formats it
//! for display in the TUI.

/// The full execution plan result for a query.
#[derive(Debug, Clone)]
pub struct ExplainResult {
    pub query: String,
    pub db_type: String,
    pub plan_text: Vec<String>,
    pub total_cost: Option<f64>,
    pub planning_time: Option<f64>,
    pub execution_time: Option<f64>,
}

impl ExplainResult {
    /// Format the plan for TUI display with separator lines and timing summary.
    pub fn formatted_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("EXPLAIN for: {}", self.query));
        lines.push("\u2500".repeat(60));

        for line in &self.plan_text {
            lines.push(line.clone());
        }

        lines.push("\u2500".repeat(60));

        if let Some(cost) = self.total_cost {
            lines.push(format!("Estimated cost: {:.2}", cost));
        }
        if let Some(pt) = self.planning_time {
            lines.push(format!("Planning time:  {:.3} ms", pt));
        }
        if let Some(et) = self.execution_time {
            lines.push(format!("Execution time: {:.3} ms", et));
        }

        lines
    }
}

/// Generate the appropriate EXPLAIN SQL for the given database type.
///
/// Handles dialect differences:
/// - PostgreSQL: `EXPLAIN (FORMAT TEXT)` or `EXPLAIN (ANALYZE, FORMAT TEXT)`
/// - MySQL: `EXPLAIN` or `EXPLAIN ANALYZE`
/// - SQLite: `EXPLAIN QUERY PLAN` (no ANALYZE equivalent)
pub fn explain_sql(query: &str, db_type: &str, analyze: bool) -> String {
    let trimmed = query.trim().trim_end_matches(';');

    match db_type {
        "postgresql" => {
            if analyze {
                format!("EXPLAIN (ANALYZE, FORMAT TEXT) {}", trimmed)
            } else {
                format!("EXPLAIN (FORMAT TEXT) {}", trimmed)
            }
        }
        "mysql" => {
            if analyze {
                format!("EXPLAIN ANALYZE {}", trimmed)
            } else {
                format!("EXPLAIN {}", trimmed)
            }
        }
        "sqlite" => {
            // SQLite uses EXPLAIN QUERY PLAN; no ANALYZE equivalent
            format!("EXPLAIN QUERY PLAN {}", trimmed)
        }
        _ => format!("EXPLAIN {}", trimmed),
    }
}

/// Parse raw EXPLAIN output rows into a structured ExplainResult.
///
/// Each database returns EXPLAIN data in a different format:
/// - PostgreSQL: single-column text output with indented tree nodes
/// - MySQL: tabular output (id, select_type, table, type, ...)
/// - SQLite: (id, parent, notused, detail) from EXPLAIN QUERY PLAN
pub fn parse_explain_output(
    rows: Vec<Vec<String>>,
    columns: Vec<String>,
    query: &str,
    db_type: &str,
) -> ExplainResult {
    let mut plan_text = Vec::new();
    let mut total_cost: Option<f64> = None;
    let mut planning_time: Option<f64> = None;
    let mut execution_time: Option<f64> = None;

    match db_type {
        "postgresql" => {
            for row in &rows {
                if let Some(line) = row.first() {
                    plan_text.push(line.clone());

                    // Extract cost from plan nodes:
                    // "Seq Scan on users  (cost=0.00..35.50 rows=10 width=540)"
                    if total_cost.is_none() {
                        if let Some(cost_str) = extract_between(line, "cost=", "..") {
                            total_cost = cost_str.parse().ok();
                        }
                    }

                    // Parse ANALYZE timing lines
                    let lower = line.trim().to_lowercase();
                    if lower.starts_with("planning time:") {
                        planning_time = extract_ms_value(line);
                    }
                    if lower.starts_with("execution time:") {
                        execution_time = extract_ms_value(line);
                    }
                }
            }
        }
        "mysql" => {
            // MySQL EXPLAIN returns tabular data; format as aligned columns
            if !columns.is_empty() {
                plan_text.push(columns.join(" | "));
                plan_text.push("\u2500".repeat(plan_text[0].len()));
            }
            for row in &rows {
                plan_text.push(row.join(" | "));
            }
        }
        "sqlite" => {
            // EXPLAIN QUERY PLAN returns: id, parent, notused, detail
            for row in &rows {
                let indent = if row.len() > 1 {
                    let parent: i64 = row[1].parse().unwrap_or(0);
                    "  ".repeat(if parent > 0 { 1 } else { 0 })
                } else {
                    String::new()
                };
                let detail = row.last().unwrap_or(&String::new()).clone();
                plan_text.push(format!("{}\u251c\u2500 {}", indent, detail));
            }
        }
        _ => {
            for row in &rows {
                plan_text.push(row.join(" | "));
            }
        }
    }

    ExplainResult {
        query: query.to_string(),
        db_type: db_type.to_string(),
        plan_text,
        total_cost,
        planning_time,
        execution_time,
    }
}

/// Extract a substring between two delimiters.
fn extract_between(s: &str, start: &str, end: &str) -> Option<String> {
    let start_idx = s.find(start)? + start.len();
    let end_idx = s[start_idx..].find(end)? + start_idx;
    Some(s[start_idx..end_idx].to_string())
}

/// Extract a millisecond value from a line like "Planning Time: 0.123 ms".
fn extract_ms_value(line: &str) -> Option<f64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "ms" && i > 0 {
            return parts[i - 1].parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_sql_postgres() {
        let sql = explain_sql("SELECT * FROM users", "postgresql", false);
        assert_eq!(sql, "EXPLAIN (FORMAT TEXT) SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_postgres_analyze() {
        let sql = explain_sql("SELECT * FROM users", "postgresql", true);
        assert_eq!(sql, "EXPLAIN (ANALYZE, FORMAT TEXT) SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_mysql() {
        let sql = explain_sql("SELECT * FROM users", "mysql", false);
        assert_eq!(sql, "EXPLAIN SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_mysql_analyze() {
        let sql = explain_sql("SELECT * FROM users", "mysql", true);
        assert_eq!(sql, "EXPLAIN ANALYZE SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_sqlite() {
        let sql = explain_sql("SELECT * FROM users", "sqlite", false);
        assert_eq!(sql, "EXPLAIN QUERY PLAN SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_sqlite_analyze_same_as_regular() {
        // SQLite has no ANALYZE variant; both should produce EXPLAIN QUERY PLAN
        let sql = explain_sql("SELECT * FROM users", "sqlite", true);
        assert_eq!(sql, "EXPLAIN QUERY PLAN SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_strips_trailing_semicolon() {
        let sql = explain_sql("SELECT * FROM users;", "postgresql", false);
        assert_eq!(sql, "EXPLAIN (FORMAT TEXT) SELECT * FROM users");
    }

    #[test]
    fn test_explain_sql_unknown_db_fallback() {
        let sql = explain_sql("SELECT 1", "unknown", false);
        assert_eq!(sql, "EXPLAIN SELECT 1");
    }

    #[test]
    fn test_parse_postgres_explain() {
        let rows = vec![
            vec!["Seq Scan on users  (cost=0.00..35.50 rows=2550 width=36)".to_string()],
            vec!["  Filter: (active = true)".to_string()],
            vec!["Planning Time: 0.083 ms".to_string()],
            vec!["Execution Time: 0.312 ms".to_string()],
        ];
        let columns = vec!["QUERY PLAN".to_string()];
        let result = parse_explain_output(
            rows, columns,
            "SELECT * FROM users WHERE active = true",
            "postgresql",
        );

        assert_eq!(result.db_type, "postgresql");
        assert_eq!(result.plan_text.len(), 4);
        assert_eq!(result.total_cost, Some(0.0));
        assert_eq!(result.planning_time, Some(0.083));
        assert_eq!(result.execution_time, Some(0.312));
    }

    #[test]
    fn test_parse_sqlite_explain() {
        let rows = vec![
            vec!["2".to_string(), "0".to_string(), "0".to_string(), "SCAN users".to_string()],
            vec!["3".to_string(), "2".to_string(), "0".to_string(), "SEARCH orders USING INDEX idx_user_id (user_id=?)".to_string()],
        ];
        let columns = vec!["id".to_string(), "parent".to_string(), "notused".to_string(), "detail".to_string()];
        let result = parse_explain_output(
            rows, columns,
            "SELECT * FROM users JOIN orders ON users.id = orders.user_id",
            "sqlite",
        );

        assert_eq!(result.plan_text.len(), 2);
        assert!(result.plan_text[0].contains("SCAN users"));
        assert!(result.plan_text[1].contains("SEARCH orders"));
    }

    #[test]
    fn test_parse_mysql_explain() {
        let columns = vec!["id".to_string(), "select_type".to_string(), "table".to_string(), "type".to_string()];
        let rows = vec![
            vec!["1".to_string(), "SIMPLE".to_string(), "users".to_string(), "ALL".to_string()],
        ];
        let result = parse_explain_output(rows, columns, "SELECT * FROM users", "mysql");

        // MySQL: header + separator + data rows
        assert_eq!(result.plan_text.len(), 3);
        assert!(result.plan_text[0].contains("id"));
        assert!(result.plan_text[2].contains("SIMPLE"));
    }

    #[test]
    fn test_formatted_lines_structure() {
        let result = ExplainResult {
            query: "SELECT 1".to_string(),
            db_type: "postgresql".to_string(),
            plan_text: vec!["Result  (cost=0.00..0.01 rows=1 width=4)".to_string()],
            total_cost: Some(0.0),
            planning_time: Some(0.05),
            execution_time: Some(0.1),
        };
        let lines = result.formatted_lines();
        assert!(lines[0].contains("EXPLAIN for:"));
        assert!(lines.iter().any(|l| l.contains("Planning time")));
        assert!(lines.iter().any(|l| l.contains("Execution time")));
        assert!(lines.iter().any(|l| l.contains("Estimated cost")));
    }

    #[test]
    fn test_extract_between() {
        assert_eq!(
            extract_between("cost=0.00..35.50", "cost=", ".."),
            Some("0.00".to_string())
        );
        assert_eq!(extract_between("no match here", "cost=", ".."), None);
    }

    #[test]
    fn test_extract_ms_value() {
        assert_eq!(extract_ms_value("Planning Time: 0.083 ms"), Some(0.083));
        assert_eq!(extract_ms_value("Execution Time: 1.234 ms"), Some(1.234));
        assert_eq!(extract_ms_value("no time here"), None);
    }

    #[test]
    fn test_parse_empty_output() {
        let result = parse_explain_output(vec![], vec![], "SELECT 1", "postgresql");
        assert!(result.plan_text.is_empty());
        assert_eq!(result.total_cost, None);
    }
}
