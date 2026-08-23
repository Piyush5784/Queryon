# Phase 2 — Multi-Engine Database Support

Queryon is currently a Postgres + MySQL client. This document tracks the target support matrix for
future engines, grouped by how much they actually cost to add given the Rust/Tauri architecture in
`CLAUDE.md` — not by marketing tiers. It exists so "add database X" requests can be evaluated against
real crate maturity and architectural fit instead of decided ad hoc.

## How support is architected today

Every engine implements the `DatabaseDriver` trait (`src-tauri/src/domain/driver.rs`):
`server_version`, `list_tables`, `get_table_columns`, `fetch_table_rows`, `update_json_cell`,
`update_cell_text`, `delete_rows`, `execute_query`. `domain/connection/service.rs::open_pool_and_verify`
is the single place that matches on `Engine` and constructs a driver; everything above that layer
(commands, `ConnectionRegistry`, the frontend) only ever holds an `Arc<dyn DatabaseDriver>` and never
branches on engine.

This trait assumes a **relational, network, schema→table→row→column** model: a client-server
connection over a wire protocol, tables with typed columns, a primary key to target a row for
update/delete. Engines that fit this shape are cheap to add — new engines that don't (document
stores, key-value, wide-column) need real UI work beyond a new driver, not just a new crate.

## Support tiers

**Tier 1 — reuses an existing driver, near-zero new code.** These speak the exact wire protocol
Postgres or MySQL already use. The work is dialect/catalog-query differences in `metadata.rs`, not a
new crate.

| Database | How it connects | What actually needs building |
|---|---|---|
| CockroachDB | `tokio-postgres` (Postgres wire protocol) | `Engine::CockroachDb` variant, a `metadata.rs` branch for its `pg_catalog` extensions |
| GreengageDB | `tokio-postgres` (Greenplum/Postgres fork) | Same as above; less documented than Cockroach, budget a validation spike |
| MariaDB | `sqlx` mysql feature (MySQL wire protocol) | `Engine::MariaDb` variant; watch MariaDB's `JSON` (alias for `LONGTEXT`) and auth-plugin differences |
| TiDB | `sqlx` mysql feature (MySQL wire protocol) | `Engine::TiDb` variant; non-sequential `AUTO_INCREMENT` across nodes affects nothing we rely on today |
| Amazon Redshift | `tokio-postgres` (Postgres wire protocol) | `metadata.rs` branch — Redshift's catalog SQL diverges more than Cockroach's. **No local Docker option**; testing needs a real (billed) cluster |

## Tier 2 — new driver, same relational/grid UI

Real driver work, but the result still fits the existing `DatabaseDriver` trait and DataGrid/SQL
editor UI unmodified.

| Database | Crate | Status | Notes |
|---|---|---|---|
| SQLite | `sqlx` (sqlite feature) | Active | Async-native, consistent with the existing sqlx-mysql code. Embedded — no host/port/credentials, needs a "local file" connection type distinct from `ConnectionProfile`'s network fields |
| SQL Server | `tiberius` | Active (community-maintained) | Pure-Rust TDS, async. No built-in pooling — pair with `deadpool` generically, same pattern as Postgres |
| SAP HANA | `hdbconnect` / `hdbconnect_async` | Active, single maintainer | Pure Rust, no proprietary client libraries needed. Bus-factor risk (one maintainer) is the real caveat, not code maturity |
| ClickHouse | `clickhouse` (official) | Active | HTTP-based today (no native TCP yet) — a typed HTTP client, not a pooled wire connection. Runs locally via Docker. Columnar types (`Array`, `Map`, `LowCardinality`) will stress the DataGrid's cell-rendering assumptions |
| DuckDB | `duckdb-rs` (official) | Active | Embedded/in-process like SQLite, not client-server. Vendored C build adds binary size and build time |
| LibSQL | `libsql` (official, Turso) | Active | SQLite-compatible. "Server" mode means running Turso's `sqld` yourself via Docker — treat as SQLite-adjacent, not a distinct paradigm |

## Tier 3 — new driver, adapter required (not a wire protocol)

These are relational-shaped but speak HTTP/REST instead of a database wire protocol, so `fetch_rows`
becomes "paginate a JSON response" rather than a row stream — real executor-layer work, not just a
new crate behind the same interface.

| Database | Crate | Status | Notes |
|---|---|---|---|
| Google BigQuery | `gcp-bigquery-client` | Active, pre-1.0 (API still shifting) | GCP service-account/OAuth auth. No local emulator with full fidelity — cloud-only testing |
| Snowflake | `snowflake-connector-rs` | Active, unofficial (no Snowflake-published Rust SDK) | REST-based SQL API, key-pair or OAuth auth. Cloud-only, no local Docker option |
| Trino / Presto | `trino-rust-client` | Active (maintained fork of the older `prusto`) | HTTP statement protocol — submit query, poll for result pages, different execution model than a blocking connection. Official Docker image, testable locally |

## Tier 4 — different UI paradigm, not just a new driver

Fundamentally non-relational. Even with a mature crate, these need their own browsing UI (document
tree, key browser, graph view) — plugging them into the existing table grid would misrepresent the
data model. **Scope these as a separate feature, not a `DatabaseDriver` implementation**, when they
come up.

| Database | Crate | Status | Data model | UI needed instead of the grid |
|---|---|---|---|---|
| MongoDB | `mongodb` (official) | Active, mature | Documents, nested/array fields, no fixed schema | JSON/document tree browser |
| Redis | `redis-rs` | Functional, but **governance unsettled** — long-inactive original maintainer, an open naming/ownership dispute with Redis Inc. as of late 2024 | Key-value + data structures (lists, hashes, sets, streams) | Key browser, per-type value viewer |
| DynamoDB | `aws-sdk-dynamodb` (official) | Active | Key-value/document hybrid, partition/sort keys, GSIs/LSIs | Key/index browser. Testable locally via AWS's official DynamoDB Local Docker image |
| Cassandra | `scylla` crate (documented Cassandra-compatible) | Active | Wide-column, CQL, partition/clustering keys, no joins | Partial grid reuse possible for row browsing, but the schema model (partition keys, TTLs, consistency levels) needs real domain modeling, not a drop-in |
| ScyllaDB | Same `scylla` crate | Active — this is effectively ScyllaDB's own driver | Same as Cassandra, protocol-compatible by design | Same UI work as Cassandra covers both |
| SurrealDB | `surrealdb` (official) | Active | Multi-model: document, graph, relational-ish, time-series, vector; own SurrealQL | SurrealQL is SQL-*like* so the editor could be reused, but graph edges/nested records don't map to rows/columns |

## Not currently viable, or viable with a real caveat

| Database | Issue |
|---|---|
| SAP SQL Anywhere | No Rust crate exists at all. Only path is `odbc-api` + SAP's proprietary, licensed ODBC driver — a non-Rust, platform-specific dependency. Not viable without accepting that |
| Oracle Database | Real fork in the road: the mature crate (`oracle`/rust-oracle) wraps ODPI-C and requires distributing Oracle's proprietary Instant Client per platform — a packaging problem for a cross-platform Tauri app that conflicts with "the Rust layer handles everything." A newer pure-Rust crate (`oracle-rs`, no OCI dependency) avoids that but is unproven. Testable locally via Oracle's free "Oracle Database Free" Docker image either way |
| Firebird | No single dominant, actively-maintained crate. `rsfbclient` is the most usable option but modest activity; alternatives are pre-1.0/beta. Expect to debug the driver itself more than with tokio-postgres or sqlx |

## Sequencing recommendation

1. **CockroachDB, MariaDB, TiDB** first — they ride the existing Postgres/MySQL drivers, so this is
   almost entirely `metadata.rs` dialect work plus a docker-compose service and integration tests per
   engine, following the exact pattern MySQL (Milestone B, see `CLAUDE.md` git history) already
   proved out. Cheapest possible next milestone.
2. **SQLite** next — high user demand, Tier 2, but requires the one piece of real new architecture:
   an embedded/file-based connection type alongside the existing network `ConnectionProfile` shape.
   Worth doing once, since DuckDB and LibSQL benefit from the same embedded-connection plumbing
   afterward.
3. **SQL Server** after that — Tier 2, genuinely new driver (`tiberius`), no embedded-connection
   complexity, broad enterprise demand.
4. Everything else (ClickHouse, DuckDB, Trino, cloud warehouses, non-relational stores) is a
   deliberate, scoped decision when there's real demand for it — not a default backlog item, given
   the UI and testing costs above.

## Out of scope for now

Redshift, BigQuery, and Snowflake cannot be integration-tested locally via Docker the way every other
engine in this document can — any work here also means committing to paid cloud test infrastructure.
Tier 4 (document/key-value/wide-column) engines are excluded from `DatabaseDriver` entirely until
there's an explicit decision to build a second browsing UI paradigm alongside the table grid.
