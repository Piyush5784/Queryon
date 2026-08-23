# Queryon Desktop

A desktop database client supporting Postgres, MySQL, SQL Server, SQLite, DuckDB, ClickHouse, StarRocks, and several Postgres/MySQL-compatible engines (Neon, CockroachDB, GreengageDB, MariaDB, TiDB). Tauri + React + TypeScript frontend, Rust native layer for every database and credential operation.

See `Docs/Phase-1-init.md` for the product spec, `Docs/Phase-2-Database-Support.md` for the multi-engine roadmap and per-engine feasibility notes, and `Docs/Phase-3-Schema-Editing.md` for the schema-editing feature scope.

This app is part of a pnpm workspace — run commands from this directory (`apps/desktop/`), or use `pnpm --filter queryon <script>` from the repo root.

## Development

Start local dev databases:

```bash
docker compose up -d
```

See `docker-compose.yml` for the full list of engine containers and their connection URLs — each engine has its own service, brought up independently (bringing every container up at once will strain most machines; see each `scripts/seed-*.sh` for the one-at-a-time workflow this app was built against).

Run the app:

```bash
pnpm tauri dev
```

`pnpm dev` (plain Vite) will not have the Tauri `invoke()` bridge available and cannot connect to a database.

## Testing

```bash
cd src-tauri
cargo test --lib                        # unit tests
cargo test --test <engine>_metadata      # integration tests against a live/embedded database
```

Most integration tests need their engine's dev container running first (see `docker-compose.yml`); SQLite and DuckDB are embedded and need no container.

## Building

```bash
pnpm tauri build
```

DuckDB links against a downloaded prebuilt binary (`DUCKDB_DOWNLOAD_LIB=1`, set automatically via `src-tauri/.cargo/config.toml`) rather than compiling DuckDB's C++ engine from source.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
