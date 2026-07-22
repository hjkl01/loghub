-- users
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(128) NOT NULL UNIQUE,
    password_hash VARCHAR(256) NOT NULL,
    role VARCHAR(32) NOT NULL DEFAULT 'viewer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- rules
CREATE TABLE rules (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(256) NOT NULL,
    match_pattern VARCHAR(512) NOT NULL,
    level VARCHAR(20),
    extract JSONB DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- logs
CREATE TABLE logs (
    id BIGSERIAL PRIMARY KEY,
    log_time TIMESTAMPTZ NOT NULL,
    ingest_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level VARCHAR(20) NOT NULL DEFAULT 'INFO',
    message TEXT NOT NULL,
    system VARCHAR(128) NOT NULL,
    service VARCHAR(128) NOT NULL,
    trace_id VARCHAR(128),
    request_id VARCHAR(128),
    extra JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- indexes for logs
CREATE INDEX idx_logs_log_time ON logs(log_time DESC);
CREATE INDEX idx_logs_system_time ON logs(system, log_time DESC);
CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_message_search ON logs USING GIN (to_tsvector('simple', message));
CREATE INDEX idx_logs_service ON logs(service);
