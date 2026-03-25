//! Integration tests for querywise using in-memory SQLite.
//!
//! These tests exercise the core database layer without requiring
//! external database servers, making them fast and CI-friendly.

use querywise::db::{Database, QueryResult};
use querywise::db::{SchemaInfo, TableInfo, ColumnInfo};

/// Helper: connect to a fresh in-memory SQLite database and create a
/// sample schema for testing.
async fn setup_test_db() -> Database {
    let db = Database::new("sqlite::memory:").await
        .expect("failed to connect to in-memory SQLite");

    db.execute_raw("CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT NOT NULL UNIQUE,
        age INTEGER,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    )").await.expect("failed to create users table");

    db.execute_raw("CREATE TABLE posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        title TEXT NOT NULL,
        body TEXT,
        published BOOLEAN DEFAULT 0,
        FOREIGN KEY (user_id) REFERENCES users(id)
    )").await.expect("failed to create posts table");

    db.execute_raw("INSERT INTO users (name, email, age) VALUES
        ('Alice', 'alice@example.com', 30),
        ('Bob', 'bob@example.com', 25),
        ('Charlie', 'charlie@example.com', 35)
    ").await.expect("failed to seed users");

    db.execute_raw("INSERT INTO posts (user_id, title, body, published) VALUES
        (1, 'Hello World', 'First post content', 1),
        (1, 'Rust Tips', 'Advanced Rust patterns', 1),
        (2, 'Draft Post', 'Work in progress', 0)
    ").await.expect("failed to seed posts");

    db
}

#[tokio::test]
async fn test_connect_in_memory_sqlite() {
    let db = Database::new("sqlite::memory:").await;
    assert!(db.is_ok(), "should connect to in-memory SQLite");
}

#[tokio::test]
async fn test_schema_introspection() {
    let db = setup_test_db().await;
    let schema = db.get_schema().await.expect("failed to get schema");

    assert!(schema.tables.len() >= 2, "should find at least users and posts tables");

    let users_table = schema.tables.iter().find(|t| t.name == "users");
    assert!(users_table.is_some(), "should find users table");

    let users = users_table.unwrap();
    assert!(users.columns.len() >= 4, "users should have at least 4 columns");

    let name_col = users.columns.iter().find(|c| c.name == "name");
    assert!(name_col.is_some(), "should find name column");
}

#[tokio::test]
async fn test_select_query() {
    let db = setup_test_db().await;
    let result = db.execute("SELECT name, email FROM users ORDER BY name")
        .await.expect("query failed");

    assert_eq!(result.columns.len(), 2, "should return 2 columns");
    assert_eq!(result.rows.len(), 3, "should return 3 rows");
    assert_eq!(result.rows[0][0], "Alice", "first row should be Alice");
}

#[tokio::test]
async fn test_query_with_where_clause() {
    let db = setup_test_db().await;
    let result = db.execute("SELECT name FROM users WHERE age > 28 ORDER BY name")
        .await.expect("query failed");

    assert_eq!(result.rows.len(), 2, "should return Alice and Charlie");
    assert_eq!(result.rows[0][0], "Alice");
    assert_eq!(result.rows[1][0], "Charlie");
}

#[tokio::test]
async fn test_join_query() {
    let db = setup_test_db().await;
    let result = db.execute(
        "SELECT u.name, p.title FROM users u JOIN posts p ON u.id = p.user_id WHERE p.published = 1 ORDER BY p.title"
    ).await.expect("join query failed");

    assert_eq!(result.rows.len(), 2, "should return 2 published posts");
}

#[tokio::test]
async fn test_aggregate_query() {
    let db = setup_test_db().await;
    let result = db.execute("SELECT COUNT(*) as total FROM users")
        .await.expect("aggregate query failed");

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], "3", "should count 3 users");
}

#[tokio::test]
async fn test_insert_and_verify() {
    let db = setup_test_db().await;

    db.execute("INSERT INTO users (name, email, age) VALUES ('Dave', 'dave@example.com', 28)")
        .await.expect("insert failed");

    let result = db.execute("SELECT COUNT(*) FROM users")
        .await.expect("count query failed");
    assert_eq!(result.rows[0][0], "4", "should now have 4 users");
}

#[tokio::test]
async fn test_update_query() {
    let db = setup_test_db().await;

    db.execute("UPDATE users SET age = 31 WHERE name = 'Alice'")
        .await.expect("update failed");

    let result = db.execute("SELECT age FROM users WHERE name = 'Alice'")
        .await.expect("select failed");
    assert_eq!(result.rows[0][0], "31");
}

#[tokio::test]
async fn test_invalid_sql_returns_error() {
    let db = setup_test_db().await;
    let result = db.execute("SELECT * FROM nonexistent_table").await;
    assert!(result.is_err(), "should error on nonexistent table");
}

#[tokio::test]
async fn test_empty_result_set() {
    let db = setup_test_db().await;
    let result = db.execute("SELECT * FROM users WHERE age > 1000")
        .await.expect("query should succeed");
    assert_eq!(result.rows.len(), 0, "should return empty result set");
}

#[tokio::test]
async fn test_execution_time_tracked() {
    let db = setup_test_db().await;
    let result = db.execute("SELECT * FROM users")
        .await.expect("query failed");
    assert!(result.execution_time.as_nanos() > 0, "execution time should be tracked");
}
