// Manual, watchable walkthrough of the MongoDB document browsing + CRUD UI.
// Runs the real React app (against the Vite dev server) with the Tauri IPC
// bridge mocked out, so no real Rust backend or database is needed.
//
// Usage:
//   1. In apps/desktop: npm run dev   (starts Vite on http://localhost:1420)
//   2. node tests/collection-view.spec.mjs
//
// Launches a headed (visible) browser window and pauses ~2s between steps
// so you can watch each interaction happen.

import { chromium } from "playwright"

const STEP_DELAY_MS = 1000

const MOCK_CONNECTION_ID = "mock-mongo-1"

const MOCK_SAVED_CONNECTION = {
  id: MOCK_CONNECTION_ID,
  name: "GUI Test Mongo",
  engine: "mongo-db",
  host: "mongodb://localhost:27119/guitest",
  port: 0,
  database: "guitest",
  user: "",
  password: "",
  readOnly: false,
  sslMode: "disable",
  sshTunnel: null,
}

const MOCK_COLLECTIONS = [
  {
    name: "products",
    estimatedCount: 3,
    storageSizeBytes: 4096,
    avgDocumentSizeBytes: 96,
    indexCount: 2,
    totalIndexSizeBytes: 8192,
  },
  {
    name: "orders",
    estimatedCount: 2,
    storageSizeBytes: 2048,
    avgDocumentSizeBytes: 64,
    indexCount: 1,
    totalIndexSizeBytes: 4096,
  },
]

const MOCK_DOCS_BY_COLLECTION = {
  products: [
    { id: "p1", preview: '{ name: "Widget", price: 9.99, inStock: true }' },
    { id: "p2", preview: '{ name: "Gadget", price: 19.99, inStock: false }' },
    { id: "p3", preview: '{ name: "Gizmo", price: 29.99, inStock: true }' },
  ],
  orders: [
    { id: "o1", preview: '{ orderId: "A1", total: 100 }' },
    { id: "o2", preview: '{ orderId: "A2", total: 200 }' },
  ],
}

const MOCK_DOCUMENT_BODIES = {
  // Realistic shape matching what the Rust driver actually serializes for
  // MongoDB ObjectId/Date fields: { "$oid": "..." } / { "$date": {...} }.
  p1: {
    _id: { $oid: "67a5ac2db81512d0b4ffddb6" },
    name: "Widget",
    price: 9.99,
    inStock: true,
    createdAt: { $date: { $numberLong: "1732000000000" } },
    createdBy: { $oid: "67a52894242e5460876ce7fb" },
  },
  p2: { _id: { $oid: "67a5ac2db81512d0b4ffddb7" }, name: "Gadget", price: 19.99, inStock: false },
  p3: { _id: { $oid: "67a5ac2db81512d0b4ffddb8" }, name: "Gizmo", price: 29.99, inStock: true },
  o1: { _id: { $oid: "67a5ac2db81512d0b4ffddb9" }, orderId: "A1", total: 100 },
  o2: { _id: { $oid: "67a5ac2db81512d0b4ffddba" }, orderId: "A2", total: 200 },
}

async function pause(label) {
  console.log(`… ${label}`)
  await new Promise((resolve) => setTimeout(resolve, STEP_DELAY_MS))
}

async function main() {
  const browser = await chromium.launch({ headless: false, args: ["--no-sandbox", "--start-maximized"] })
  const page = await browser.newPage({ viewport: null })

  page.on("console", (m) => {
    if (m.type() === "error") console.log(`[browser console error] ${m.text()}`)
  })
  page.on("pageerror", (e) => console.log(`[browser pageerror] ${e.message}`))

  await page.addInitScript(
    ({ connectionId, savedConnection, collections, docsByCollection, bodies }) => {
      // In-memory mutable store so insert/update/delete actually behave
      // like a real database for the duration of this test.
      const state = {
        collections: JSON.parse(JSON.stringify(collections)),
        docsByCollection: JSON.parse(JSON.stringify(docsByCollection)),
        bodies: JSON.parse(JSON.stringify(bodies)),
        nextId: 100,
      }

      function previewOf(doc) {
        const parts = Object.entries(doc)
          .filter(([k]) => k !== "_id")
          .slice(0, 4)
          .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
        return `{ ${parts.join(", ")} }`
      }

      function handle(cmd, args) {
        switch (cmd) {
          case "db_list_saved_connections":
            return [savedConnection]
          case "doc_list_active_connections":
            return [connectionId]
          case "doc_default_database":
            return savedConnection.database
          case "doc_connect_saved":
          case "doc_connect":
            return { id: connectionId, serverVersion: "7.0.0-mock" }
          case "doc_disconnect":
            return null
          case "doc_list_databases":
            return [{ name: savedConnection.database }]
          case "doc_list_collections":
            return state.collections
          case "doc_list_documents": {
            const docs = state.docsByCollection[args.collection] ?? []
            return { documents: docs, hasMore: false, durationMs: 2 }
          }
          case "doc_get_document": {
            const body = state.bodies[args.id]
            return body ? JSON.stringify(body) : null
          }
          case "doc_insert_document": {
            const id = `new-${state.nextId++}`
            const parsed = JSON.parse(args.document)
            parsed._id = id
            state.bodies[id] = parsed
            const list = state.docsByCollection[args.collection] ?? []
            list.push({ id, preview: previewOf(parsed) })
            state.docsByCollection[args.collection] = list
            const coll = state.collections.find((c) => c.name === args.collection)
            if (coll) coll.estimatedCount += 1
            return id
          }
          case "doc_update_document": {
            const parsed = JSON.parse(args.document)
            parsed._id = args.id
            state.bodies[args.id] = parsed
            const list = state.docsByCollection[args.collection] ?? []
            const row = list.find((d) => d.id === args.id)
            if (row) row.preview = previewOf(parsed)
            return null
          }
          case "doc_delete_document": {
            delete state.bodies[args.id]
            const list = state.docsByCollection[args.collection] ?? []
            state.docsByCollection[args.collection] = list.filter((d) => d.id !== args.id)
            const coll = state.collections.find((c) => c.name === args.collection)
            if (coll) coll.estimatedCount = Math.max(0, coll.estimatedCount - 1)
            return null
          }
          case "plugin:updater|check":
            return null
          case "plugin:event|listen":
            return 0
          case "plugin:event|unlisten":
            return null
          default:
            console.error("UNMOCKED TAURI COMMAND:", cmd, JSON.stringify(args))
            return null
        }
      }

      window.__TAURI_INTERNALS__ = {
        invoke: (cmd, args) =>
          new Promise((resolve, reject) => {
            try {
              resolve(handle(cmd, args ?? {}))
            } catch (err) {
              reject(err)
            }
          }),
        transformCallback: (cb) => cb,
        convertFileSrc: (p) => p,
      }
    },
    {
      connectionId: MOCK_CONNECTION_ID,
      savedConnection: MOCK_SAVED_CONNECTION,
      collections: MOCK_COLLECTIONS,
      docsByCollection: MOCK_DOCS_BY_COLLECTION,
      bodies: MOCK_DOCUMENT_BODIES,
    }
  )

  await pause("Loading the app (Tauri bridge mocked)")
  await page.goto("http://localhost:1420", { waitUntil: "networkidle", timeout: 20000 })

  await pause("Looking for the saved 'GUI Test Mongo' connection card")
  await page.getByRole("main").getByText("GUI Test Mongo").click()

  await pause("Connecting…")
  await page.waitForSelector("text=guitest", { timeout: 10000 })

  await pause("Expanding the 'guitest' database in the sidebar")
  await page.getByText("guitest", { exact: true }).first().click()

  await pause("Opening the collections overview screen")
  await page.waitForSelector("text=Storage size", { timeout: 10000 })

  await pause("Clicking into the 'products' collection card")
  await page.getByText("products", { exact: true }).first().click()

  await pause("Waiting for the document list to load")
  await page.waitForSelector("text=Widget", { timeout: 10000 })

  await pause("Clicking the first document row")
  await page.getByText("Widget", { exact: false }).first().click()

  await pause("Checking the detail panel actually shows the document (this was the reported bug)")
  const detailVisible = await page
    .getByText("Widget", { exact: false })
    .nth(1) // 0 = the row preview text, 1 = the detail panel's rendered value
    .isVisible()
    .catch(() => false)
  console.log(`Detail panel showing document content: ${detailVisible ? "YES" : "NO (bug reproduced)"}`)

  await pause("Clicking Edit on the selected document")
  await page.getByRole("button", { name: "Edit" }).click().catch(() => console.log("Edit button not found"))

  await pause("Clicking Cancel to back out of edit mode")
  await page.getByRole("button", { name: "Cancel" }).click().catch(() => console.log("Cancel button not found"))

  await pause("Clicking the '+' New document button")
  await page.getByRole("button", { name: "New document" }).click()

  await pause("Typing a new document's JSON into the editor")
  const editor = page.locator("textarea")
  await editor.fill('{\n  "name": "Doohickey",\n  "price": 4.5\n}')

  await pause("Saving the new document")
  await page.getByRole("button", { name: "Save" }).click()

  await pause("Confirming the new document now appears in the list")
  const newDocVisible = await page
    .getByText("Doohickey", { exact: false })
    .first()
    .isVisible()
    .catch(() => false)
  console.log(`New document appears in list: ${newDocVisible ? "YES" : "NO (insert flow broken)"}`)

  await pause("Selecting the first document again to test delete")
  await page.getByText("Widget", { exact: false }).first().click()

  await pause("Clicking Delete")
  await page.getByRole("button", { name: "Delete" }).click()

  await pause("Confirming the delete in the alert dialog")
  await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click()

  await pause("Confirming the document is now gone from the list")
  const stillThere = await page
    .getByText("Widget", { exact: false })
    .first()
    .isVisible()
    .catch(() => false)
  console.log(`Deleted document still visible: ${stillThere ? "YES (delete flow broken)" : "NO (deleted correctly)"}`)

  await pause("Test finished. Leaving the window open for you to inspect — close it manually when done.")

  // Intentionally not closing the browser so you can keep looking at it.
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
