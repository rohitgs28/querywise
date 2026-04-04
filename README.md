# QueryWise

**Talk to your database in plain English. Get SQL back in milliseconds.**

[![CI](https://github.com/rohitgs28/querywise/workflows/CI/badge.svg)](https://github.com/rohitgs28/querywise/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-dea584.svg)](https://www.rust-lang.org/)
[![Built with Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs)

---

QueryWise is an AI-powered terminal database client built in Rust. Type a question in plain English, get SQL back, see results. When a query fails, it rewrites the SQL and retries automatically. No browser, no GUI, no context switching.

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

Every other terminal database client assumes you already know SQL. QueryWise doesn't. Describe what you want, and it writes the query, runs it, and shows results. If the query fails, it reads the error, rewrites, and retries. No other TUI database client does this.

| Feature | QueryWise | pgcli | usql | dbcli |
|---|---|---|---|---|
| Natural language to SQL | ✅ | ❌ | ❌ | ❌ |
| Self-healing queries | ✅ | ❌ | ❌ | ❌ |
| AI query explanation | ✅ | ❌ | ❌ | ❌ |
| Query plan visualization | ✅ | ❌ | ❌ | ❌ |
| Local LLM support (Ollama) | ✅ | ❌ | ❌ | ❌ |
| Safe mode | ✅ | ❌ | ❌ | ❌ |
| Color themes (7 built-in) | ✅ | ✅ | ❌ | ✅ |
| Query bookmarks | ✅ | ❌ | ❌ | ❌ |
| Multi-database | ✅ | ❌ | ✅ | ❌ |
| SQL syntax highlighting | ✅ | ✅ | ✅ | ✅ |
| Query history | ✅ | ✅ | ✅ | ✅ |
| CSV export | ✅ | ✅ | ❌ | ✅ |
| Works offline | ✅ (Ollama) | ✅ | ✅ | ✅ |

## Features

### AI-Native

- **Natural language to SQL** via Anthropic Claude, OpenAI, or local Ollama models
- **Self-healing queries**: auto-detects errors, rewrites SQL, retries
- **`:explain` command**: plain-English breakdown of any SQL query
- **`:plan` command**: visualize query execution plans as structured trees
- **Conversational context**: follow-up questions reference previous results

### Database Support

- PostgreSQL, MySQL, and SQLite via sqlx
- Schema browser with table/column introspection and row counts
- Connection pooling with health checks

### Terminal UI

- Four-panel layout: Schema, AI Chat, SQL Preview, Results
- SQL syntax highlighting (80+ keywords, 35+ functions, 20+ data types)
- **7 built-in color themes**: dark, light, dracula, nord, monokai, solarized-dark, solarized-light
- Custom themes via config.toml with hex color overrides
- Persistent query history with up/down navigation
- **Query bookmarks** — save and recall favorite queries with Ctrl+B
- TTL-aware LRU query cache with atomic persistence
- CSV export with Ctrl+E
- Safe mode blocks destructive queries

### Configurable

- AI model selection via config.toml, environment variables, or `--model` CLI flag
- Works fully offline with Ollama (codellama, mistral, deepseek-coder)
- Keybinding-driven workflow designed for speed

## Installation

```bash
# From source
git clone https://github.com/rohitgs28/querywise
cd querywise
cargo build --release

# Binary at ./target/release/querywise
```

## Quick Start

```bash
# Connect to SQLite
querywise -f ./data.db

# Connect to PostgreSQL
querywise -u postgres://user:pass@localhost/mydb

# Connect to MySQL
querywise -u mysql://user:pass@localhost/mydb

# Use a local Ollama model (fully offline)
querywise -f ./data.db --ai-provider ollama --model codellama

# Run a single query and exit
querywise -f ./data.db -e "SELECT count(*) FROM users"
```

## Keybindings

| Key | Action |
|---|---|
| `Enter` | Run query or natural language question |
| `Up` / `Down` | Navigate query history |
| `Tab` | Cycle panels: Schema → Chat → SQL Preview → Results |
| `F1` `F2` `F3` `F4` | Jump directly to a panel |
| `Ctrl+B` | Toggle query bookmarks |
| `Ctrl+S` | Toggle safe mode |
| `Ctrl+E` | Export results to CSV |
| `Ctrl+L` | Clear chat and conversation context |
| `Ctrl+Q` | Quit |
| `:explain <query>` | Get AI explanation of SQL |
| `:explain` | Explain the last generated query |
| `:plan <query>` | Visualize query execution plan |
| `:theme <name>` | Switch color theme |

## Color Themes

Switch themes with `:theme <name>` or set in config.toml:

```toml
[theme]
name = "dracula"  # dark, light, dracula, nord, monokai, solarized-dark, solarized-light
```

Custom color overrides:

```toml
[theme]
name = "dark"

[theme.colors]
keyword = "#ff79c6"
string = "#f1fa8c"
success = "#50fa7b"
error = "#ff5555"
```

## Query Plan Visualization

```
> :plan SELECT u.*, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id

── Query Plan (PostgreSQL) ──

  HashAggregate  (cost: 45.2)  rows: 100
  ���─ Hash Left Join on orders  (cost: 35.5)  rows: 250
     ├─ Seq Scan on users  (cost: 12.0)  rows: 100
     └─ Hash  (cost: 15.0)  rows: 500
        └─ Seq Scan on orders  (cost: 15.0)  rows: 500

  Planning time:  0.234 ms
  Execution time: 1.567 ms
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

[theme]
name = "dracula"
```

Environment variables: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OLLAMA_MODEL`.

## Architecture

```
User Input (natural language or SQL)
│
├─ :explain ─────→ AI Agent ───→ Plain-English explanation
├─ :plan ────────→ EXPLAIN ────→ Structured plan tree
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
├── config/mod.rs        # TOML config with theme support
├── ai/
│   ├── agent.rs         # generate_sql(), fix_query(), explain_query()
│   └── provider.rs      # Anthropic / OpenAI / Ollama with timeouts
├── db/
│   ├── connection.rs    # Connect, execute, introspect (Postgres/MySQL/SQLite)
│   ├── schema.rs        # SchemaInfo, TableInfo, ColumnInfo
│   ├── history.rs       # Persistent query history (JSON, dedup, 1000 cap)
│   ├── query_cache.rs   # TTL LRU cache with atomic save
│   ├── query_plan.rs    # EXPLAIN parser and tree formatter
│   └── bookmarks.rs     # Query bookmarks with persistence
└── ui/
    ├── renderer.rs      # 4-panel ratatui layout
    ├── theme.rs         # 7 built-in themes + custom color overrides
    └── components/
        └── sql_highlight.rs  # SQL tokenizer and syntax highlighting
```

## Contributing

Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions.

```bash
cargo test        # run tests
cargo clippy      # lint
cargo fmt --check # format check
```

Areas where help is needed: tab completion, multi-line editor, vim keybindings, demo GIF. See [ROADMAP.md](ROADMAP.md) for the full plan.

## Built With

- [Rust](https://www.rust-lang.org/) — performance and safety
- [Ratatui](https://ratatui.rs/) — terminal UI
- [sqlx](https://github.com/launchbadge/sqlx) — async database access
- [Tokio](https://tokio.rs/) — async runtime
- [clap](https://github.com/clap-rs/clap) — CLI argument parsing

## License

MIT. See [LICENSE](LICENSE) for details.
