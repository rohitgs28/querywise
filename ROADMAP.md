# QueryWise Roadmap

## Vision
Make QueryWise the best terminal database client. AI-native features no competitor has, plus all the table-stakes features developers expect.

## Audit Findings (March 2026)

### Security
- [ ] SQL injection risk in schema introspection (format! with table names)
- [ ] No timeout on AI API requests (hung server blocks TUI)
- [ ] No retry logic for AI calls
- [ ] No input validation layer

### Architecture  
- [ ] app.rs is 23KB monolith — needs decomposition
- [ ] No structured error types (all anyhow)
- [ ] No logging/tracing
- [ ] No connection pooling
- [ ] Empty ui/components module

## Priority Queue

### Phase 1: Security & Stability
1. Sanitize table name interpolation in schema queries
2. Add 30s timeout + 5s connect timeout to AI providers
3. Add retry with exponential backoff (3 attempts, 1s/2s/4s)
4. Add structured error types (QueryWiseError enum)
5. Add tracing for observability

### Phase 2: Architecture
6. Decompose app.rs into command modules (query, explain, export)
7. Connection pooling with health checks
8. Input validation layer
9. Extract reusable UI components from renderer.rs
10. Integration tests with in-memory SQLite

### Phase 3: Star-Worthy Features
11. SQL syntax highlighting (biggest visual impact)
12. Tab completion for table/column names
13. Multi-line query editor (Shift+Enter)
14. Clipboard copy (Ctrl+Y)
15. Query bookmarks (Ctrl+B)
16. Saved connections manager
17. Configurable color themes
18. Query plan visualization (:plan)
19. Streaming results for large datasets
20. Vim keybindings mode

### Phase 4: Launch
21. Demo GIF in README
22. Publish to crates.io
23. Homebrew formula
24. Show HN post
25. Blog post on dev.to

## Competitive Edge

QueryWise is the only TUI database client with:
- Natural language to SQL
- Self-healing queries (auto-fix on error)
- AI query explanation
- Local LLM support (Ollama)
- Safe mode (blocks destructive queries)

Once we add syntax highlighting, tab completion, and multi-line editing, we're strictly better than pgcli, usql, and dbcli.
