# ---- Backend builder ----
FROM rust:bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --release -p loghub-backend

# ---- Frontend builder ----
FROM node:22-bookworm AS frontend-builder
WORKDIR /app
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack enable && corepack prepare pnpm@10 --activate
ENV CI=true
RUN pnpm install --frozen-lockfile
COPY frontend/ .
RUN pnpm build

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/target/release/loghub-backend /usr/local/bin/loghub-backend
COPY --from=backend-builder /app/backend/migrations /app/migrations
COPY .env /app/.env
COPY --from=frontend-builder /app/dist /app/frontend/dist
WORKDIR /app
CMD ["loghub-backend"]
