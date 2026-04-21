# Quick Start Commands

**Essential commands for this project**

---

## Development

```bash
# Start development server
cargo run -p animus-server

# Run tests
cargo test --all
# Run coverage
cargo llvm-cov
```

## Database (if applicable)

See in `data` folder.

## Common Workflows

1. **Starting work**: `cargo run -p animus-server` (set `DEV=1` to proxy Vite)
2. **Running tests**: `cargo test --all`
3. **Lint**: `cargo clippy --all-targets -- -D warnings`
4. **Prod build**: `npm run build` → `cargo build --release`

## DB

- File: `data/animus.db`
- Migrations: `crates/animus-db/migrations/`
- Auto-run on server start via `sqlx::migrate!()`

---

**Last Updated**: 2026-04-20
