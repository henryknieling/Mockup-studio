# Mockup Studio

A single-user mockup editor designed to run locally. It uses Rust/Axum for server-rendered project pages and APIs, PostgreSQL for persistence, and a small JavaScript canvas editor.

## Requirements

- Rust stable (edition 2024)
- Docker Compose
- `just`

## Run locally

```sh
just run
```

This starts PostgreSQL in the background, applies embedded migrations on server startup, and serves the app at <http://localhost:3000>. Stop the database with `just stop`. The default database connection is `postgres://mockup:mockup@localhost:5432/mockup_studio`; override it with `DATABASE_URL` if needed.

Other useful commands: `just test`, `just fmt`, `just check`, and `just migrations`.

The editor supports screen-specific viewport presets, component placement and resizing, an inspector and layers list, undo/redo, and exporting a screen or project to PDF. PDFs are generated as vector documents for crisp output.
