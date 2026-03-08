# Contributing to QueryWise

Thanks for taking the time to contribute. QueryWise is a terminal-based AI database client written in Rust.

## Getting Started

### Prerequisites
- Rust 1.75+ (`rustup update stable`)
- A supported database (PostgreSQL, MySQL, or SQLite)
- An AI provider API key (Anthropic, OpenAI, or local Ollama)

### Local Setup
```bash
git clone https://github.com/rohitgs28/querywise
cd querywise
cargo build
cargo run -- -f /tmp/test.db
```

## Project Structure
```
src/
├── main.rs              # CLI entrypoint
├── app.rs               # Core state machine + event loop
├── db/
│   ├── connection.rs   # sqlx pool, query execution
│   ├── history.rs       # Persistent query history
│   └── query_cache.rs   # TTL-aware LRU cache (auto-purges expired)
└── ui/renderer.rs      # ratatui layout
```

## Submitting a PR
1. Fork and branch: `git checkout -b feat/your-feature`
2. Before pushing:
   ```bash
   cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
   ```
3. Open a PR against `main`.

## Code Style
- `cargo fmt` before every commit
- No `unwrap()` in production paths
- Unit tests for all new data structures
- No blocking calls in the render path

## License
AGPL-3.0 -- same license applies to all contributions.
