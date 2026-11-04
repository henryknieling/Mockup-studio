# Mockup Studio

Mockup Studio is a lightweight, local-first editor for creating app and website screen mockups. Organize work into projects, compose screens on an interactive canvas, and export a single screen or a complete project as a crisp, vector-based PDF. It is designed for one user on one device; there are no accounts or hosted service.

## Features

- Organize mockups into projects with multiple screens.
- Design mobile and web layouts using viewport presets or custom dimensions.
- Add, move, resize, and configure components such as text, buttons, images, inputs, cards, and navigation.
- Inspect component properties and screen layers; undo and redo editor changes.
- Save projects and screen content in a local PostgreSQL database.
- Export individual screens or every screen in a project to PDF.

## Tech stack

- Rust and Axum for the web server and APIs.
- Server-rendered HTML with JavaScript for the interactive canvas editor.
- PostgreSQL for persistence, managed locally with Docker Compose.
- `printpdf` for PDF generation.

## Architecture

Mockup Studio is a small, single-process web application. The Rust server owns project and screen management, persistence, and PDF generation; the browser handles the interactive editing experience.

```text
Browser
  ├─ HTML pages and forms ──> Axum routes in `src/main.rs`
  ├─ `static/editor.js` ────> screen document API (`/api/screens/:id`)
  └─ PDF download ──────────> export routes (`/export/...`)
                                  │
                              SQLx / PgPool
                                  │
                              PostgreSQL
```

- **Startup and routing:** `src/main.rs` creates a Tokio runtime, connects a SQLx connection pool using `DATABASE_URL`, runs embedded SQLx migrations, and starts Axum on `127.0.0.1:3000`. Axum serves the project and screen pages, form actions, JSON API, and static assets.
- **Editor:** The server renders the editor page; `static/editor.js` manages canvas state, component interactions, selection, layers, and undo/redo in the browser. It loads a screen through `GET /api/screens/:id` and saves the versioned JSON document through a debounced `PUT` to the same route. Project and screen names, ordering, and viewport settings are handled by Axum form routes.
- **Persistence:** PostgreSQL stores projects and their screens. A screen contains its viewport dimensions and device frame, plus a versioned JSONB document of canvas elements. Foreign-key cascading removes a project's screens when the project is deleted. Schema changes live in `migrations/` and are applied at startup.
- **PDF export:** The export routes load saved screen documents and render them on the server with `printpdf`. A screen export produces one page; a project export produces one page per screen in project order. Text and shapes are written as PDF content rather than a screenshot of the editor.
- **Local services and commands:** `docker-compose.yml` runs PostgreSQL for local development, while `justfile` provides run, stop, migration, formatting, test, and check commands.

## Requirements

- Rust stable (edition 2024)
- Docker Compose
- [`just`](https://just.systems/)

## Run locally

From the repository root, run:

```sh
just run
```

This starts PostgreSQL in the background, applies embedded database migrations on server startup, and serves the app at <http://localhost:3000>. Stop the database with `just stop`.

The default database connection is `postgres://mockup:mockup@localhost:5432/mockup_studio`. Set `DATABASE_URL` to use a different local database.

## Development

- `just test` — run tests.
- `just fmt` — format Rust code.
- `just check` — run formatting checks, compilation, and Clippy.
- `just migrations` — list database migration files.
- `just watch` — run the app with automatic rebuilds (requires `cargo-watch`).
