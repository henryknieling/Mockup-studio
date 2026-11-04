set dotenv-load := true

DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://mockup:mockup@localhost:5432/mockup_studio")

run:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose up -d --wait
    export DATABASE_URL="{{DATABASE_URL}}"
    cargo run

watch:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose up -d --wait
    export DATABASE_URL="{{DATABASE_URL}}"
    cargo watch -x run

stop:
    docker compose down

migrate:
    DATABASE_URL="{{DATABASE_URL}}" cargo run -- migrate

migrations:
    @find migrations -maxdepth 1 -type f -print | sort

fmt:
    cargo fmt --all

test:
    cargo test

check:
    cargo fmt --all -- --check
    cargo check
    cargo clippy --all-targets -- -D warnings
