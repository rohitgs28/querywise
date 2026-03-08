# querywise ⚡

> Talk to your database like a human. Get SQL back like a pro.

```
$ querywise "show me all users who signed up last week and haven't logged in since"
→ SELECT * FROM users WHERE created_at >= NOW() - INTERVAL 7 DAY AND last_login < NOW() - INTERVAL 7 DAY;
✓ Executed in 3ms — 42 rows returned
```

**querywise** is an AI-powered database query generator and executor built in Rust. You write plain English, it writes the SQL, runs it, and shows you the results — all from your terminal.

No more context-switching to StackOverflow. No more forgetting JOIN syntax at 2am.

---

## Features

- **Natural language to SQL** — powered by OpenAI / local LLM support
- **TTL-aware query cache** — repeated questions skip the LLM entirely
- **Interactive REPL** — run queries in a live session with history
- **Multiple DB backends** — PostgreSQL, MySQL, SQLite
- **Query explanation mode** — understand what a query does before you run it
- **Safe mode** — flags destructive queries (DROP, DELETE, TRUNCATE) before executing
- **Export results** — to CSV, JSON, or pretty-printed table

---

## Installation

```bash
cargo install querywise
```

Or build from source:

```bash
git clone https://github.com/rohitgs28/querywise
cd querywise
cargo build --release
./target/release/querywise --help
```

---

## Quick Start

```bash
# Connect to a database
querywise connect postgres://localhost/mydb

# Ask a question
querywise "how many orders were placed today?"

# Start interactive REPL
querywise repl

# Explain a query without running it
querywise explain "delete all inactive users"
```

---

## Architecture

```
User Input (natural language)
        │
        ▼
   Query Cache ──── HIT ──▶ Return cached result
        │ MISS
        ▼
   AI Layer (OpenAI / local LLM)
        │
        ▼
   SQL Query (validated)
        │
        ▼
   DB Executor (postgres / mysql / sqlite)
        │
        ▼
   Result Formatter (table / json / csv)
```

---

## Configuration

```toml
# querywise.toml
[database]
url = "postgres://localhost/mydb"

[ai]
provider = "openai"       # or "ollama" for local
model = "gpt-4o-mini"
api_key_env = "OPENAI_API_KEY"

[cache]
enabled = true
ttl_secs = 300
max_entries = 500
```

---

## Contributing

PRs welcome! Check out the open issues for good first contributions.

```bash
cargo test        # run all tests
cargo clippy      # lint
cargo fmt         # format
```

---

## License

MIT
