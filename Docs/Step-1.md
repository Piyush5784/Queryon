Build a modern desktop PostgreSQL database client using Tauri + React + TypeScript.

The product should feel like a desktop-first alternative to modern database consoles such as Neon/Drizzle Studio, NOT like a traditional enterprise database client such as DBeaver.

Core goal:

Create a beautiful, fast, developer-focused PostgreSQL workspace where developers can connect to databases, explore schemas/tables, write SQL, execute queries, and inspect/edit data.

Tech stack:

\- Tauri

\- React

\- TypeScript

\- Vite

\- Tailwind CSS

\- shadcn/ui

\- CodeMirror or Monaco for the SQL editor

\- TanStack Router if routing is needed

\- Zustand for client state where appropriate

\- PostgreSQL connectivity through a safe Tauri/native layer

\- Keep Rust minimal initially; don't add unnecessary Rust complexity

Design direction:

\- Modern SaaS/developer-tool aesthetic

\- Dark-first UI

\- Clean and minimal

\- Dense but readable information layout

\- Smooth interactions

\- Excellent keyboard navigation

\- No unnecessary gradients or flashy animations

\- Should feel like a serious developer product

\- Inspired by the usability and visual quality of modern tools like Neon and Drizzle Studio, but do NOT copy their branding or exact UI

Main application layout:

1\. Left sidebar

- Workspace/project

- Database connections

- Schemas

- Tables

- Views

- Functions

- Enums

- Saved queries

- Query history

2\. Main workspace

- SQL editor

- Query tabs

- Run query button

- Query execution time

- Results panel

- Error panel

3\. Table explorer

- Rows displayed in an interactive spreadsheet-like table

- Sorting

- Filtering

- Column resizing

- Column visibility

- Pagination

- Inline editing

- Insert row

- Delete row

- Refresh

- Copy cells/rows

- Export data

4\. Schema explorer

- Tables

- Columns

- Data types

- Primary keys

- Foreign keys

- Indexes

- Constraints

- Relationships

5\. Query system

- Multiple SQL tabs

- Query history

- Saved queries

- Keyboard shortcut to execute

- Clear error messages

- Execution duration

- Result row count

Important UX:

\- Clicking a table should immediately open its data.

\- Double-clicking a cell should allow editing.

\- SQL queries should be executable with Cmd/Ctrl + Enter.

\- Results should support large datasets using virtualization.

\- Loading states should be polished.

\- Empty states should be useful.

\- Errors should be understandable.

\- Avoid unnecessary modal dialogs.

Architecture:

Keep the frontend cleanly separated from the native/database layer.

React:

\- UI

\- routing

\- state

\- table rendering

\- SQL editor

\- query/result presentation

Tauri:

\- native database communication

\- secure credential handling

\- filesystem integration

\- OS-level functionality

Security:

\- Never expose database credentials unnecessarily to the frontend.

\- Never hardcode credentials.

\- Use secure local storage/keychain where possible.

\- Make destructive database operations require confirmation.

\- Clearly distinguish read and write operations.

MVP FIRST:

Do not build every feature immediately.

First implement only:

1\. Application shell

2\. Database connection screen

3\. PostgreSQL connection

4\. Connection management

5\. Database/schema/table explorer

6\. SQL editor

7\. Execute SQL

8\. Results table

9\. Basic table browsing

10\. Basic row editing

11\. Query history

Make the architecture extensible so we can later add:

\- AI SQL assistant

\- Drizzle schema integration

\- Prisma schema integration

\- ER diagrams

\- Query visualization

\- EXPLAIN/ANALYZE visualization

\- Database migrations

\- CSV/JSON import/export

\- Multiple database engines

\- Local Ollama integration

\- Database diffing

\- Schema comparison

\- Branching/workspaces

\- Migrate the table/query results grid from @tanstack/react-table + @tanstack/react-virtual (DOM-rendered) to @glideapps/glide-data-grid (canvas-rendered) for large-row-count scale. Deferred deliberately: would require rebuilding NULL/boolean/JSON cell rendering, inline text editing, and the pending-edit Save/Discard flow against Glide's cell-renderer + edit-overlay APIs. Revisit once core features (SQL editor, saved queries, schema views) are in place, so grid work isn't done twice.

Development approach:

Before implementing complex functionality, create the project structure and core UI shell.

Use realistic mock database data initially if native PostgreSQL connectivity makes the first implementation too large.

Do not over-engineer.

Do not introduce unnecessary dependencies.

Do not implement fake functionality while presenting it as real functionality.

Build the MVP incrementally and keep the code production-quality.

Start by creating the project architecture and the initial application shell.
