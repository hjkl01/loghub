.PHONY: help ensure-env dev build run clean migrate lint test docker-up docker-down

.DEFAULT_GOAL := help

# 检查 .env 是否已从 .env.example 复制, 不存在则自动复制
ensure-env:
	@if [ ! -f .env ]; then \
		echo "==> .env 不存在, 自动从 .env.example 复制"; \
		cp .env.example .env; \
	else \
		echo "==> .env 已存在"; \
	fi

help:
	@echo "LogHub 管理命令"
	@echo "================"
	@echo ""
	@echo "后端:"
	@echo "  make dev          启动开发模式 (cargo watch)"
	@echo "  make build        编译 release"
	@echo "  make run          运行 release"
	@echo "  make migrate      执行数据库迁移"
	@echo "  make lint         Clippy 代码检查"
	@echo "  make test         运行测试"
	@echo ""
	@echo "前端:"
	@echo "  make frontend-install  安装依赖"
	@echo "  make frontend-dev      启动开发服务器"
	@echo "  make frontend-build    构建生产版本"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-up    启动所有服务"
	@echo "  make docker-down  停止服务"
	@echo "  make docker-build 重新构建镜像"
	@echo "  make docker-logs  查看日志"
	@echo ""
	@echo "数据库:"
	@echo "  make db-create    创建数据库"
	@echo "  make db-drop      删除数据库"
	@echo "  make db-reset     重置数据库"
	@echo ""
	@echo "其他:"
	@echo "  make clean        清理构建产物"
	@echo "  make setup        初始化项目 (建库+迁移+安装前端)"
	@echo "  make all          构建后端+前端"

# Backend
dev: ensure-env frontend-build
	cargo install cargo-watch && cargo watch -x run -w backend/src

build: ensure-env frontend-build
	cargo build --release

run: ensure-env frontend-build
	cargo run --release

clean: ensure-env
	cargo clean

migrate: ensure-env
	cargo run --release -- migrate

lint: ensure-env
	cargo clippy -- -D warnings

test: ensure-env
	cargo test

# Docker
docker-up: ensure-env
	docker-compose up -d

docker-down: ensure-env
	docker-compose down

docker-build: ensure-env
	docker-compose build

docker-logs: ensure-env
	docker-compose logs -f

# Database
db-create: ensure-env
	createdb -U postgres loghub

db-drop: ensure-env
	dropdb -U postgres loghub

db-reset: ensure-env db-drop db-create migrate

# Frontend
frontend-install: ensure-env
	cd frontend && pnpm install

frontend-dev: ensure-env frontend-install
	cd frontend && pnpm dev

frontend-build: ensure-env frontend-install
	cd frontend && pnpm build

frontend-lint: ensure-env frontend-install
	cd frontend && pnpm lint

# All
setup: ensure-env db-create migrate frontend-install

all: build frontend-build
