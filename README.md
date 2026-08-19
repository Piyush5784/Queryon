# Queryon

A modern desktop PostgreSQL client. Tauri + React + TypeScript frontend, Rust native layer for database connections and credential handling.

See `Docs/Step-1.md` for the product spec, `Docs/Phase-2-Database-Support.md` for the multi-engine roadmap, and `CLAUDE.md` for architecture and development conventions.

## Development

Start local dev databases (Postgres, a Postgres "samples" instance for large test datasets, and MySQL):

```bash
docker compose up -d
```

Connection URLs:

- Postgres (app dev/test fixtures): `postgres://devuser:devpass@localhost:55434/devdb`
- Postgres (large sample datasets): `postgres://devuser:devpass@localhost:55435/samples`
- MySQL: `mysql://devuser:devpass@localhost:33066/devdb`

Run the app:

```bash
npm run tauri dev
```

`npm run dev` (plain Vite) will not have the Tauri `invoke()` bridge available and cannot connect to a database.

## Testing

```bash
cd src-tauri
cargo test --lib              # unit tests
cargo test --test table_operations  # integration tests (needs the dev Postgres running)
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)