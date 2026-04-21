# Architecture Map

**Rust + React local roleplay app (Animus). Single binary, Cargo workspace.**

---

## Directory Structure

```
Animus/
├── Cargo.toml                          # workspace (resolver = "2")
├── data/animus.db                      # SQLite database
├── crates/
│   ├── animus-core/src/
│   │   ├── lib.rs                      # re-exports all domain types
│   │   ├── character_card.rs           # CharacterCardV2 parsing
│   │   ├── content_rating.rs           # ContentRating enum (pg/mature/nsfw)
│   │   └── persona.rs                  # Persona domain struct
│   ├── animus-db/
│   │   ├── migrations/
│   │   │   ├── 001_initial.sql         # personas, conversations, messages, summaries
│   │   │   └── 002_personas_unique_name.sql
│   │   └── src/
│   │       ├── lib.rs                  # Db pool init + run_migrations
│   │       ├── persona_repo.rs         # CRUD for personas
│   │       └── summary_repo.rs         # Summary insert/fetch
│   ├── animus-llm/src/
│   │   └── lib.rs                      # OllamaClient + build_prompt (pure fn)
│   └── animus-server/src/
│       ├── main.rs                     # Axum entry point, tracing, router
│       ├── state.rs                    # AppState (db pool, config)
│       ├── error.rs                    # AppError → IntoResponse
│       └── routes/
│           ├── mod.rs                  # router assembly
│           └── personas.rs             # /api/personas handlers
└── docs/technical/technical_plan.md   # authoritative spec
```

## Key File Locations

- **Entry point**: `crates/animus-server/src/main.rs`
- **Domain types**: `crates/animus-core/src/lib.rs`
- **DB schema**: `crates/animus-db/migrations/001_initial.sql`
- **Prompt logic**: `crates/animus-llm/src/lib.rs` (`build_prompt`)
- **Persona API**: `crates/animus-server/src/routes/personas.rs`
- **Config**: `.env` (DB path, Ollama URL, DEV flag)

## Dependency Graph

```
animus-server → animus-db, animus-llm, animus-core
animus-db     → animus-core
animus-llm    → animus-core
animus-core   → (no internal deps)
```

## API Endpoints (implemented)

```
POST   /api/personas/import
GET    /api/personas
GET    /api/personas/:id
DELETE /api/personas/:id
```

## Pending (per technical_plan.md)

- `animus-llm`: `build_prompt` + `OllamaClient::complete()` + streaming
- Conversation routes + SSE handler
- Summary trigger (`evaluate_summary_trigger`)
- Frontend (Vite + React + shadcn/ui)
- Config API + ServeDir prod

## Dev Setup

```bash
cargo run -p animus-server   # :3000, proxies /* → Vite :5173 if DEV=1
cargo test --all
cargo clippy --all-targets -- -D warnings
```

---

**Last Updated**: 2026-04-20
