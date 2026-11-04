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
