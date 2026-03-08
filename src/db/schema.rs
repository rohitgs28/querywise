use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
    pub db_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

impl SchemaInfo {
    pub fn to_ddl(&self) -> String {
        let mut ddl = String::new();
        for table in &self.tables {
            ddl.push_str(&format!(
                "-- Table: {} (~{} rows)\nCREATE TABLE {} (\n",
                table.name, table.row_count, table.name
            ));
            let cols: Vec<String> = table
                .columns
                .iter()
                .map(|c| {
                    let mut parts = format!("  {} {}", c.name, c.data_type);
                    if c.is_primary_key {
                        parts.push_str(" PRIMARY KEY");
                    }
                    if !c.is_nullable {
                        parts.push_str(" NOT NULL");
                    }
                    parts
                })
                .collect();
            ddl.push_str(&cols.join(",\n"));
            ddl.push_str("\n);\n\n");
        }
        ddl
    }
}
