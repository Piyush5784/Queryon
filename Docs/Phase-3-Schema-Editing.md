# Phase 3 — Schema Read/Write (DDL)

Queryon today edits **rows**: browse, filter, sort, insert, update, delete data in existing tables.
It has no way to change the shape of the database itself — create a table, add a column, add an
index, drop a constraint. This phase adds that: schema introspection deep enough to show real
structure, and DDL execution safe enough to run against someone's actual database.

This is the single largest feature this app has taken on. It is **not one milestone** — see
"Milestones" below for the actual build order. Do not implement past the current milestone in one
pass; that's an explicit project rule (`CLAUDE.md`), and it matters more here than anywhere else
because a DDL bug silently corrupts someone's schema, not just a row.

## What "big companies" (DBeaver, TablePlus, Beekeeper, pgAdmin) actually do here

Surveyed pattern across those four, since that's the explicit reference point for this phase:

1. **They never hand-write DDL from form state and execute it blind.** Every one of them has a
   **"generate DDL, show it, let you review/edit it, then run it"** step for anything beyond a
   trivial single-column add. DBeaver and pgAdmin call this the "SQL preview" / "DDL" tab on their
   table-editor dialogs; TablePlus shows the generated `ALTER TABLE` before committing.
2. **Structural changes are diffed, not applied field-by-field.** You edit a table's columns/indexes
   in a form, and the tool computes *one* diff (added columns, dropped columns, type changes, index
   changes) into a minimal set of `ALTER TABLE` statements — not one statement per keystroke.
3. **Destructive operations get a distinct, harder-to-misfire confirmation** than a normal row
   delete — typing the table name to confirm a `DROP TABLE`, a red/warning-styled dialog, a preview
   of exactly what SQL will run. This app already treats row deletes this way (`AlertDialog` in
   `TableView.tsx`); DDL needs the same pattern turned up a level, since it's harder to undo.
3. **Object tree, not a flat table list.** The sidebar grows a real schema tree: schemas → tables →
   columns/indexes/constraints/triggers, each independently expandable, mirroring how DBeaver/pgAdmin
   structure their navigator. Queryon's `ConnectionTreeItem` already has schema→table grouping; this
   extends one more level down per table.
4. **No transaction-wrapping magic promised.** DBeaver executes generated DDL as plain statements,
   not inside an app-managed transaction with automatic rollback-on-error, because DDL is often
   non-transactional or auto-commits mid-statement depending on the engine (true for MySQL always;
   true for many Postgres DDL forms too, e.g. anything involving `CONCURRENTLY`). Don't promise safety
   this app's actual engines can't provide — show what ran and what failed, don't pretend atomicity
   that isn't there.

## Scope for this phase

**In scope**: tables, columns, indexes, primary keys, foreign keys, unique/check constraints, table
rename/drop, schema browsing of all of the above (read).

**Out of scope for this phase** (call out explicitly so it isn't assumed): triggers, stored
procedures/functions, views (create/alter — read-only listing is fine), sequences as first-class
objects, partitioning, row-level security, extensions. Each of those is its own follow-on scope if
wanted later — don't fold them in silently.

## Architecture fit

Follows the same layering `CLAUDE.md` already mandates — no new pattern invented for this feature:

```
commands/schema.rs        thin IPC layer — new commands, e.g. db_create_table, db_alter_table,
                           db_drop_table, db_create_index, db_get_table_ddl, db_list_indexes,
                           db_list_constraints
domain/schema/
    models.rs              new types: TableDefinition, ColumnDefinition, IndexDefinition,
                            ConstraintDefinition, SchemaDiff, DdlStatement
    service.rs              diff computation (desired vs current TableDefinition → SchemaDiff →
                             Vec<DdlStatement>), calls into DatabaseDriver
domain/driver.rs           DatabaseDriver gains: list_indexes, list_constraints, get_table_ddl,
                            execute_ddl(statements: &[DdlStatement]) -> Result<DdlExecutionResult>
infrastructure/postgres/    metadata.rs: add index/constraint/FK introspection (pg_index,
    ddl.rs (new)            pg_constraint, information_schema.*); ddl.rs: builds Postgres-dialect
                            CREATE/ALTER/DROP strings from DdlStatement
infrastructure/mysql/       same split; MySQL's SHOW CREATE TABLE / information_schema
    ddl.rs (new)            equivalents; note MySQL ALTER TABLE syntax differs meaningfully from
                            Postgres for a lot of these (see "Engine differences" below)
```

`DdlStatement` is the key new concept: an engine-agnostic intermediate representation
(`AddColumn`, `DropColumn`, `AlterColumnType`, `AddIndex`, `DropIndex`, `AddForeignKey`, ...) that
each engine's `ddl.rs` renders into its own SQL dialect. The frontend never sees raw SQL until the
preview step — it works with `SchemaDiff`/`DdlStatement` so the "review generated SQL before running"
step (see pattern #1 above) is a rendering of these, not a separate hand-maintained string.

### Frontend

```
features/schema/                  new feature folder, sibling to tables/query/connections
    components/
        TableDesigner/             the "create/alter table" form — columns list, index list,
                                    constraint list editors
        DdlPreviewDialog.tsx        shows generated SQL for review before execution (pattern #1)
        SchemaTree/                 sidebar extension: indexes/constraints under each table node
    types.ts                        mirrors domain/schema/models.rs's new types (existing
                                     hand-mirroring convention, no ts-rs/specta yet per CLAUDE.md)
    api.ts
```

`ConnectionTreeItem.tsx` gains the deeper tree levels; a table's row in the sidebar gets a "Design
Table" action (context-menu style, consistent with the existing per-row 6-dot menu pattern in
`DataGrid`) that opens `TableDesigner` for editing an existing table, or a toolbar action for
creating a new one.

## Engine differences that affect the design (don't discover these mid-implementation)

- **Column type change**: Postgres supports `ALTER COLUMN ... TYPE ...` (with a `USING` clause for
  non-trivial casts). MySQL requires `MODIFY COLUMN` restating the *entire* column definition
  (type, nullability, default — omit one and it resets). `DdlStatement::AlterColumnType` needs
  enough column context for MySQL's renderer to reconstruct the full definition, not just the diff.
- **Adding a NOT NULL column to a non-empty table**: Postgres requires a `DEFAULT` (or a two-step
  add-nullable/backfill/set-not-null) to avoid failing on existing rows. MySQL fills the engine
  default silently. The diff/preview UI needs to surface this rather than let it fail confusingly at
  execution.
- **Dropping a column that's part of an index/constraint**: Postgres cascades or errors depending on
  the drop type; MySQL auto-drops dependent indexes on that column silently. Preview should warn,
  not just execute.
- **`IF NOT EXISTS`/`IF EXISTS`**: both support it on `CREATE TABLE`/`DROP TABLE`; support is
  inconsistent per-object-type beyond that (e.g. Postgres has `ADD COLUMN IF NOT EXISTS`, MySQL 8
  does not on all forms) — the ddl.rs renderer needs an engine capability check per statement kind,
  not a blanket assumption.
- **Renaming**: Postgres `ALTER TABLE ... RENAME TO`, `ALTER TABLE ... RENAME COLUMN ... TO ...`.
  MySQL 8 supports `RENAME COLUMN` directly (older MySQL needed the `CHANGE` verb restating the full
  definition) — target MySQL 8+ only, consistent with this project's existing `sqlx` MySQL feature
  set, and don't attempt to support pre-8 rename syntax.

## Milestones (build order)

Each milestone is independently shippable and testable. Do not start milestone N+1 before N is done
and both `npm run build` / `cargo check` are clean, per the existing process rule.

1. **Read-only schema depth** — `list_indexes`, `list_constraints`, `get_table_ddl` (the "show me the
   `CREATE TABLE` for this" — genuinely useful on its own, ships first, zero write risk). Sidebar
   tree grows the new levels, read-only.
2. **Column add/drop/rename on an existing table** — the smallest real write surface. `DdlStatement`
   + one-statement-at-a-time execution, no diffing yet since there's only ever one change.
3. **DDL preview dialog** — before milestone 2 ships to "done", the preview-before-execute step
   (pattern #1) needs to exist; retrofit it onto milestone 2's flow rather than shipping without it.
4. **Column type/nullable/default change** — the trickiest single-column edit (engine differences
   above are concentrated here).
5. **Indexes** — create/drop, including unique indexes.
6. **Constraints** — primary key, foreign key, check, unique (distinct from a unique index).
7. **Table create** — brings together everything above into one form (a new table is "many column
   adds + constraints" from the diff engine's point of view, so this should mostly compose out of
   milestones 2–6 rather than being new logic).
8. **Table rename/drop** — small, but last on purpose: by this point the diff/preview/confirm
   machinery already exists and this just needs to plug into it, plus the harder confirmation UX
   (pattern #2) reused from the existing row-delete `AlertDialog` pattern.

Each milestone needs both Postgres and MySQL support before being considered done — don't ship
Postgres-only and backfill MySQL later; the two engines' DDL dialects diverge enough (see above)
that deferring MySQL risks discovering a design problem after the frontend/diff layer is already
built around Postgres-only assumptions.
