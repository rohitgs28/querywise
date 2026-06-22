# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-21

First public release. QueryWise is an AI-powered terminal database client: ask a
question in plain English, get SQL back, see results. Failed queries are
rewritten and retried automatically.

### Added

- Natural-language to SQL via Anthropic Claude, OpenAI, or local Ollama models.
- Self-healing queries: detect errors, rewrite SQL, retry.
- `:explain` (plain-English breakdown) and `:plan` (execution-plan tree) commands.
- PostgreSQL, MySQL, and SQLite support via sqlx, with schema introspection.
- Four-panel ratatui TUI with SQL syntax highlighting and 7 built-in themes.
- Persistent query history, query bookmarks, and a TTL-aware LRU query cache.
- CSV export and a safe mode that blocks destructive queries.
- Fully offline operation with Ollama.

[0.1.0]: https://github.com/rohitgs28/querywise/releases/tag/v0.1.0
