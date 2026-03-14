use anyhow::Result;
use sqlx::any::{AnyPool, AnyRow, AnyPoolOptions};
use sqlx::Row;
use super::schema::{SchemaInfo, TableInfo, ColumnInfo};

/// Sanitizes a SQL identifier to prevent injection attacks.
/// Rejects names containing double quotes, null bytes, or backslashes
/// which could break out of quoted identifier context.
fn sanitize_identifier(name: &str) -> Result<String> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Empty identifier name"));
    }
    if name.contains('"') || name.contains('\0') || name.contains('\\') {
        return Err(anyhow::anyhow!(
            "Invalid identifier '{}': contains prohibited characters",
            name
        ));
    }
    Ok(format!("\"{}\""  , name))
}

pub struct Database {
    pool: AnyPool,
    pub db_type: String,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        sqlx::any::install_default_drivers();

        let db_type = if url.starts_with("postgres") {
            "postgresql"
        } else if url.starts_with("mysql") {
            "mysql"
        } else if url.starts_with("sqlite") {
            "sqlite"
        } else {
            "unknown"
        }
        .to_string();

        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(url)
            .await?;

        Ok(Self { pool, db_type })
    }

    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult> {
        let start = std::time::Instant::now();
        let rows: Vec<AnyRow> = sqlx::query(sql).fetch_all(&self.pool).await?;
        let duration = start.elapsed();

        let columns: Vec<String> = if !rows.is_empty() {
            rows[0]
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect()
        } else {
            vec![]
        };

        let data: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        row.try_get_raw(i)
                            .ok()
                            .and_then(|v| {
                                if v.is_null() {
                                    Some("NULL".to_string())
                                } else {
                                    // Try common types
                                    row.try_get::<String, _>(i)
                                        .ok()
                                        .or_else(|| row.try_get::<i64, _>(i).ok().map(|v| v.to_string()))
                                        .or_else(|| row.try_get::<f64, _>(i).ok().map(|v| v.to_string()))
                                        .or_else(|| row.try_get::<bool, _>(i).ok().map(|v| v.to_string()))
                                        .or_else(|| Some("<binary>".to_string()))
                                }
                            })
                            .unwrap_or_else(|| "NULL".to_string())
                    })
                    .collect()
            })
            .collect();

        Ok(QueryResult {
            columns,
            rows: data,
            row_count: rows.len(),
            execution_time_ms: duration.as_millis() as u64,
        })
    }

    pub async fn introspect(&self) -> Result<SchemaInfo> {
        let tables = match self.db_type.as_str() {
            "postgresql" => self.introspect_postgres().await?,
            "mysql" => self.introspect_mysql().await?,
            "sqlite" => self.introspect_sqlite().await?,
            _ => vec![],
        };

        Ok(SchemaInfo {
            tables,
            db_type: self.db_type.clone(),
        })
    }

    async fn introspect_postgres(&self) -> Result<Vec<TableInfo>> {
        let table_rows: Vec<AnyRow> = sqlx::query(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
             ORDER BY table_name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tables = Vec::new();
        for row in &table_rows {
            let table_name: String = row.try_get("table_name")?;
            let columns = self.get_postgres_columns(&table_name).await?;

            let count_row: AnyRow = let safe_name = sanitize_identifier(&table_name)?;
            sqlx::query(&format!("SELECT COUNT(*)::text as cnt FROM {}", safe_name))
            .fetch_one(&self.pool)
            .await?;
            let count: String = count_row.try_get("cnt")?;

            tables.push(TableInfo {
                name: table_name,
                columns,
                row_count: count.parse().unwrap_or(0),
            });
        }
        Ok(tables)
    }

    async fn get_postgres_columns(&self, table: &str) -> Result<Vec<ColumnInfo>> {
        let rows: Vec<AnyRow> = sqlx::query(
            "SELECT c.column_name, c.data_type, c.is_nullable, \
             CASE WHEN pk.column_name IS NOT NULL THEN 'YES' ELSE 'NO' END as is_pk \
             FROM information_schema.columns c \
             LEFT JOIN ( \
               SELECT kcu.column_name FROM information_schema.table_constraints tc \
               JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
               WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_name = $1 \
             ) pk ON c.column_name = pk.column_name \
             WHERE c.table_schema = 'public' AND c.table_name = $1 \
             ORDER BY c.ordinal_position",
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| ColumnInfo {
                name: r.try_get("column_name").unwrap_or_default(),
                data_type: r.try_get("data_type").unwrap_or_default(),
                is_nullable: r
                    .try_get::<String, _>("is_nullable")
                    .unwrap_or_default()
                    == "YES",
                is_primary_key: r
                    .try_get::<String, _>("is_pk")
                    .unwrap_or_default()
                    == "YES",
            })
            .collect())
    }

    async fn introspect_mysql(&self) -> Result<Vec<TableInfo>> {
        let table_rows: Vec<AnyRow> = sqlx::query(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE' \
             ORDER BY table_name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tables = Vec::new();
        for row in &table_rows {
            let table_name: String = row.try_get("table_name")?;

            let col_rows: Vec<AnyRow> = sqlx::query(
                "SELECT column_name, data_type, is_nullable, column_key \
                 FROM information_schema.columns \
                 WHERE table_schema = DATABASE() AND table_name = ? \
                 ORDER BY ordinal_position",
            )
            .bind(&table_name)
            .fetch_all(&self.pool)
            .await?;

            let columns: Vec<ColumnInfo> = col_rows
                .iter()
                .map(|r| ColumnInfo {
                    name: r.try_get("column_name").unwrap_or_default(),
                    data_type: r.try_get("data_type").unwrap_or_default(),
                    is_nullable: r
                        .try_get::<String, _>("is_nullable")
                        .unwrap_or_default()
                        == "YES",
                    is_primary_key: r
                        .try_get::<String, _>("column_key")
                        .unwrap_or_default()
                        == "PRI",
                })
                .collect();

            let count_row: AnyRow = let safe_name = sanitize_identifier(&table_name)?;
            sqlx::query(&format!("SELECT CAST(COUNT(*) AS CHAR) as cnt FROM {}", safe_name))
            .fetch_one(&self.pool)
            .await?;
            let count: String = count_row.try_get("cnt")?;

            tables.push(TableInfo {
                name: table_name,
                columns,
                row_count: count.parse().unwrap_or(0),
            });
        }
        Ok(tables)
    }

    async fn introspect_sqlite(&self) -> Result<Vec<TableInfo>> {
        let table_rows: Vec<AnyRow> = sqlx::query(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tables = Vec::new();
        for row in &table_rows {
            let table_name: String = row.try_get("name")?;

            let col_rows: Vec<AnyRow> =
                let safe_name = sanitize_identifier(&table_name)?;

            let col_rows: Vec<AnyRow> =
                sqlx::query(&format!("PRAGMA table_info({})", safe_name))
                    .fetch_all(&self.pool)
                    .await?;

            let columns: Vec<ColumnInfo> = col_rows
                .iter()
                .map(|r| ColumnInfo {
                    name: r.try_get("name").unwrap_or_default(),
                    data_type: r.try_get("type").unwrap_or_default(),
                    is_nullable: r.try_get::<i64, _>("notnull").unwrap_or(0) == 0,
                    is_primary_key: r.try_get::<i64, _>("pk").unwrap_or(0) > 0,
                })
                .collect();

            let count_row: AnyRow =
                sqlx::query(&format!("SELECT COUNT(*) as cnt FROM {}", safe_name))
                    .fetch_one(&self.pool)
                    .await?;
            let count: i64 = count_row.try_get("cnt").unwrap_or(0);

            tables.push(TableInfo {
                name: table_name,
                columns,
                row_count: count,
            });
        }
        Ok(tables)
    }
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}\n
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier_valid() {
        assert_eq!(sanitize_identifier("users").unwrap(), "\"users\"");
        assert_eq!(sanitize_identifier("my_table").unwrap(), "\"my_table\"");
    }

    #[test]
    fn test_sanitize_identifier_rejects_injection() {
        assert!(sanitize_identifier("").is_err());
        assert!(sanitize_identifier("users\"; DROP TABLE").is_err());
        assert!(sanitize_identifier("users\0evil").is_err());
        assert!(sanitize_identifier("users\\evil").is_err());
    }
}
