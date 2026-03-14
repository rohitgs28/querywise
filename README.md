<p align="center">
  <h1 align="center">QueryWise</h1>
  <p align="center">
    <strong>Talk to your database in plain English. Get SQL back in milliseconds.</strong>
  </p>
  <p align="center">
    <a href="https://github.com/rohitgs28/querywise/actions"><img src="https://github.com/rohitgs28/querywise/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/rohitgs28/querywise/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
    <a href="https://github.com/rohitgs28/querywise"><img src="https://img.shields.io/badge/built_with-Rust-dea584.svg" alt="Built with Rust"></a>
    <a href="https://ratatui.rs"><img src="https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff" alt="Built with Ratatui"></a>
  </p>
</p>

---

QueryWise is an AI-powered terminal database client built in Rust. You type natural language questions, it generates SQL, executes it, and shows results. When a query fails, it auto-fixes and retries. No browser, no GUI, no context switching.

```
$ querywise -f mydb.sqlite

> show me users who signed up this week but never placed an order

  SQL: SELECT u.* FROM users u
       LEFT JOIN orders o ON u.id = o.user_id
       WHERE u.created_at >= date('now', '-7 days')
       AND o.id IS NULL;

  ✓ 12 rows in 3ms
```

## Why QueryWise

Most database tools assume you already know the exact SQL. QueryWise doesn't. Describe what you want in plain English, and it writes the query, runs it, and shows the results. If the generated query fails, it reads the error, rewrites the SQL, and retries automatically.

No other terminal database client does this.

| Feature | QueryWise | pgcli | usql | dbcli |
|---------|:---------:|:-----:|:----:|:-----:|
| Natural language to SQL | Yes | | | |
| Self-healing queries | Yes | | | |
| AI query explanation | Yes | | | |
| Local LLM support | Yes | | | |
| Safe mode | Yes | | | |
| Multi-database | Yes | | Yes | |
| SQL syntax highlighting | Yes | Yes | Yes | Yes |
| Query history | Yes | Yes | Yes | Yes |
| CSV export | Yes | Yes | | Yes |

## Features

**AI-Native**
- Natural language to SQL generation via Anthropic Claude, OpenAI, or local Ollama models
- Self-healing queries: auto-detects errors, rewrites, and retries
- `:explain` command: get a plain-English breakdown of any SQL query
- Conversational context: follow-up questions reference previous results

**Database Support**
- PostgreSQL, MySQL, and SQLite via sqlx
- Schema browser with table/column introspection and row counts
- Connection pooling with health checks and acquire timeouts

**Terminal UI**
- Four-panel layout: Schema, AI Chat, SQL Preview, Results
- SQL syntax highlighting with tokenizer (80+ keywords, 35+ functions, 20+ data types)
- Persistent query history with up/down navigation and fuzzy recall
- TTL-aware LRU query cache with atomic persistence
- CSV export with Ctrl+E
- Safe mode blocks destructive queries (INSERT, UPDATE, DELETE, DROP, ALTER, TRUNCATE)

**Configurable**
- AI model selection via config.toml, environment variables, or `--model` CLI flag
- Works fully offline with Ollama (codellama, mistral, deepseek-coder, or any local model)
- Keybinding-driven workflow designed for speed

## Installation

**From source**

```bash
git clone https://github.com/rohitgs28/querywise
cd querywise
cargo build --release
```

The binary will be at `./target/release/querywise`.

## Quick Start

```bash
# Connect to SQLite
querywise -f ./data.db

# Connect to PostgreSQL
querywise -u postgres://user:pass@localhost/mydb

# Connect to MySQL
querywise -u mysql://user:pass@localhost/mydb

# Use a local Ollama model instead of cloud AI
querywise -f ./data.db --ai-provider ollama --model codellama

# Run a single query and exit
querywise -f ./data.db -e "SELECT count(*) FROM users"
```

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Run query or natural language question |
| `Up` / `Down` | Navigate query history |
| `Tab` | Cycle panels: Schema, Chat, SQL Preview, Results |
| `F1` `F2` `F3` `F4` | Jump directly to a panel |
| `Ctrl+S` | Toggle safe mode |
| `Ctrl+E` | Export results to CSV |
| `Ctrl+L` | Clear chat and conversation context |
| `Ctrl+Q` | Quit |
| `:explain <query>` | Get AI explanation of SQL |
| `:explain` | Explain the last generated query |

## Architecture

```
User Input (natural language or SQL)
│
├─ :explain ─────→ AI Agent ───→ Plain-English explanation
│
├─ SQL detected ──→ Safe mode check ──→ Execute
│
└─ Natural language ─→ AI Agent ──────────→ Generate SQL
                                              │
                                    Safe mode check
                                              │
                                         Execute query
                                              │
                                  ┌─────────┴─────────┐
                                  │                   │
                               Success             Failure
                                  │                   │
                            Show results       Auto-fix query
                            Save to history    Retry execution
```

```
src/
├── main.rs              # CLI: -u, -f, -e, --ai-provider, --model
├── app.rs               # Event loop, input handling, query orchestration
├── config/mod.rs        # TOML config (~/.config/querywise/config.toml)
├── ai/
│   ├── agent.rs         # generate_sql(), fix_query(), explain_query()
│   └── provider.rs      # Anthropic / OpenAI / Ollama with timeouts
├── db/
│   ├── connection.rs    # Connect, execute, introspect (Postgres/MySQL/SQLite)
│   ├── schema.rs        # SchemaInfo, TableInfo, ColumnInfo
│   ├── history.rs       # Persistent query history (JSON, dedup, 1000 cap)
│   └── query_cache.rs   # TTL LRU cache with atomic save
└── ui/
    ├── renderer.rs      # 4-panel ratatui layout
    └── components/
        └── sql_highlight.rs  # SQL tokenizer and syntax highlighting
```

## Configuration

QueryWise reads from `~/.config/querywise/config.toml`:

```toml
# AI provider settings
anthropic_api_key = "sk-ant-..."
# openai_api_key = "sk-..."

# Local LLM via Ollama (no API key needed)
# ollama_url = "http://localhost:11434"
# ollama_model = "codellama"

# default_ai_provider = "anthropic"  # or "openai" or "ollama"
```

Environment variables also work: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OLLAMA_MODEL`.

## Contributing

Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions.

```bash
cargo test        # run tests (25+ unit tests)
cargo clippy      # lint
cargo fmt --check # format check
```

Areas where help is needed:
- Tab completion for table and column names
- Multi-line query editor
- Vim keybindings mode
- Additional database backends
- Demo GIF for this README

See [ROADMAP.md](ROADMAP.md) for the full plan.

## Built With

- [Rust](https://www.rust-lang.org/) for performance and safety
- [Ratatui](https://ratatui.rs/) for the terminal UI
- [sqlx](https://github.com/launchbadge/sqlx) for async database access
- [Tokio](https://tokio.rs/) for the async runtime
- [clap](https://github.com/clap-rs/clap) for CLI argument parsing

## License

MIT. See [LICENSE](LICENSE) for details.
