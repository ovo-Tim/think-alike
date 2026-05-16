#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT_DIR/.env"

if [[ ! -f "$ENV_FILE" ]]; then
  printf 'Missing .env. Copy .env.example to .env first.\n' >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  printf 'Docker is required for local Postgres.\n' >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  printf 'Docker daemon is not running. Start Docker Desktop first.\n' >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  printf 'cargo is required for the backend dev server.\n' >&2
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  printf 'npx is required for the frontend dev server.\n' >&2
  exit 1
fi

set -a
source "$ENV_FILE"
set +a

if [[ -z "${POSTGRES_PASSWORD:-}" ]]; then
  printf 'POSTGRES_PASSWORD must be set in .env.\n' >&2
  exit 1
fi

printf 'Starting Postgres on 127.0.0.1:55432...\n'
docker compose -f "$ROOT_DIR/docker-compose.yml" -f "$ROOT_DIR/docker-compose.dev.yml" up -d db

cleanup() {
  if [[ -n "${BACKEND_PID:-}" ]]; then
    kill "$BACKEND_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${FRONTEND_PID:-}" ]]; then
    kill "$FRONTEND_PID" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT INT TERM

export DATABASE_URL="postgres://postgres:${POSTGRES_PASSWORD}@127.0.0.1:55432/think_alike"

printf 'Starting backend on http://localhost:3000...\n'
cargo run --manifest-path "$ROOT_DIR/backend/Cargo.toml" &
BACKEND_PID=$!

printf 'Starting frontend on http://localhost:5173...\n'
npx pnpm --dir "$ROOT_DIR/frontend" dev --host 0.0.0.0 &
FRONTEND_PID=$!

printf '\nLocal development is starting.\n'
printf 'Frontend: http://localhost:5173\n'
printf 'Backend:  http://localhost:3000\n'
printf 'Postgres: postgres://postgres:<redacted>@127.0.0.1:55432/think_alike\n'
printf 'Embedding server expected at: http://127.0.0.1:8080/v1/embeddings\n\n'

wait "$BACKEND_PID" "$FRONTEND_PID"
