use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use crate::ai::{AiAgent, AiProvider};
use crate::config::AppConfig;
use crate::db::{Database, HistoryEntry, QueryHistory, SchemaInfo};
use crate::ui;

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Schema,
    QueryInput,
    SqlPreview,
    Results,
    AiChat,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct App {
    pub db: Database,
    pub schema: SchemaInfo,
    pub ai_agent: Option<AiAgent>,
    pub active_panel: Panel,
    pub query_input: String,
    pub generated_sql: String,
    pub result_columns: Vec<String>,
    pub result_rows: Vec<Vec<String>>,
    pub result_info: String,
    pub chat_messages: Vec<ChatMessage>,
    pub selected_table: usize,
    pub result_scroll: usize,
    pub schema_scroll: usize,
    pub safe_mode: bool,
    pub status: String,
    pub should_quit: bool,
    pub cursor_pos: usize,
    pub query_history: QueryHistory,
    pub saved_input: String,
}

impl App {
    pub async fn new(conn_url: String, ai_provider: String) -> Result<Self> {
        let db = Database::connect(&conn_url).await?;
        let schema = db.introspect().await?;
        let config = AppConfig::load().unwrap_or_default();
        let ai_agent = AiProvider::from_config(&ai_provider, &config)
            .ok()
            .map(AiAgent::new);

        let table_count = schema.tables.len();
        let query_history = QueryHistory::load();
        let history_count = query_history.len();

        Ok(Self {
            db,
            schema,
            ai_agent,
            active_panel: Panel::AiChat,
            query_input: String::new(),
            generated_sql: String::new(),
            result_columns: Vec::new(),
            result_rows: Vec::new(),
            result_info: String::new(),
            chat_messages: vec![ChatMessage {
                role: "info".to_string(),
                content: format!(
                    "Connected! Found {} table{}. {} queries in history. Type a question or SQL.",
                    table_count,
                    if table_count == 1 { "" } else { "s" },
                    history_count,
                ),
            }],
            selected_table: 0,
            result_scroll: 0,
            schema_scroll: 0,
            safe_mode: true,
            status: format!("Connected to {} | {} tables", conn_url, table_count),
            should_quit: false,
            cursor_pos: 0,
            query_history,
            saved_input: String::new(),
        })
    }

    pub async fn execute_and_print(&self, query: &str) -> Result<()> {
        let result = self.db.execute_query(query).await?;
        println!("{}", result.columns.join("\t"));
        for row in &result.rows {
            println!("{}", row.join("\t"));
        }
        println!("\n{} rows ({} ms)", result.row_count, result.execution_time_ms);
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| ui::render(f, self))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key.code, key.modifiers).await? {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    async fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
        match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(true),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(true),
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.safe_mode = !self.safe_mode;
                let mode = if self.safe_mode { "ON" } else { "OFF" };
                self.status = format!("Safe mode {}", mode);
                self.chat_messages.push(ChatMessage {
                    role: "info".to_string(),
                    content: format!(
                        "Safe mode {}. {}",
                        mode,
                        if self.safe_mode {
                            "Destructive queries will be blocked."
                        } else {
                            "All queries allowed. Be careful!"
                        }
                    ),
                });
                return Ok(false);
            }
            (KeyCode::Tab, _) => {
                self.cycle_panel();
                return Ok(false);
            }
            (KeyCode::F(1), _) => {
                self.active_panel = Panel::Schema;
                return Ok(false);
            }
            (KeyCode::F(2), _) => {
                self.active_panel = Panel::AiChat;
                return Ok(false);
            }
            (KeyCode::F(3), _) => {
                self.active_panel = Panel::SqlPreview;
                return Ok(false);
            }
            (KeyCode::F(4), _) => {
                self.active_panel = Panel::Results;
                return Ok(false);
            }
            _ => {}
        }

        match &self.active_panel {
            Panel::AiChat | Panel::QueryInput => match (code, modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    self.query_history.reset_navigation();
                    self.saved_input.clear();
                    self.submit_query().await?;
                }
                (KeyCode::Up, _) => {
                    if !self.query_history.is_navigating() {
                        self.saved_input = self.query_input.clone();
                    }
                    if let Some(h) = self.query_history.navigate_back(&self.saved_input) {
                        self.query_input = h.to_string();
                        self.cursor_pos = self.query_input.len();
                    }
                }
                (KeyCode::Down, _) => {
                    match self.query_history.navigate_forward() {
                        Some(h) => {
                            self.query_input = h.to_string();
                            self.cursor_pos = self.query_input.len();
                        }
                        None => {
                            self.query_input = self.saved_input.clone();
                            self.cursor_pos = self.query_input.len();
                        }
                    }
                }
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    if self.query_history.is_navigating() {
                        self.query_history.reset_navigation();
                        self.saved_input.clear();
                    }
                    self.query_input.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                }
                (KeyCode::Backspace, _) => {
                    if self.query_history.is_navigating() {
                        self.query_history.reset_navigation();
                        self.saved_input.clear();
                    }
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.query_input.remove(self.cursor_pos);
                    }
                }
                (KeyCode::Left, _) => {
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
                (KeyCode::Right, _) => {
                    if self.cursor_pos < self.query_input.len() {
                        self.cursor_pos += 1;
                    }
                }
                (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                    self.chat_messages.clear();
                    self.query_history.reset_navigation();
                    self.saved_input.clear();
                    if let Some(agent) = &mut self.ai_agent {
                        agent.clear_conversation();
                    }
                }
                _ => {}
            },
            Panel::Schema => match code {
                KeyCode::Up => {
                    self.selected_table = self.selected_table.saturating_sub(1);
                }
                KeyCode::Down => {
                    if self.selected_table + 1 < self.schema.tables.len() {
                        self.selected_table += 1;
                    }
                }
                _ => {}
            },
            Panel::Results => match (code, modifiers) {
                (KeyCode::Up, _) => {
                    self.result_scroll = self.result_scroll.saturating_sub(1);
                }
                (KeyCode::Down, _) => {
                    if self.result_scroll + 1 < self.result_rows.len() {
                        self.result_scroll += 1;
                    }
                }
                (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                    self.export_csv()?;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(false)
    }

    async fn submit_query(&mut self) -> Result<()> {
        let input = self.query_input.trim().to_string();
        if input.is_empty() {
            return Ok(());
        }

        self.query_input.clear();
        self.cursor_pos = 0;

        // Handle :explain command
        if input.starts_with(":explain ") || input.starts_with(":explain\t") {
            let target = input.trim_start_matches(":explain").trim();
            return self.handle_explain(target).await;
        }

        self.chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: input.clone(),
        });

        let sql = if self.looks_like_sql(&input) {
            input.clone()
        } else if let Some(agent) = &mut self.ai_agent {
            self.status = "Generating SQL...".to_string();
            match agent.generate_sql(&input, &self.schema).await {
                Ok(sql) => {
                    self.chat_messages.push(ChatMessage {
                        role: "sql".to_string(),
                        content: sql.clone(),
                    });
                    sql
                }
                Err(e) => {
                    self.chat_messages.push(ChatMessage {
                        role: "error".to_string(),
                        content: format!("AI error: {}", e),
                    });
                    return Ok(());
                }
            }
        } else {
            self.chat_messages.push(ChatMessage {
                role: "error".to_string(),
                content: "No AI provider configured. Enter raw SQL or set ANTHROPIC_API_KEY."
                    .to_string(),
            });
            return Ok(());
        };

        // Safe mode enforcement: block destructive queries
        if self.safe_mode && self.is_destructive_sql(&sql) {
            self.generated_sql = sql.clone();
            self.chat_messages.push(ChatMessage {
                role: "error".to_string(),
                content: format!(
                    "Blocked: destructive query while safe mode is ON. Toggle with Ctrl+S to allow."
                ),
            });
            self.status = "Query blocked (safe mode)".to_string();
            return Ok(());
        }

        self.generated_sql = sql.clone();
        self.status = "Executing...".to_string();

        match self.db.execute_query(&sql).await {
            Ok(result) => {
                let info = format!(
                    "{} rows in {} ms",
                    result.row_count, result.execution_time_ms
                );
                self.chat_messages.push(ChatMessage {
                    role: "result".to_string(),
                    content: info.clone(),
                });
                self.result_columns = result.columns;
                self.result_rows = result.rows;
                self.result_info = info;
                self.result_scroll = 0;
                self.status = format!("Query completed | {}", self.result_info);

                self.query_history
                    .push(HistoryEntry {
                        input: input.clone(),
                        sql: sql.clone(),
                        success: true,
                        row_count: result.row_count,
                        execution_time_ms: result.execution_time_ms,
                        timestamp: Local::now(),
                        db_type: self.schema.db_type.clone(),
                    })
                    .ok();
            }
            Err(e) => {
                let err = e.to_string();
                if let Some(agent) = &mut self.ai_agent {
                    self.status = "Query failed. Auto-fixing...".to_string();
                    self.chat_messages.push(ChatMessage {
                        role: "info".to_string(),
                        content: format!("Query failed: {}. Attempting fix...", err),
                    });

                    match agent.fix_query(&sql, &err, &self.schema).await {
                        Ok(fixed) => {
                            self.chat_messages.push(ChatMessage {
                                role: "sql".to_string(),
                                content: format!("Fixed: {}", fixed),
                            });
                            self.generated_sql = fixed.clone();

                            // Also enforce safe mode on the fixed query
                            if self.safe_mode && self.is_destructive_sql(&fixed) {
                                self.chat_messages.push(ChatMessage {
                                    role: "error".to_string(),
                                    content: "Blocked: auto-fixed query is destructive. Toggle safe mode with Ctrl+S.".to_string(),
                                });
                                self.status = "Fix blocked (safe mode)".to_string();
                                return Ok(());
                            }

                            match self.db.execute_query(&fixed).await {
                                Ok(result) => {
                                    let info = format!(
                                        "{} rows in {} ms (auto-fixed)",
                                        result.row_count, result.execution_time_ms
                                    );
                                    self.chat_messages.push(ChatMessage {
                                        role: "result".to_string(),
                                        content: info.clone(),
                                    });
                                    self.result_columns = result.columns;
                                    self.result_rows = result.rows;
                                    self.result_info = info;
                                    self.result_scroll = 0;
                                    self.status = "Auto-fixed and executed".to_string();

                                    self.query_history
                                        .push(HistoryEntry {
                                            input: input.clone(),
                                            sql: fixed.clone(),
                                            success: true,
                                            row_count: result.row_count,
                                            execution_time_ms: result.execution_time_ms,
                                            timestamp: Local::now(),
                                            db_type: self.schema.db_type.clone(),
                                        })
                                        .ok();
                                }
                                Err(e2) => {
                                    self.chat_messages.push(ChatMessage {
                                        role: "error".to_string(),
                                        content: format!("Fix also failed: {}", e2),
                                    });
                                    self.status = "Query failed".to_string();
                                }
                            }
                        }
                        Err(e2) => {
                            self.chat_messages.push(ChatMessage {
                                role: "error".to_string(),
                                content: format!("Could not auto-fix: {}", e2),
                            });
                            self.status = "Query failed".to_string();
                        }
                    }
                } else {
                    self.chat_messages.push(ChatMessage {
                        role: "error".to_string(),
                        content: format!("Error: {}", err),
                    });
                    self.status = "Query failed".to_string();
                }
            }
        }
        Ok(())
    }

    async fn handle_explain(&mut self, target: &str) -> Result<()> {
        if target.is_empty() {
            // Explain the last generated SQL
            if self.generated_sql.is_empty() {
                self.chat_messages.push(ChatMessage {
                    role: "error".to_string(),
                    content: "Nothing to explain. Provide SQL or run a query first.".to_string(),
                });
                return Ok(());
            }
            self.chat_messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!(":explain {}", self.generated_sql),
            });
            let sql_to_explain = self.generated_sql.clone();
            return self.do_explain(&sql_to_explain).await;
        }

        self.chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!(":explain {}", target),
        });

        // If target looks like SQL, explain it directly
        if self.looks_like_sql(target) {
            return self.do_explain(target).await;
        }

        // Otherwise, generate SQL first then explain it
        if let Some(agent) = &mut self.ai_agent {
            self.status = "Generating SQL...".to_string();
            match agent.generate_sql(target, &self.schema).await {
                Ok(sql) => {
                    self.chat_messages.push(ChatMessage {
                        role: "sql".to_string(),
                        content: sql.clone(),
                    });
                    self.generated_sql = sql.clone();
                    return self.do_explain(&sql).await;
                }
                Err(e) => {
                    self.chat_messages.push(ChatMessage {
                        role: "error".to_string(),
                        content: format!("AI error: {}", e),
                    });
                }
            }
        } else {
            self.chat_messages.push(ChatMessage {
                role: "error".to_string(),
                content: "No AI provider configured for explain.".to_string(),
            });
        }
        Ok(())
    }

    async fn do_explain(&mut self, sql: &str) -> Result<()> {
        if let Some(agent) = &self.ai_agent {
            self.status = "Explaining query...".to_string();
            match agent.explain_query(sql).await {
                Ok(explanation) => {
                    self.chat_messages.push(ChatMessage {
                        role: "info".to_string(),
                        content: explanation,
                    });
                    self.status = "Explanation ready".to_string();
                }
                Err(e) => {
                    self.chat_messages.push(ChatMessage {
                        role: "error".to_string(),
                        content: format!("Explain error: {}", e),
                    });
                    self.status = "Explain failed".to_string();
                }
            }
        } else {
            self.chat_messages.push(ChatMessage {
                role: "error".to_string(),
                content: "No AI provider configured for explain.".to_string(),
            });
        }
        Ok(())
    }

    /// Returns true if the SQL statement is destructive (modifies data or schema).
    fn is_destructive_sql(&self, sql: &str) -> bool {
        let upper = sql.trim().to_uppercase();
        upper.starts_with("INSERT")
            || upper.starts_with("UPDATE")
            || upper.starts_with("DELETE")
            || upper.starts_with("DROP")
            || upper.starts_with("ALTER")
            || upper.starts_with("CREATE")
            || upper.starts_with("TRUNCATE")
            || upper.starts_with("REPLACE")
            || upper.starts_with("MERGE")
            || upper.starts_with("GRANT")
            || upper.starts_with("REVOKE")
    }

    fn export_csv(&mut self) -> Result<()> {
        if self.result_columns.is_empty() {
            self.status = "Nothing to export".to_string();
            return Ok(());
        }

        let filename = format!(
            "querywise_export_{}.csv",
            Local::now().format("%Y%m%d_%H%M%S")
        );
        let path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(&filename);

        let mut wtr = csv::Writer::from_path(&path)?;
        wtr.write_record(&self.result_columns)?;
        for row in &self.result_rows {
            wtr.write_record(row)?;
        }
        wtr.flush()?;

        self.status = format!("Exported {} rows to ~/{}", self.result_rows.len(), filename);
        self.chat_messages.push(ChatMessage {
            role: "info".to_string(),
            content: format!("Exported to {}", path.display()),
        });
        Ok(())
    }

    fn looks_like_sql(&self, input: &str) -> bool {
        let u = input.trim().to_uppercase();
        u.starts_with("SELECT")
            || u.starts_with("INSERT")
            || u.starts_with("UPDATE")
            || u.starts_with("DELETE")
            || u.starts_with("CREATE")
            || u.starts_with("ALTER")
            || u.starts_with("DROP")
            || u.starts_with("WITH")
            || u.starts_with("EXPLAIN")
            || u.starts_with("SHOW")
            || u.starts_with("DESCRIBE")
            || u.starts_with("PRAGMA")
    }

    fn cycle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Schema => Panel::AiChat,
            Panel::AiChat => Panel::SqlPreview,
            Panel::SqlPreview => Panel::Results,
            Panel::Results => Panel::Schema,
            Panel::QueryInput => Panel::AiChat,
        };
    }
}
