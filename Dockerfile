# ---- Backend builder ----
FROM rust:bookworm AS backend-builder
WORKDIR /app
# Debian apt 镜像
RUN sed -i 's|deb.debian.org|mirrors.aliyun.com|g' /etc/apt/sources.list.d/debian.sources
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*
# Rust crates.io 镜像
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN mkdir -p ~/.cargo && echo '[source.crates-io]' > ~/.cargo/config.toml && \
    echo 'replace-with = "ustc"' >> ~/.cargo/config.toml && \
    echo '[source.ustc]' >> ~/.cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> ~/.cargo/config.toml
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --release -p loghub-backend

# ---- Frontend builder ----
FROM node:22-bookworm AS frontend-builder
WORKDIR /app
# npm 镜像
RUN corepack enable && corepack prepare pnpm@10 --activate
RUN pnpm config set registry https://registry.npmmirror.com
COPY frontend/package.json frontend/pnpm-lock.yaml ./
ENV CI=true
RUN pnpm install --frozen-lockfile
COPY frontend/ .
RUN pnpm build

# ---- Runtime ----
FROM debian:bookworm-slim
RUN sed -i 's|deb.debian.org|mirrors.aliyun.com|g' /etc/apt/sources.list.d/debian.sources
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
ENV TZ=Asia/Shanghai
COPY --from=backend-builder /app/target/release/loghub-backend /usr/local/bin/loghub-backend
COPY --from=backend-builder /app/backend/migrations /app/migrations
COPY .env /app/.env
COPY --from=frontend-builder /app/dist /app/frontend/dist
WORKDIR /app
CMD ["sh", "-c", "loghub-backend migrate && loghub-backend"]
