// src/db/query_plan.rs
//
// Query plan visualization for QueryWise.
//
// The `:plan` command runs EXPLAIN (QUERY PLAN) on a SQL query and renders
// the output as a structured, color-coded tree in the TUI. This is a feature
// no other terminal database client has with AI integration — QueryWise can
// explain the plan in plain English too.

use std::fmt;

/// A single node in a query execution plan.
#[derive(Debug, Clone)]
pub struct PlanNode {
    /// The operation type (e.g., "Seq Scan", "Index Scan", "Hash Join")
    pub operation: String,
    /// The target table or index, if applicable
    pub target: Option<String>,
    /// Estimated cost (startup..total)
    pub cost: Option<(f64, f64)>,
    /// Estimated rows
    pub rows: Option<u64>,
    /// Estimated width in bytes
    pub width: Option<u64>,
    /// Actual time in ms (if EXPLAIN ANALYZE)
    pub actual_time: Option<(f64, f64)>,
    /// Actual rows returned (if EXPLAIN ANALYZE)
    pub actual_rows: Option<u64>,
    /// Filter condition
    pub filter: Option<String>,
    /// Join condition
    pub join_condition: Option<String>,
    /// Sort key
    pub sort_key: Option<String>,
    /// Child nodes
    pub children: Vec<PlanNode>,
    /// Raw text from EXPLAIN output
    pub raw: String,
    /// Depth in the tree (for indentation)
    pub depth: usize,
}

impl PlanNode {
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            target: None,
            cost: None,
            rows: None,
            width: None,
            actual_time: None,
            actual_rows: None,
            filter: None,
            join_condition: None,
            sort_key: None,
            children: Vec::new(),
            raw: String::new(),
            depth: 0,
        }
    }
}

/// Parsed query plan containing the root node and metadata.
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub root: Option<PlanNode>,
    pub raw_output: String,
    pub database_type: DatabaseType,
    pub planning_time: Option<f64>,
    pub execution_time: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::SQLite => write!(f, "SQLite"),
        }
    }
}

/// Generate the EXPLAIN SQL for a given database type.
pub fn explain_sql(query: &str, db_type: DatabaseType, analyze: bool) -> String {
    match db_type {
        DatabaseType::PostgreSQL => {
            if analyze {
                format!("EXPLAIN (ANALYZE, FORMAT TEXT) {}", query)
            } else {
                format!("EXPLAIN {}", query)
            }
        }
        DatabaseType::MySQL => {
            if analyze {
                format!("EXPLAIN ANALYZE {}", query)
            } else {
                format!("EXPLAIN {}", query)
            }
        }
        DatabaseType::SQLite => {
            format!("EXPLAIN QUERY PLAN {}", query)
        }
    }
}

/// Parse raw EXPLAIN output into a structured QueryPlan.
pub fn parse_plan(raw: &str, db_type: DatabaseType) -> QueryPlan {
    let mut plan = QueryPlan {
        root: None,
        raw_output: raw.to_string(),
        database_type: db_type,
        planning_time: None,
        execution_time: None,
    };

    match db_type {
        DatabaseType::PostgreSQL => parse_postgres_plan(&mut plan, raw),
        DatabaseType::MySQL => parse_mysql_plan(&mut plan, raw),
        DatabaseType::SQLite => parse_sqlite_plan(&mut plan, raw),
    }

    plan
}

/// Parse PostgreSQL EXPLAIN output.
fn parse_postgres_plan(plan: &mut QueryPlan, raw: &str) {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return;
    }

    let mut nodes: Vec<(usize, PlanNode)> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // Extract planning/execution time
        if trimmed.starts_with("Planning Time:") || trimmed.starts_with("Planning time:") {
            if let Some(time_str) = trimmed.split(':').nth(1) {
                plan.planning_time = time_str
                    .trim()
                    .trim_end_matches(" ms")
                    .parse()
                    .ok();
            }
            continue;
        }
        if trimmed.starts_with("Execution Time:") || trimmed.starts_with("Execution time:") {
            if let Some(time_str) = trimmed.split(':').nth(1) {
                plan.execution_time = time_str
                    .trim()
                    .trim_end_matches(" ms")
                    .parse()
                    .ok();
            }
            continue;
        }

        // Calculate indentation depth
        let indent = line.len() - line.trim_start_matches(|c: char| c == ' ' || c == '-' || c == '>').len();
        let depth = indent / 6; // PostgreSQL uses 6-char indent levels

        // Extract operation name
        let op_line = trimmed.trim_start_matches("-> ");
        if op_line.is_empty() {
            continue;
        }

        let mut node = PlanNode::new(extract_operation(op_line));
        node.raw = op_line.to_string();
        node.depth = depth;
        node.target = extract_target(op_line);
        node.cost = extract_cost(op_line);
        node.rows = extract_rows(op_line);
        node.width = extract_width(op_line);
        node.actual_time = extract_actual_time(op_line);
        node.actual_rows = extract_actual_rows(op_line);

        nodes.push((depth, node));
    }

    if !nodes.is_empty() {
        plan.root = Some(build_tree(nodes));
    }
}

/// Parse SQLite EXPLAIN QUERY PLAN output.
fn parse_sqlite_plan(plan: &mut QueryPlan, raw: &str) {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return;
    }

    let mut nodes: Vec<(usize, PlanNode)> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // SQLite format: |--SCAN table_name
        // or: `--SEARCH table_name USING INDEX idx
        let depth = line.matches("--").count();
        let op_text = trimmed
            .trim_start_matches(|c: char| c == '|' || c == '`' || c == '-' || c == ' ');

        if op_text.is_empty() {
            continue;
        }

        let mut node = PlanNode::new(op_text.to_string());
        node.raw = trimmed.to_string();
        node.depth = depth;

        // Extract table name from common patterns
        if op_text.starts_with("SCAN") {
            node.operation = "SCAN".to_string();
            node.target = op_text.strip_prefix("SCAN ").map(|s| {
                s.split_whitespace().next().unwrap_or(s).to_string()
            });
        } else if op_text.starts_with("SEARCH") {
            node.operation = "SEARCH".to_string();
            node.target = op_text.strip_prefix("SEARCH ").map(|s| {
                s.split_whitespace().next().unwrap_or(s).to_string()
            });
        }

        nodes.push((depth, node));
    }

    if !nodes.is_empty() {
        plan.root = Some(build_tree(nodes));
    }
}

/// Parse MySQL EXPLAIN output (simplified).
fn parse_mysql_plan(plan: &mut QueryPlan, raw: &str) {
    // MySQL EXPLAIN ANALYZE outputs a tree similar to PostgreSQL
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return;
    }

    let mut nodes: Vec<(usize, PlanNode)> = Vec::new();

    for line in &lines {
        let indent = line.len() - line.trim_start_matches(|c: char| c == ' ' || c == '-' || c == '>').len();
        let depth = indent / 4;
        let trimmed = line.trim().trim_start_matches("-> ");

        if trimmed.is_empty() {
            continue;
        }

        let mut node = PlanNode::new(trimmed.to_string());
        node.raw = trimmed.to_string();
        node.depth = depth;
        nodes.push((depth, node));
    }

    if !nodes.is_empty() {
        plan.root = Some(build_tree(nodes));
    }
}

/// Build a tree from a flat list of (depth, node) pairs.
fn build_tree(nodes: Vec<(usize, PlanNode)>) -> PlanNode {
    if nodes.is_empty() {
        return PlanNode::new("Empty Plan".to_string());
    }

    let (_, root) = nodes[0].clone();
    let mut stack: Vec<(usize, PlanNode)> = vec![(0, root)];

    for (depth, node) in nodes.into_iter().skip(1) {
        while stack.len() > 1 && stack.last().map(|(d, _)| *d >= depth).unwrap_or(false) {
            let (_, child) = stack.pop().unwrap();
            if let Some((_, parent)) = stack.last_mut() {
                parent.children.push(child);
            }
        }
        stack.push((depth, node));
    }

    // Collapse remaining stack
    while stack.len() > 1 {
        let (_, child) = stack.pop().unwrap();
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(child);
        }
    }

    stack.pop().map(|(_, n)| n).unwrap_or_else(|| PlanNode::new("Empty Plan".to_string()))
}

/// Extract the operation name from an EXPLAIN line.
fn extract_operation(line: &str) -> String {
    // Operation name is everything before the first parenthesis or "on"
    let ops = line
        .split(|c: char| c == '(' || c == '[')
        .next()
        .unwrap_or(line)
        .trim();

    // Remove "on table_name" suffix for cleaner operation name
    if let Some(idx) = ops.find(" on ") {
        ops[..idx].trim().to_string()
    } else {
        ops.to_string()
    }
}

/// Extract the target table/index from an EXPLAIN line.
fn extract_target(line: &str) -> Option<String> {
    if let Some(idx) = line.find(" on ") {
        let rest = &line[idx + 4..];
        let target = rest
            .split(|c: char| c == ' ' || c == '(')
            .next()
            .unwrap_or(rest)
            .trim();
        if !target.is_empty() {
            return Some(target.to_string());
        }
    }
    None
}

/// Extract cost estimates from "(cost=X..Y ...)".
fn extract_cost(line: &str) -> Option<(f64, f64)> {
    if let Some(start) = line.find("cost=") {
        let rest = &line[start + 5..];
        let costs: Vec<&str> = rest
            .split(|c: char| c == ' ' || c == ')')
            .next()
            .unwrap_or("")
            .split("..")
            .collect();
        if costs.len() == 2 {
            if let (Ok(a), Ok(b)) = (costs[0].parse(), costs[1].parse()) {
                return Some((a, b));
            }
        }
    }
    None
}

/// Extract estimated rows from "rows=N".
fn extract_rows(line: &str) -> Option<u64> {
    extract_numeric(line, "rows=")
}

/// Extract width from "width=N".
fn extract_width(line: &str) -> Option<u64> {
    extract_numeric(line, "width=")
}

/// Extract actual time from "actual time=X..Y".
fn extract_actual_time(line: &str) -> Option<(f64, f64)> {
    if let Some(start) = line.find("actual time=") {
        let rest = &line[start + 12..];
        let times: Vec<&str> = rest
            .split(|c: char| c == ' ' || c == ')')
            .next()
            .unwrap_or("")
            .split("..")
            .collect();
        if times.len() == 2 {
            if let (Ok(a), Ok(b)) = (times[0].parse(), times[1].parse()) {
                return Some((a, b));
            }
        }
    }
    None
}

/// Extract actual rows from "rows=N" in EXPLAIN ANALYZE output.
fn extract_actual_rows(line: &str) -> Option<u64> {
    // In EXPLAIN ANALYZE, actual rows appears after "rows=" in the actual section
    if let Some(start) = line.find("actual") {
        extract_numeric(&line[start..], "rows=")
    } else {
        None
    }
}

/// Generic numeric extraction helper.
fn extract_numeric(line: &str, prefix: &str) -> Option<u64> {
    if let Some(start) = line.find(prefix) {
        let rest = &line[start + prefix.len()..];
        let num_str = rest
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .unwrap_or("");
        num_str.parse().ok()
    } else {
        None
    }
}

/// Format a QueryPlan as a displayable string for the TUI.
pub fn format_plan(plan: &QueryPlan) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("── Query Plan ({}) ──", plan.database_type));
    lines.push(String::new());

    if let Some(ref root) = plan.root {
        format_node(&mut lines, root, 0, true);
    } else {
        lines.push("  No plan data available.".to_string());
    }

    lines.push(String::new());

    if let Some(pt) = plan.planning_time {
        lines.push(format!("  Planning time:  {:.3} ms", pt));
    }
    if let Some(et) = plan.execution_time {
        lines.push(format!("  Execution time: {:.3} ms", et));
    }

    lines
}

/// Recursively format a plan node with tree-drawing characters.
fn format_node(lines: &mut Vec<String>, node: &PlanNode, depth: usize, is_last: bool) {
    let prefix = if depth == 0 {
        "  ".to_string()
    } else {
        let mut p = String::new();
        for _ in 0..depth - 1 {
            p.push_str("  │ ");
        }
        if is_last {
            p.push_str("  └─");
        } else {
            p.push_str("  ├─");
        }
        p
    };

    let mut line = format!("{} {}", prefix, node.operation);

    if let Some(ref target) = node.target {
        line.push_str(&format!(" on {}", target));
    }
    if let Some((_, total)) = node.cost {
        line.push_str(&format!("  (cost: {:.1})", total));
    }
    if let Some(rows) = node.rows {
        line.push_str(&format!("  rows: {}", rows));
    }
    if let Some((_, actual)) = node.actual_time {
        line.push_str(&format!("  actual: {:.3}ms", actual));
    }

    lines.push(line);

    for (i, child) in node.children.iter().enumerate() {
        let last = i == node.children.len() - 1;
        format_node(lines, child, depth + 1, last);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_sql_postgres() {
        let sql = explain_sql("SELECT * FROM users", DatabaseType::PostgreSQL, false);
        assert!(sql.starts_with("EXPLAIN"));
        assert!(sql.contains("SELECT * FROM users"));
    }

    #[test]
    fn test_explain_sql_postgres_analyze() {
        let sql = explain_sql("SELECT * FROM users", DatabaseType::PostgreSQL, true);
        assert!(sql.contains("ANALYZE"));
    }

    #[test]
    fn test_explain_sql_sqlite() {
        let sql = explain_sql("SELECT * FROM users", DatabaseType::SQLite, false);
        assert!(sql.starts_with("EXPLAIN QUERY PLAN"));
    }

    #[test]
    fn test_parse_sqlite_plan() {
        let raw = r#"SCAN users
`--SEARCH orders USING INDEX idx_user_id"#;

        let plan = parse_plan(raw, DatabaseType::SQLite);
        assert!(plan.root.is_some());
    }

    #[test]
    fn test_extract_cost() {
        let line = "Seq Scan on users  (cost=0.00..35.50 rows=2550 width=36)";
        assert_eq!(extract_cost(line), Some((0.0, 35.50)));
    }

    #[test]
    fn test_extract_rows() {
        let line = "Seq Scan on users  (cost=0.00..35.50 rows=2550 width=36)";
        assert_eq!(extract_rows(line), Some(2550));
    }

    #[test]
    fn test_extract_target() {
        let line = "Seq Scan on users  (cost=0.00..35.50)";
        assert_eq!(extract_target(line), Some("users".to_string()));
    }

    #[test]
    fn test_format_plan_empty() {
        let plan = QueryPlan {
            root: None,
            raw_output: String::new(),
            database_type: DatabaseType::SQLite,
            planning_time: None,
            execution_time: None,
        };
        let lines = format_plan(&plan);
        assert!(lines.iter().any(|l| l.contains("No plan data")));
    }

    #[test]
    fn test_format_plan_with_node() {
        let mut root = PlanNode::new("Seq Scan".to_string());
        root.target = Some("users".to_string());
        root.cost = Some((0.0, 35.5));
        root.rows = Some(100);

        let plan = QueryPlan {
            root: Some(root),
            raw_output: String::new(),
            database_type: DatabaseType::PostgreSQL,
            planning_time: Some(0.123),
            execution_time: Some(4.567),
        };

        let lines = format_plan(&plan);
        assert!(lines.iter().any(|l| l.contains("Seq Scan")));
        assert!(lines.iter().any(|l| l.contains("users")));
        assert!(lines.iter().any(|l| l.contains("Planning time")));
    }
}
