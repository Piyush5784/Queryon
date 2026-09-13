# Queryon:- Desktop database client 

A fast, native desktop app for working with databases, built with React and Tauri instead of Electron for a lighter footprint. Think of it as a local, more flexible alternative to tools like Neon's web console: 
* Connect to 10+ database engines (Postgres, SQLite, DuckDB, NeonDB, and more), 
* Run queries with real syntax suggestions, and manage multiple databases side by side across tabs, all without leaving your desktop.

## Demo

<img width="1717" height="930" alt="image" src="https://github.com/user-attachments/assets/9f3660de-4da0-467c-a7d9-82a379eb30aa" />
<img width="1717" height="930" alt="image" src="https://github.com/user-attachments/assets/1a17246a-addf-487a-9365-745ea6444fc0" />
<img width="1717" height="930" alt="image" src="https://github.com/user-attachments/assets/20f52157-44fc-410e-ac66-ef474597482a" />


Video Link: [Demo Video](https://res.cloudinary.com/dzf9kamfw/video/upload/queryon-demo-1788368408502_c2eeeq.mp4)


A pnpm monorepo:

- **`apps/desktop/`** — the Queryon desktop database client (Tauri + React + Rust). See `apps/desktop/README.md`.
- **`apps/web/`** — the marketing site and install docs (Next.js).

## Setup

```bash
pnpm install
```

## Working on the desktop app

```bash
pnpm --filter queryon tauri dev
```

or `cd apps/desktop && pnpm tauri dev`. See `apps/desktop/README.md` for database setup and testing.

## Working on the website

```bash
pnpm --filter queryon-web dev
```

or `cd apps/web && pnpm dev`.
