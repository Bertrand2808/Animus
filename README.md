# Animus

Animus is a local role-play chat tool built around structured character cards,
Ollama, and a local SQLite database. The project goal is to provide a comfy
single-user experience for creating characters, chatting with them, iterating on
their early replies, and keeping all data on the user's machine.

## Current Stack

- Backend: Rust workspace with Axum, SQLx, SQLite, and Ollama integration.
- Frontend: React, TypeScript, Vite, Tailwind CSS.
- LLM runtime: Ollama via `/api/chat` streaming and `/api/generate`.
- Storage: SQLite database plus planned local asset/backup directories.

## Workspace Layout

- `crates/animus-core`: shared domain types such as personas, messages, content
  ratings, and Character Card v2 parsing.
- `crates/animus-db`: SQLite repositories and migrations.
- `crates/animus-llm`: prompt construction and Ollama client.
- `crates/animus-server`: Axum API server.
- `frontend`: React application.
- `docs`: project notes, API collections, roadmap, and design documents.

## Local Development

Prerequisites:

- Rust toolchain
- Bun
- Ollama running locally
- A local SQLite database URL

Start the backend:

```sh
DATABASE_URL=sqlite://./animus.db OLLAMA_URL=http://localhost:11434 OLLAMA_MODEL=llama3.1:8b cargo run -p animus-server
```

Start the frontend:

```sh
cd frontend
bun install
bun run dev
```

Useful checks:

```sh
cargo test
cd frontend && bun run build
cd frontend && bun run lint
```

## Current Capabilities

- Create, edit, import, list, and delete personas.
- Import Character Card v2 JSON into the current flat persona model.
- Start a conversation from a persona first message.
- Stream assistant replies from Ollama.
- Store conversations, messages, and summaries in SQLite.
- Display a summary drawer for existing conversation summaries.
- Store avatar/background values today as data URLs in the database.

## Product Direction

The next major direction is stronger character driving. Animus should move from
simple persona text fields toward a structured local character card with model
instructions, speech/style fields, response constraints, sampling parameters,
redo/edit tools, local assets, automatic backups, and eventually keyword-driven
lorebook injection.

See [docs/design-character-driving.md](docs/design-character-driving.md) for the
detailed roadmap and GitHub issue breakdown.
