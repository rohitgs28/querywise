# Contributing to QueryWise

Thank you for considering contributing to QueryWise. Whether you are fixing a typo, adding a test, or building a major feature, every contribution matters and is appreciated.

This guide will get you from zero to your first merged PR.

## Table of Contents

- [Finding something to work on](#finding-something-to-work-on)
- [Development setup](#development-setup)
- [Project architecture](#project-architecture)
- [Making changes](#making-changes)
- [Code standards](#code-standards)
- [Commit conventions](#commit-conventions)
- [Pull request process](#pull-request-process)
- [Getting help](#getting-help)

## Finding something to work on

The best place to start is the [Issues](https://github.com/rohitgs28/querywise/issues) page.

**New to the project?** Look for issues labeled [`good first issue`](https://github.com/rohitgs28/querywise/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22). These are scoped, well-specified, and include pointers to the exact files you will need to change.

**Looking for something more challenging?** Issues labeled [`help wanted`](https://github.com/rohitgs28/querywise/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) are larger features where design input is welcome.

**Have your own idea?** Open an issue first. Describe the problem you want to solve and how you plan to approach it. This avoids duplicated effort and gives the maintainer a chance to provide direction before you write code.

**No issue needed for:**
- Fixing typos or broken links
- Improving documentation
- Adding or improving tests

## Development setup

### Prerequisites

- **Rust 1.75 or later.** Install via [rustup](https://rustup.rs/): `rustup update stable`
- **A database for testing.** SQLite works out of the box with no setup. PostgreSQL and MySQL are optional.
- **An AI provider (optional).** Set `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or run [Ollama](https://ollama.ai) locally. QueryWise works without an AI provider; you just cannot use natural language queries.

### Clone and build

```bash
git clone https://github.com/rohitgs28/querywise
cd querywise
cargo build
```

### Run locally

```bash
# Quick test with SQLite
cargo run -- -f ./test.db

# With a PostgreSQL database
cargo run -- -u postgres://user:pass@localhost/mydb

# With Ollama (local LLM, no API key needed)
cargo run -- -f ./test.db --ai-provider ollama --model codellama
```

### Run the test suite

```bash
cargo test              # run all tests
cargo clippy            # lint (must pass with zero warnings)
cargo fmt --check       # formatting check
```

All three must pass before you open a PR. CI runs these automatically.

## Project architecture

```
src/
├── main.rs              # CLI entrypoint (clap argument parsing)
├── app.rs               # Core state machine, event loop, query orchestration
├── config/
│   └── mod.rs           # TOML config loading (~/.config/querywise/config.toml)
├── ai/
│   ├── agent.rs         # AI orchestration: generate_sql, fix_query, explain_query
│   └── provider.rs      # HTTP clients for Anthropic, OpenAI, Ollama
├── db/
│   ├── connection.rs    # Database pool, query execution, schema introspection
│   ├── schema.rs        # SchemaInfo, TableInfo, ColumnInfo data structures
│   ├── history.rs       # Persistent query history with dedup and fuzzy navigation
│   └── query_cache.rs   # TTL-aware LRU cache with atomic file persistence
└── ui/
    ├── renderer.rs      # Four-panel ratatui layout (schema, chat, SQL, results)
    └── components/
        └── sql_highlight.rs  # SQL tokenizer and syntax highlighting
```

### Key data flow

1. User types input in the chat panel
2. `app.rs submit_query()` determines if the input is SQL or natural language
3. Natural language goes to `ai/agent.rs generate_sql()` which calls the configured provider
4. Generated SQL passes through safe mode check (`is_destructive_sql()`)
5. SQL is executed via `db/connection.rs execute_query()`
6. If execution fails and an AI provider is configured, `fix_query()` auto-repairs and retries
7. Results render in the results panel via `ui/renderer.rs`

### Where to add new features

| You want to... | Start here |
|----------------|-----------|
| Add a new keybinding | `app.rs handle_key()` |
| Add a new `:` command | `app.rs submit_query()` (see `:explain` for example) |
| Change how the UI looks | `ui/renderer.rs` |
| Add a new UI component | `ui/components/` (create new module, register in `mod.rs`) |
| Support a new database | `db/connection.rs` (add `introspect_*` method) |
| Change AI behavior | `ai/agent.rs` (prompt engineering) or `ai/provider.rs` (new provider) |
| Add persistent state | Follow the pattern in `db/history.rs` (JSON file in data dir) |

## Making changes

1. **Fork the repository** and clone your fork
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
3. **Make your changes.** Write code, add tests, update docs.
4. **Test everything:**
   ```bash
   cargo test && cargo clippy -- -D warnings && cargo fmt --check
   ```
5. **Commit with a conventional message** (see below)
6. **Push and open a PR** against `main`

## Code standards

### Rust style

- Run `cargo fmt` before every commit. No exceptions.
- All `cargo clippy` warnings must be resolved. CI enforces `-D warnings`.
- No `.unwrap()` in production code paths. Use `?`, `.unwrap_or_default()`, or handle the error explicitly. `.unwrap()` in tests is fine.
- No `unsafe` blocks unless absolutely necessary and well-documented.
- No blocking operations in the render path. The TUI event loop must stay responsive.

### Testing

- Every new data structure or utility function should have unit tests.
- Use `#[cfg(test)] mod tests { ... }` at the bottom of the file.
- Integration tests go in the `tests/` directory at the project root.
- Aim for test names that describe behavior: `test_sanitize_identifier_rejects_double_quotes` not `test_sanitize_1`.

### Documentation

- Public functions and structs should have `///` doc comments.
- Non-obvious logic should have inline `//` comments explaining why, not what.
- If you add a new feature, update the README keybindings table or feature list.

### Security

- Never interpolate user input directly into SQL strings. Use parameterized queries or `sanitize_identifier()`.
- AI provider API keys must never be logged or printed.
- HTTP clients must have timeouts set (see `http_client()` in `provider.rs`).

## Commit conventions

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add tab completion for table names
fix: prevent panic when schema has zero tables
docs: add Ollama setup guide to README
test: add integration tests for safe mode
refactor: extract command handlers from app.rs
fix(security): sanitize table names in PRAGMA queries
feat(ui): add SQL syntax highlighting
```

**Types:** `feat`, `fix`, `docs`, `test`, `refactor`, `ci`, `chore`

**Scopes (optional):** `security`, `ui`, `db`, `ai`, `cache`, `config`

Keep the subject line under 72 characters. Use the body for details if needed.

## Pull request process

1. **Link the issue.** If your PR addresses an open issue, include `Closes #N` in the PR description.
2. **Keep PRs focused.** One feature or fix per PR. If you find an unrelated bug while working, open a separate issue.
3. **Describe what changed and why.** The PR description should explain your approach so reviewers can follow along.
4. **All CI checks must pass.** The PR will not be reviewed until tests, clippy, and formatting all pass.
5. **Be responsive to feedback.** The maintainer may request changes. This is normal and constructive.

### What makes a PR easy to review

- Small, focused diffs (under 300 lines is ideal)
- Tests included for new behavior
- Clear commit messages
- Updated documentation if the change affects user-facing behavior

### What will get a PR rejected

- Unrelated changes bundled together
- No tests for new functionality
- Failing CI
- Changes that break existing functionality without discussion

## Getting help

- **Open an issue** with your question. There are no stupid questions.
- **Start a discussion** in the [Issues](https://github.com/rohitgs28/querywise/issues) tab if you want feedback on an approach before coding.
- **Read the code.** The codebase is under 2,000 lines of Rust. You can read the entire thing in an afternoon.

## License

By contributing to QueryWise, you agree that your contributions will be licensed under the [MIT License](LICENSE).

Thank you for helping make QueryWise better.
