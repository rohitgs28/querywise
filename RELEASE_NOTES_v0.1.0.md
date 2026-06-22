# QueryWise v0.1.0

The first public release of **QueryWise**, an AI-powered terminal database
client written in Rust. Ask your database a question in plain English, get SQL
back, and see results. When a query fails, it rewrites and retries.

## Highlights

- Natural-language to SQL (Anthropic, OpenAI, or local Ollama)
- Self-healing queries with automatic retry
- `:explain` and `:plan` commands
- PostgreSQL, MySQL, SQLite via sqlx
- ratatui TUI with syntax highlighting, 7 themes, history, bookmarks, CSV export
- Runs fully offline with Ollama

## Install

Download a prebuilt binary for your platform from the assets below, or build
from source:

```bash
cargo build --release
```

See the [README](https://github.com/rohitgs28/querywise#readme) for usage.
