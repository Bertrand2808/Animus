# Architecture

Animus is a local-first role-play chat application. The backend owns domain
logic, persistence, prompt construction, and Ollama communication. The frontend
owns the interactive persona and chat experience. Project specs and evolving
implementation plans live in `docs/`.

## System Overview

```text
React UI
  |
  | HTTP / SSE
  v
Axum API server
  |
  | repositories
  v
SQLite database

Axum API server
  |
  | Ollama HTTP API
  v
Local Ollama runtime
```

All user data is intended to stay on the local machine. SQLite stores personas,
conversations, messages, and summaries. Local files are the planned storage for
persona assets and deletion backups.

## Workspace Layout

- `crates/animus-core`: shared domain types and Character Card parsing.
- `crates/animus-db`: SQL migrations and repository layer.
- `crates/animus-llm`: prompt construction and Ollama client.
- `crates/animus-server`: Axum routes, app state, and server startup.
- `frontend`: React/TypeScript application.
- `docs`: specs, roadmap, API collections, and design notes that change as the
  product evolves.

## Backend Boundaries

The backend is split into small Rust crates:

- `animus-core` should remain free of I/O and framework details.
- `animus-db` owns SQLite access and schema migrations.
- `animus-llm` owns prompt assembly and Ollama request/response handling.
- `animus-server` wires repositories, routes, app state, and HTTP behavior.

API handlers should stay thin: validate input, call repositories or LLM services,
map errors, and return DTOs.

## Frontend Boundaries

The frontend is a Vite React app. It talks to the backend through `frontend/src/lib/api.ts`
and keeps API shapes in `frontend/src/types/api.ts`.

Page components own screen-level flows. Shared persona form controls live under
`frontend/src/components/persona-form/`. Chat streaming behavior is isolated in
`frontend/src/hooks/useStreamingMessage.ts`.

## Data Model

The stable core model is:

- persona: character definition and model-generation settings
- conversation: chat session for a persona
- message: user, assistant, or system content in a conversation
- summary: compressed long-session memory
- settings: planned global local app settings

The project direction is to make Animus' structured persona model the source of
truth, while keeping Character Card v2 as an import/export compatibility format.

## Prompt and Generation Flow

For normal chat:

1. The frontend posts a user message to the conversation endpoint.
2. The server persists the user message.
3. The server loads persona, recent history, and optional summary.
4. `animus-llm` builds the prompt/messages for Ollama.
5. The server streams Ollama tokens back over SSE.
6. The full assistant response is persisted when streaming completes.

Prompt-building details are product-sensitive and expected to evolve. Keep the
latest prompt-driving spec in `docs/`, not in this file.

## Long-Lived Docs Policy

This file should describe boundaries and data flow that rarely change. It should
not track every feature decision or prompt tweak.

Use `docs/` for evolving specs, including:

- character-driving roadmap
- prompt formats
- DB migration plans
- API additions
- UI workflow decisions
- lorebook and memory design

Current detailed spec: `docs/design-character-driving.md`.
