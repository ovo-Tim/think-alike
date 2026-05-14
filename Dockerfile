# syntax=docker/dockerfile:1.7

FROM node:22-bookworm-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/tsconfig.json frontend/tsconfig.node.json frontend/vite.config.ts frontend/index.html ./
COPY frontend/src ./src
RUN corepack enable && pnpm install --frozen-lockfile=false && pnpm build

FROM rust:1.87-bookworm AS backend-builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/src backend/src
COPY infra infra
COPY --from=frontend-builder /app/frontend/dist frontend/dist
RUN cargo build --manifest-path backend/Cargo.toml --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser
WORKDIR /app
COPY --from=backend-builder /app/target/release/think-alike-backend /app/think-alike-backend
COPY --from=backend-builder /app/frontend/dist /app/frontend/dist
COPY infra /app/infra
USER appuser
EXPOSE 3000
ENV RUST_LOG=info
CMD ["/app/think-alike-backend"]
