// Full-feature, watchable Postgres/SQL-engine walkthrough.
// Mocks the Tauri IPC bridge (window.__TAURI_INTERNALS__.invoke) so the real
// React app runs against fake-but-realistic data — no live Rust backend
// needed for most flows. Pagination uses REAL data queried from the actual
// queryon-pg container (see tests/fixtures/seed-pagination-table.sh and
// tests/fixtures/pagination-rows.json) so paging through 5,000 rows is
// genuine, not simulated.
//
// Usage:
//   1. In apps/desktop: pnpm dev          (Vite dev server on :1420)
//   2. (optional, for real pagination data) ./tests/fixtures/seed-pagination-table.sh
//   3. node tests/postgres.spec.mjs

import { chromium } from "playwright"
import { readFileSync } from "node:fs"
import { fileURLToPath } from "node:url"
import { dirname, join } from "node:path"

const __dirname = dirname(fileURLToPath(import.meta.url))

const CONNECTION_ID = "mock-pg-1"

const SAVED_CONNECTION = {
  id: CONNECTION_ID,
  name: "GUI Test Postgres",
  engine: "postgres",
  host: "localhost",
  port: 55434,
  database: "devdb",
  user: "devuser",
  password: "devpass",
  readOnly: false,
  sslMode: "disable",
  sshTunnel: null,
}

const TABLES = [
  { schema: "public", name: "users", kind: "table", estimatedRows: 3 },
  { schema: "public", name: "orders", kind: "table", estimatedRows: 2 },
  { schema: "public", name: "pagination_test", kind: "table", estimatedRows: 5000 },
]

const COLUMNS_BY_TABLE = {
  users: [
    { name: "id", dataType: "int4", isNullable: false, default: null, isPrimaryKey: true, ordinalPosition: 1 },
    { name: "email", dataType: "text", isNullable: false, default: null, isPrimaryKey: false, ordinalPosition: 2 },
    { name: "plan", dataType: "text", isNullable: true, default: "'free'", isPrimaryKey: false, ordinalPosition: 3 },
  ],
  pagination_test: [
    { name: "id", dataType: "int4", isNullable: false, default: null, isPrimaryKey: true, ordinalPosition: 1 },
    { name: "email", dataType: "text", isNullable: false, default: null, isPrimaryKey: false, ordinalPosition: 2 },
    { name: "full_name", dataType: "text", isNullable: false, default: null, isPrimaryKey: false, ordinalPosition: 3 },
    { name: "plan", dataType: "text", isNullable: false, default: null, isPrimaryKey: false, ordinalPosition: 4 },
    { name: "created_at", dataType: "date", isNullable: false, default: null, isPrimaryKey: false, ordinalPosition: 5 },
  ],
}

const ROWS_BY_TABLE = {
  users: {
    columns: ["id", "email", "plan"],
    rows: [
      [1, "alice@example.com", "pro"],
      [2, "bob@example.com", "free"],
      [3, "carol@example.com", "pro"],
    ],
  },
}

// Real data queried from the live queryon-pg container (see seed script).
// Falls back to a small generated set if the fixture hasn't been produced
// yet, so this spec still runs without requiring Docker.
let paginationRows
try {
  const raw = readFileSync(join(__dirname, "fixtures", "pagination-rows.json"), "utf8")
  paginationRows = JSON.parse(raw)
  console.log(`Loaded ${paginationRows.length} REAL rows from tests/fixtures/pagination-rows.json`)
} catch {
  paginationRows = Array.from({ length: 5000 }, (_, i) => [
    i + 1,
    `user${i + 1}@example.com`,
    `User ${i + 1}`,
    i % 2 === 0 ? "pro" : "free",
    "2026-01-01",
  ])
  console.log("Fixture not found — using generated fallback data for pagination_test (run tests/fixtures/seed-pagination-table.sh for real data)")
}
ROWS_BY_TABLE.pagination_test = { columns: ["id", "email", "full_name", "plan", "created_at"], rows: paginationRows }

function applyFilterSortToRows(data, filters, sort) {
  let rows = data.rows
  const colIndex = (name) => data.columns.indexOf(name)

  for (const f of filters ?? []) {
    const idx = colIndex(f.column)
    if (idx === -1) continue
    rows = rows.filter((row) => {
      const cell = row[idx]
      switch (f.operator) {
        case "equals":
          return String(cell) === f.value
        case "not-equals":
          return String(cell) !== f.value
        case "like":
        case "ilike":
          return String(cell).toLowerCase().includes(String(f.value ?? "").replace(/%/g, "").toLowerCase())
        case "is-null":
          return cell === null
        case "is-not-null":
          return cell !== null
        default:
          return true
      }
    })
  }

  for (const s of [...(sort ?? [])].reverse()) {
    const idx = colIndex(s.column)
    if (idx === -1) continue
    rows = [...rows].sort((a, b) => {
      const av = a[idx]
      const bv = b[idx]
      const cmp = av < bv ? -1 : av > bv ? 1 : 0
      return s.direction === "desc" ? -cmp : cmp
    })
  }

  return { columns: data.columns, rows }
}

async function main() {
  const browser = await chromium.launch({ headless: false, args: ["--start-maximized"] })
  const context = await browser.newContext({
    viewport: process.env.PLAYWRIGHT_HEADLESS_DEBUG ? { width: 1600, height: 1000 } : null,
    permissions: ["clipboard-read", "clipboard-write"],
  })
  const page = await context.newPage()

  page.on("console", (m) => {
    if (m.type() === "error") console.log(`[browser console error] ${m.text()}`)
  })
  page.on("pageerror", (e) => console.log(`[browser pageerror] ${e.message}`))

  await page.exposeFunction("__logStep", (msg) => console.log(`… ${msg}`))

  await page.addInitScript(
    ({ connectionId, savedConnectionSeed, tables, columnsByTable, rowsByTable }) => {
      // Mutable state so this behaves like a real (if fake) backend across
      // the whole run: renamed/deleted connections, inserted/deleted rows,
      // staged DDL, etc. all persist for the session.
      const state = {
        savedConnections: [{ ...savedConnectionSeed }],
        connected: new Set(),
        tables: JSON.parse(JSON.stringify(tables)),
        columnsByTable: JSON.parse(JSON.stringify(columnsByTable)),
        rowsByTable: rowsByTable, // left as-is (large), mutated in place per-table
        indexesByTable: {},
        constraintsByTable: {},
        eventListeners: {},
        nextRowId: 100000,
        transactions: new Set(),
        savedQueries: [],
        queryHistory: [],
      }
      window.__QUERYON_TEST_STATE__ = state

      function applyFilterSortToRows(data, filters, sort) {
        let rows = data.rows
        const colIndex = (name) => data.columns.indexOf(name)
        for (const f of filters ?? []) {
          const idx = colIndex(f.column)
          if (idx === -1) continue
          rows = rows.filter((row) => {
            const cell = row[idx]
            switch (f.operator) {
              case "equals":
                return String(cell) === f.value
              case "not-equals":
                return String(cell) !== f.value
              case "like":
              case "ilike":
                return String(cell)
                  .toLowerCase()
                  .includes(String(f.value ?? "").replace(/%/g, "").toLowerCase())
              case "is-null":
                return cell === null
              case "is-not-null":
                return cell !== null
              default:
                return true
            }
          })
        }
        for (const s of [...(sort ?? [])].reverse()) {
          const idx = colIndex(s.column)
          if (idx === -1) continue
          rows = [...rows].sort((a, b) => {
            const av = a[idx]
            const bv = b[idx]
            const cmp = av < bv ? -1 : av > bv ? 1 : 0
            return s.direction === "desc" ? -cmp : cmp
          })
        }
        return { columns: data.columns, rows }
      }

      function encodeRows(rows) {
        return rows.map((row) => row.map((cell) => JSON.stringify(cell)))
      }

      function decodeJsonMap(map) {
        const out = {}
        for (const [k, v] of Object.entries(map ?? {})) {
          try {
            out[k] = JSON.parse(v)
          } catch {
            out[k] = v
          }
        }
        return out
      }

      function emit(eventName, payload) {
        for (const listenerId of state.eventListeners[eventName] ?? []) {
          const cb = state.callbacksById.get(listenerId)
          if (cb) cb({ event: eventName, payload, id: listenerId })
        }
      }

      function handle(cmd, args) {
        switch (cmd) {
          // ---- connections ----
          case "db_list_saved_connections":
            return state.savedConnections
          case "db_list_active_connections":
            return [...state.connected]
          case "db_test_connection":
            return { id: args.profile.id, serverVersion: "PostgreSQL 16.0 (mock)" }
          case "db_connect":
          case "db_connect_saved": {
            const id = args.connectionId ?? args.profile?.id ?? connectionId
            state.connected.add(id)
            return { id, serverVersion: "PostgreSQL 16.0 (mock)" }
          }
          case "db_disconnect":
            state.connected.delete(args.connectionId)
            return null
          case "db_save_connection": {
            const exists = state.savedConnections.some((c) => c.id === args.profile.id)
            if (!exists) state.savedConnections.push(args.profile)
            return null
          }
          case "db_delete_saved_connection":
            state.savedConnections = state.savedConnections.filter((c) => c.id !== args.connectionId)
            state.connected.delete(args.connectionId)
            return null
          case "db_rename_saved_connection": {
            const conn = state.savedConnections.find((c) => c.id === args.connectionId)
            if (conn) conn.name = args.name
            return null
          }

          // ---- schema browsing ----
          case "db_list_tables":
            return state.tables
          case "db_get_table_columns":
            return state.columnsByTable[args.table] ?? []
          case "db_list_indexes":
            return state.indexesByTable[args.table] ?? []
          case "db_list_constraints":
            return state.constraintsByTable[args.table] ?? []
          case "db_get_table_ddl":
            return `CREATE TABLE ${args.schema}.${args.table} (\n  id int4 PRIMARY KEY,\n  email text NOT NULL,\n  plan text DEFAULT 'free'\n);`
          case "db_render_ddl":
            return (args.statements ?? []).map((s) => ({ sql: `-- mock DDL for ${s.op}\n${JSON.stringify(s)}` }))
          case "db_execute_ddl": {
            const results = (args.statements ?? []).map((s) => {
              if (s.op === "addColumn") {
                const cols = state.columnsByTable[s.table] ?? []
                cols.push({
                  name: s.column.name,
                  dataType: s.column.dataType,
                  isNullable: s.column.isNullable ?? true,
                  default: s.column.default ?? null,
                  isPrimaryKey: false,
                  ordinalPosition: cols.length + 1,
                })
                state.columnsByTable[s.table] = cols
              } else if (s.op === "dropColumn") {
                state.columnsByTable[s.table] = (state.columnsByTable[s.table] ?? []).filter(
                  (c) => c.name !== s.column
                )
              } else if (s.op === "createTable") {
                state.tables.push({ schema: args.schema, name: s.table, kind: "table", estimatedRows: 0 })
                state.columnsByTable[s.table] = s.columns.map((c, i) => ({
                  name: c.name,
                  dataType: c.dataType,
                  isNullable: c.isNullable ?? true,
                  default: c.default ?? null,
                  isPrimaryKey: false,
                  ordinalPosition: i + 1,
                }))
                state.rowsByTable[s.table] = { columns: s.columns.map((c) => c.name), rows: [] }
              } else if (s.op === "dropTable") {
                state.tables = state.tables.filter((t) => t.name !== s.table)
              } else if (s.op === "renameTable") {
                const t = state.tables.find((t) => t.name === s.table)
                if (t) t.name = s.newName
              }
              return { sql: `mock ${s.op}`, success: true, error: null }
            })
            return { results, rolledBack: false }
          }

          // ---- table data ----
          case "db_fetch_table_rows": {
            const data = state.rowsByTable[args.table] ?? { columns: [], rows: [] }
            const filtered = applyFilterSortToRows(data, args.filters, args.sort)
            const limit = args.limit
            const offset = args.offset
            const page = filtered.rows.slice(offset, offset + limit)
            return {
              columns: filtered.columns,
              rows: encodeRows(page),
              rowCount: page.length,
              hasMore: offset + limit < filtered.rows.length,
              durationMs: 2,
            }
          }
          case "db_count_table_rows": {
            const data = state.rowsByTable[args.table] ?? { columns: [], rows: [] }
            return applyFilterSortToRows(data, args.filters, []).rows.length
          }
          case "db_insert_row": {
            const data = state.rowsByTable[args.table]
            const values = decodeJsonMap(args.values)
            const row = data.columns.map((c) => (c === "id" ? state.nextRowId++ : values[c] ?? null))
            data.rows.unshift(row)
            return null
          }
          case "db_update_cell_text": {
            const data = state.rowsByTable[args.table]
            const rowObj = decodeJsonMap(args.row)
            const idIdx = data.columns.indexOf("id")
            const target = data.rows.find((r) => r[idIdx] === rowObj.id)
            if (target) {
              const colIdx = data.columns.indexOf(args.column)
              target[colIdx] = args.value
            }
            return null
          }
          case "db_delete_rows": {
            const data = state.rowsByTable[args.table]
            const idIdx = data.columns.indexOf("id")
            const idsToDelete = new Set(args.rows.map((r) => decodeJsonMap(r).id))
            const before = data.rows.length
            data.rows = data.rows.filter((r) => !idsToDelete.has(r[idIdx]))
            return before - data.rows.length
          }

          // ---- SQL query execution ----
          case "db_execute_query": {
            const sql = String(args.sql ?? "").trim().toLowerCase()
            if (sql.startsWith("select")) {
              const data = state.rowsByTable.users
              return {
                kind: "rows",
                columns: data.columns,
                rows: encodeRows(data.rows),
                totalRowCount: data.rows.length,
                durationMs: 4,
              }
            }
            if (sql.startsWith("insert")) {
              state.rowsByTable.users.rows.push([99, "queryinsert@example.com", "free"])
              return { kind: "affected", rowCount: 1, durationMs: 3 }
            }
            if (sql.startsWith("update")) {
              return { kind: "affected", rowCount: 1, durationMs: 3 }
            }
            if (sql.startsWith("delete")) {
              return { kind: "affected", rowCount: 1, durationMs: 3 }
            }
            return { kind: "affected", rowCount: 0, durationMs: 1 }
          }
          case "db_fetch_query_result_page": {
            const data = state.rowsByTable.users
            const page = data.rows.slice(args.offset, args.offset + args.limit)
            return { columns: data.columns, rows: encodeRows(page), totalRowCount: data.rows.length }
          }
          case "db_clear_query_result_cache":
            return null
          case "db_cancel_query":
            return null
          case "db_begin_transaction":
            state.transactions.add(args.tabId)
            return null
          case "db_commit_transaction":
          case "db_rollback_transaction":
            state.transactions.delete(args.tabId)
            return null
          case "db_has_active_transaction":
            return state.transactions.has(args.tabId)
          case "db_save_query":
            state.savedQueries.push(args.query)
            return null
          case "db_list_saved_queries":
            return state.savedQueries
          case "db_delete_saved_query":
            state.savedQueries = state.savedQueries.filter((q) => q.id !== args.queryId)
            return null
          case "db_list_query_history":
            return state.queryHistory
          case "db_clear_query_history":
            state.queryHistory = []
            return null

          // ---- export ----
          case "export_default_directory":
            return "/home/mockuser/Downloads"
          case "export_pick_directory":
            return args.initialDirectory || "/home/mockuser/Downloads"
          case "export_run_table":
          case "export_run_rows":
          case "export_run_query": {
            const jobId = args.jobId
            const total = args.request?.rows?.length ?? 3
            setTimeout(() => {
              emit("export:progress", { jobId, kind: "progress", rowsWritten: total, totalRows: total })
              emit("export:done", {
                jobId,
                kind: "done",
                rowsWritten: total,
                path: `${args.request.directory}/${args.request.fileName}`,
              })
            }, 50)
            return null
          }
          case "export_cancel":
            return undefined

          // ---- misc plumbing ----
          case "plugin:updater|check":
            return null
          case "plugin:event|listen": {
            // `handler` is a numeric callback id already registered via our
            // own transformCallback (listen() calls transformCallback(cb)
            // itself before invoking this command) — just index it by event
            // name so emit() can find and call it later.
            const eventName = args.event
            state.eventListeners[eventName] = state.eventListeners[eventName] ?? []
            state.eventListeners[eventName].push(args.handler)
            return args.handler
          }
          case "plugin:event|unlisten":
            return null
          default:
            console.error("UNMOCKED TAURI COMMAND:", cmd, JSON.stringify(args))
            return null
        }
      }

      // Tauri's real `listen(event, cb)` calls `transformCallback(cb)` ITSELF
      // (getting back a numeric id) before invoking `plugin:event|listen`
      // with that id as `handler`. So our transformCallback is the single
      // source of truth for id -> real callback function; `handle()` above
      // just indexes those ids by event name so `emit()` can call them back.
      state.callbacksById = new Map()
      let nextCallbackId = 1

      window.__TAURI_INTERNALS__ = {
        invoke: (cmd, args) =>
          new Promise((resolve, reject) => {
            try {
              resolve(handle(cmd, args ?? {}))
            } catch (err) {
              reject(String(err?.message ?? err))
            }
          }),
        transformCallback: (cb) => {
          const id = nextCallbackId++
          state.callbacksById.set(id, cb)
          return id
        },
        convertFileSrc: (p) => p,
      }

      // Real @tauri-apps/api event.js calls this directly on unlisten();
      // stub it so cleanup doesn't throw in the mocked environment.
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: () => {},
      }
    },
    {
      connectionId: CONNECTION_ID,
      savedConnectionSeed: SAVED_CONNECTION,
      tables: TABLES,
      columnsByTable: COLUMNS_BY_TABLE,
      rowsByTable: ROWS_BY_TABLE,
    }
  )

  const log = (msg) => console.log(`… ${msg}`)

  // ---------------------------------------------------------------------
  // 1. New Connection: open dialog, fill details, Test Connection, Connect
  // ---------------------------------------------------------------------
  log("Loading the app")
  await page.goto("http://localhost:1420", { waitUntil: "networkidle", timeout: 20000 })

  log("Opening New Connection dialog")
  await page.getByRole("button", { name: "New Connection" }).first().click()
  await page.getByRole("heading", { name: "New Connection" }).waitFor({ state: "visible" })

  log("Selecting PostgreSQL engine")
  await page.getByText("PostgreSQL", { exact: true }).click()

  log("Filling in connection details")
  await page.locator("#conn-name").fill("New GUI Postgres")
  await page.locator("#conn-host").fill("localhost")
  await page.locator("#conn-port").fill("55434")
  await page.locator("#conn-database").fill("devdb")
  await page.locator("#conn-user").fill("devuser")
  await page.locator("#conn-password").fill("devpass")

  log("Clicking Test Connection")
  const testConnBtn = page.getByRole("button", { name: "Test Connection" })
  await testConnBtn.scrollIntoViewIfNeeded()
  await testConnBtn.click()
  const testOk = await page
    .getByText("PostgreSQL 16.0", { exact: false })
    .waitFor({ state: "visible", timeout: 5000 })
    .then(() => true)
    .catch(() => false)
  console.log(`Test Connection succeeded: ${testOk ? "YES" : "NO"}`)

  log("Clicking Connect")
  await page.getByRole("button", { name: "Connect", exact: true }).click()
  const connected = await page
    .getByText("New GUI Postgres", { exact: false })
    .first()
    .waitFor({ state: "visible", timeout: 8000 })
    .then(() => true)
    .catch(() => false)
  console.log(`Connected via New Connection flow: ${connected ? "YES" : "NO"}`)

  // ---------------------------------------------------------------------
  // 2. Use the pre-seeded "GUI Test Postgres" connection for the rest,
  //    since it's already wired with our full mock dataset.
  // ---------------------------------------------------------------------
  log("Opening the pre-seeded 'GUI Test Postgres' connection")
  await page.getByRole("main").getByText("GUI Test Postgres").click()
  await page.getByText("users", { exact: true }).first().waitFor({ state: "visible", timeout: 10000 })

  log("Opening the 'users' table")
  await page.getByText("users", { exact: true }).first().click()
  await page
    .getByText("3 / 3 rows", { exact: false })
    .waitFor({ state: "visible", timeout: 8000 })
    .then(() => console.log("Table rows loaded: YES"))
    .catch(() => console.log("Table rows loaded: NO"))

  // ---------------------------------------------------------------------
  // 3. Filters, sort, column visibility
  // ---------------------------------------------------------------------
  log("Opening Filters panel and applying one filter")
  await page.getByRole("button", { name: "Filters" }).click()
  await page.locator('[role="combobox"]').first().click()
  await page.getByRole("option", { name: "plan" }).click()
  await page.locator('input[placeholder="Enter Value"]').fill("pro")
  await page.getByTitle("Apply filters").click()
  const filterApplied = await page
    .getByText("2 / 2 rows", { exact: false })
    .waitFor({ state: "visible", timeout: 5000 })
    .then(() => true)
    .catch(() => false)
  console.log(`Filter applied correctly (2 pro users): ${filterApplied ? "YES" : "NO"}`)

  log("Clearing filters")
  await page.getByRole("button", { name: "Clear filters" }).click()

  log("Opening Sort panel and applying one sort")
  await page.getByRole("button", { name: "Sort" }).click()
  await page.locator('[role="combobox"]').first().click()
  await page.getByRole("option", { name: "email" }).click()
  await page.getByTitle("Apply sort").click()
  await page.waitForTimeout(300)
  console.log("Sort applied (visual check needed on canvas grid — see screenshot)")

  log("Clearing sort")
  await page.getByRole("button", { name: "Clear sort" }).click()

  log("Opening Columns panel and hiding one column")
  await page.getByRole("button", { name: "Columns" }).click()
  const columnsPopover = page.locator('[role="dialog"], [data-slot="popover-content"]').last()
  await columnsPopover.getByText("plan", { exact: true }).click()
  await page.keyboard.press("Escape")
  log("Restoring hidden column via Show all")
  await page.getByRole("button", { name: "Columns" }).click()
  await page.getByRole("button", { name: "Show all" }).click()
  await page.keyboard.press("Escape")

  // ---------------------------------------------------------------------
  // 4. Row CRUD via the UI
  // ---------------------------------------------------------------------
  log("Adding a row")
  await page.getByRole("button", { name: "Add Row" }).click()
  const addingBarVisible = await page
    .getByText("Adding new row", { exact: false })
    .isVisible()
    .catch(() => false)
  console.log(`Add-row edit bar shown: ${addingBarVisible ? "YES" : "NO"}`)
  log("Discarding the new row (canvas cell entry not exercised — see notes)")
  await page.getByRole("button", { name: "Discard" }).click()

  log("Refreshing the table")
  await page.getByRole("button", { name: "Refresh" }).click()

  // ---------------------------------------------------------------------
  // 5. Copy / Download / Refresh
  // ---------------------------------------------------------------------
  log("Opening Copy dropdown and copying the page")
  await page.getByRole("button", { name: "Copy" }).click()
  await page.getByText("Copy Page", { exact: false }).click()
  const copiedLabel = await page
    .getByRole("button", { name: "Copied" })
    .isVisible()
    .catch(() => false)
  console.log(`Copy Page worked: ${copiedLabel ? "YES" : "NO"}`)

  log("Opening Download dialog")
  await page.getByRole("button", { name: "Download" }).click()
  await page.getByText("Export users", { exact: false }).waitFor({ state: "visible", timeout: 5000 })

  log("Expanding Advanced Options to see chunk size")
  await page.getByText("Advanced Options", { exact: false }).click()
  const chunkVisible = await page.locator("#export-chunk-size").isVisible().catch(() => false)
  console.log(`Chunk size field visible: ${chunkVisible ? "YES" : "NO"}`)

  log("Running the export")
  await page.getByRole("button", { name: "Run" }).click()
  const exportDone = await page
    .getByText("Exported", { exact: false })
    .waitFor({ state: "visible", timeout: 5000 })
    .then(() => true)
    .catch(() => false)
  console.log(`Export completed: ${exportDone ? "YES" : "NO"}`)
  await page.getByRole("button", { name: "Close" }).first().click()

  // ---------------------------------------------------------------------
  // 6. Structure tab: see structure, CRUD columns, new table
  // ---------------------------------------------------------------------
  log("Opening the Structure tab")
  await page.getByRole("button", { name: "Structure", exact: true }).click()
  const structureLoaded = await page
    .getByText("email", { exact: true })
    .first()
    .waitFor({ state: "visible", timeout: 5000 })
    .then(() => true)
    .catch(() => false)
  console.log(`Structure tab shows real columns: ${structureLoaded ? "YES" : "NO"}`)

  log("Adding a new column")
  await page.getByRole("button", { name: "Add column" }).click()
  await page.locator('input[placeholder="column_name"]').last().fill("nickname")
  await page.locator('input[placeholder="text, varchar(255), int8, ..."]').last().fill("text")
  await page.keyboard.press("Escape")

  log("Saving the staged column change (opens Review SQL)")
  await page.getByRole("button", { name: "Save", exact: true }).click()
  await page.getByText("Review SQL", { exact: true }).waitFor({ state: "visible", timeout: 5000 })

  log("Running the DDL")
  await page.getByRole("button", { name: "Run", exact: true }).click()
  const ddlDone = await page
    .getByText("Schema updated", { exact: false })
    .waitFor({ state: "visible", timeout: 5000 })
    .then(() => true)
    .catch(() => false)
  console.log(`DDL executed (column added): ${ddlDone ? "YES" : "NO"}`)
  await page.keyboard.press("Escape")

  // ---------------------------------------------------------------------
  // 7. New Query tab: run CRUD via raw SQL
  // ---------------------------------------------------------------------
  log("Opening a new SQL query tab")
  await page.getByRole("button", { name: "New Query", exact: true }).click()
  const editor = page.locator(".cm-content").first()

  async function runSql(sql, label) {
    await editor.click()
    await page.keyboard.press("Control+a")
    await page.keyboard.press("Delete")
    await page.waitForFunction(() => document.querySelector(".cm-content")?.textContent === "")
    await page.keyboard.type(sql)
    await page.getByRole("button", { name: "Run" }).first().click()
    await page.waitForTimeout(600)
    console.log(`Ran ${label}: (see screenshot for result)`)
  }

  await runSql("select * from users;", "SELECT")
  await runSql("insert into users (email, plan) values ('new@example.com', 'free');", "INSERT")
  await runSql("update users set plan = 'pro' where id = 1;", "UPDATE")
  await runSql("delete from users where id = 2;", "DELETE")

  // ---------------------------------------------------------------------
  // 8. Schema graph
  // ---------------------------------------------------------------------
  log("Going back to the users table and opening the Schema tab")
  await page.getByText("users", { exact: true }).first().click()
  await page.getByRole("button", { name: "Schema", exact: true }).click()
  const schemaLoaded = await page
    .locator(".react-flow")
    .waitFor({ state: "visible", timeout: 8000 })
    .then(() => true)
    .catch(() => false)
  console.log(`Schema graph rendered: ${schemaLoaded ? "YES" : "NO"}`)

  // ---------------------------------------------------------------------
  // 9. Pagination against the real 5,000-row table
  // ---------------------------------------------------------------------
  log("Opening the large 'pagination_test' table (real data if fixture present)")
  await page.getByText("pagination_test", { exact: true }).first().click()
  await page.getByText("of 25", { exact: false }).waitFor({ state: "visible", timeout: 8000 })
  console.log(`Pagination shows 25 pages for 5,000 rows @ 200/page: YES`)

  log("Clicking Next page")
  const nextButtons = page.locator('button:has(svg)')
  await page.locator("span", { hasText: "Page 1" }).locator("xpath=following-sibling::button[1]").click()
  await page.getByText("Page 2", { exact: false }).waitFor({ state: "visible", timeout: 5000 })
  console.log("Advanced to page 2: YES")

  // ---------------------------------------------------------------------
  // 10. Sidebar connection management
  // ---------------------------------------------------------------------
  log("Opening the connection's '...' menu")
  await page.getByTitle("More options").first().click()

  log("Copying the connection string")
  await page.getByText("Copy Connection String", { exact: true }).click()

  log("Renaming the connection")
  await page.getByTitle("More options").first().click()
  await page.getByText("Rename", { exact: true }).click()
  await page.getByText("Rename Connection", { exact: true }).waitFor({ state: "visible", timeout: 5000 })
  await page.locator('[role="dialog"] input').first().fill("Renamed Postgres Connection")
  await page.getByRole("button", { name: "Save", exact: true }).click()
  const renamed = await page
    .getByText("Renamed Postgres Connection", { exact: false })
    .first()
    .isVisible()
    .catch(() => false)
  console.log(`Rename connection worked: ${renamed ? "YES" : "NO"}`)

  log("Exporting tables from the sidebar menu")
  await page.getByTitle("More options").first().click()
  await page.getByText("Export Tables", { exact: false }).click()
  await page.getByText("Export Tables", { exact: true }).waitFor({ state: "visible", timeout: 5000 }).catch(() => {})
  await page.keyboard.press("Escape")

  log("Disconnecting")
  await page.getByTitle("More options").first().click()
  await page.getByText("Disconnect", { exact: true }).click()

  log("Deleting the connection")
  await page.getByTitle("More options").first().click()
  await page.getByText("Delete Connection", { exact: true }).click()
  await page.getByRole("button", { name: "Delete", exact: true }).click()
  const deleted = await page
    .getByText("Renamed Postgres Connection", { exact: false })
    .first()
    .isVisible()
    .catch(() => false)
  console.log(`Connection deleted (should be gone): ${deleted ? "STILL THERE (bug)" : "YES, gone"}`)

  await page.screenshot({
    path: join(__dirname, "..", "..", "..", "..", "..", "tmp-postgres-final.png"),
  })

  console.log("… Full walkthrough finished. Leaving the window open — close it manually when done.")
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
