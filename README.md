# LogHub

轻量级应用日志管理平台，用于收集 Python、Rust、Golang 等应用产生的结构化日志。

## 特性

- **多语言接入**：支持 Python、Rust、Golang 等任意语言通过 HTTP API 上报日志
- **实时推送**：WebSocket 实时日志流，支持多客户端、自定义过滤条件
- **全文搜索**：PostgreSQL GIN 索引，支持日志内容全文检索
- **结构化日志**：JSONB 扩展字段，灵活存储业务上下文
- **日志规则**：可配置的匹配规则，自动识别异常类型
- **JWT 认证**：admin/viewer 角色权限控制
- **Swagger UI**：内置 API 文档 `/swagger-ui`

## 技术栈

| 层 | 技术 |
|---|------|
| 后端 | Rust + Axum + Tokio + SQLx |
| 前端 | Vue 3 + TypeScript + Element Plus + Pinia |
| 数据库 | PostgreSQL 16+ (JSONB) |
| 部署 | Docker Compose |

## 快速开始

### Docker Compose（推荐）

```bash
docker-compose up -d
```

服务启动后访问：
- 前端：http://localhost:8080
- API 文档：http://localhost:8080/swagger-ui
- 健康检查：http://localhost:8080/api/health

### 手动部署

```bash
# 1. 启动 PostgreSQL
docker run -d --name postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:15-alpine

docker exec postgres psql -U postgres -c "CREATE DATABASE loghub;"

# 2. 构建前端
cd frontend && pnpm install && pnpm run build && cd ..

# 3. 编译并启动后端
cargo run --release
```

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `LOG__SERVER__PORT` | 服务端口 | 8080 |
| `LOG__DATABASE__URL` | PostgreSQL 连接串 | postgres://postgres:postgres@localhost:5432/loghub |
| `LOG__AUTH__JWT_SECRET` | JWT 密钥 | loghub-secret-key-change-in-production |
| `LOG__ADMIN__USERNAME` | 初始管理员用户名 | admin |
| `LOG__ADMIN__PASSWORD` | 初始管理员密码 | admin123 |

## API 接口

### 认证

```
POST /api/auth/login     # 登录获取 Token
GET  /api/auth/me         # 获取当前用户信息
```

### 日志

```
POST /api/logs            # 上报日志
GET  /api/logs             # 查询日志
GET  /api/logs/ws          # WebSocket 实时日志
```

### 规则

```
GET    /api/rules          # 规则列表
POST   /api/rules          # 创建规则
PUT    /api/rules/:id/toggle  # 启用/禁用规则
DELETE /api/rules/:id      # 删除规则
```

### 其他

```
GET /api/health            # 健康检查
GET /swagger-ui            # API 文档
```

---

## 接入指南

### 日志上报格式

```json
POST /api/logs
Content-Type: application/json

{
  "timestamp": "2026-07-21T10:20:30Z",
  "level": "ERROR",
  "message": "database connection timeout",
  "system": "order-system",
  "service": "order-api",
  "trace_id": "abc123",
  "request_id": "req001",
  "file_name": "app/db.py",
  "function_name": "connect",
  "line_number": 42,
  "extra": {
    "user_id": 42,
    "retry_count": 3
  }
}
```

**字段说明：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| timestamp | ISO8601 | ✅ | 日志产生时间 |
| level | string | ✅ | 日志级别：DEBUG/INFO/WARN/ERROR/FATAL |
| message | string | ✅ | 日志内容 |
| system | string | ✅ | 系统名称（如 "order-system"） |
| service | string | ✅ | 服务名称（如 "order-api"） |
| trace_id | string | ❌ | 链路追踪 ID |
| request_id | string | ❌ | 请求 ID |
| file_name | string | ❌ | 源文件名（如 "app/db.py"） |
| function_name | string | ❌ | 函数名（如 "connect"） |
| line_number | integer | ❌ | 行号（如 42） |
| extra | object | ❌ | 扩展字段（JSONB 存储） |

### Python 接入

#### 方式一：requests 直接上报

```python
import requests
from datetime import datetime, timezone

LOGHUB_URL = "http://localhost:8080/api/logs"

def send_log(level: str, message: str, system: str, service: str, **extra):
    """上报日志到 LogHub"""
    payload = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "level": level,
        "message": message,
        "system": system,
        "service": service,
        "extra": extra if extra else None,
    }
    try:
        resp = requests.post(LOGHUB_URL, json=payload, timeout=5)
        return resp.json()
    except Exception as e:
        print(f"Failed to send log: {e}")

# 使用示例
send_log("INFO", "User login successful", "user-system", "user-api", user_id=42)
send_log("ERROR", "Database timeout", "order-system", "order-api", retry=3, table="orders")
```

#### 方式二：logging Handler 集成

```python
import logging
import requests
from datetime import datetime, timezone


class LogHubHandler(logging.Handler):
    """Python logging 集成 Handler"""

    def __init__(self, url: str, system: str, service: str):
        super().__init__()
        self.url = url
        self.system = system
        self.service = service

    def emit(self, record: logging.LogRecord):
        try:
            # 提取 extra 字段（排除标准字段）
            standard_attrs = {
                "name", "msg", "args", "created", "relativeCreated",
                "levelname", "levelno", "pathname", "filename",
                "module", "exc_info", "exc_text", "stack_info",
                "lineno", "funcName", "msecs", "thread", "threadName",
                "processName", "process", "message",
            }
            extra = {
                k: v for k, v in record.__dict__.items()
                if k not in standard_attrs and not k.startswith("_")
            }

            payload = {
                "timestamp": datetime.fromtimestamp(record.created, tz=timezone.utc).isoformat(),
                "level": record.levelname,
                "message": self.format(record),
                "system": self.system,
                "service": self.service,
                "file_name": record.filename,
                "function_name": record.funcName,
                "line_number": record.lineno,
                "extra": extra if extra else None,
            }

            # 异步发送，不阻塞主线程
            requests.post(self.url, json=payload, timeout=3)
        except Exception:
            self.handleError(record)


# 使用示例
logger = logging.getLogger("my_app")
logger.setLevel(logging.DEBUG)

# 添加 LogHub Handler
handler = LogHubHandler(
    url="http://localhost:8080/api/logs",
    system="user-system",
    service="user-api",
)
logger.addHandler(handler)

# 也可以同时输出到控制台
console = logging.StreamHandler()
console.setFormatter(logging.Formatter("%(asctime)s %(levelname)s %(message)s"))
logger.addHandler(console)

# 使用
logger.info("User login", extra={"user_id": 42, "ip": "192.168.1.1"})
logger.error("Database connection failed", extra={"host": "db-master", "port": 5432})
```

#### 方式三：loguru 集成

```python
from loguru import logger
import requests
from datetime import datetime, timezone


def loghub_sink(message):
    """loguru 自定义 sink"""
    record = message.record
    payload = {
        "timestamp": record["time"].isoformat(),
        "level": record["level"].name,
        "message": record["message"],
        "system": "my-system",
        "service": "my-service",
        "file_name": record["file"].name,
        "function_name": record["function"],
        "line_number": record["line"],
    }
    try:
        requests.post("http://localhost:8080/api/logs", json=payload, timeout=3)
    except Exception:
        pass


logger.add(loghub_sink, level="INFO")

# 使用
logger.info("User {user_id} logged in", user_id=42)
logger.error("Payment failed for order {order_id}", order_id="ORD-001")
```

### Rust 接入

#### 方式一：reqwest 直接上报

```rust
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    system: String,
    service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<Value>,
}

pub async fn send_log(
    url: &str,
    level: &str,
    message: &str,
    system: &str,
    service: &str,
    extra: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        level: level.to_string(),
        message: message.to_string(),
        system: system.to_string(),
        service: service.to_string(),
        trace_id: None,
        request_id: None,
        extra,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/logs", url))
        .json(&entry)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Log send failed: {}", resp.status()).into())
    }
}

// 使用示例
#[tokio::main]
async fn main() {
    send_log(
        "http://localhost:8080",
        "INFO",
        "Server started on port 8080",
        "my-system",
        "my-service",
        Some(serde_json::json!({"port": 8080})),
    )
    .await
    .ok();
}
```

#### 方式二：tracing Subscriber 集成

```toml
# Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
reqwest = { version = "0.12", features = ["json"] }
chrono = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

```rust
use std::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;

/// LogHub tracing Layer
struct LogHubLayer {
    tx: mpsc::Sender<LogEntry>,
}

#[derive(serde::Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    system: String,
    service: String,
    extra: Option<serde_json::Value>,
}

impl LogHubLayer {
    fn new(url: &str, system: &str, service: &str) -> Self {
        let (tx, rx) = mpsc::channel::<LogEntry>();
        let url = url.to_string();
        let system = system.to_string();
        let service = service.to_string();

        // 后台线程批量发送
        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            while let Ok(entry) = rx.recv() {
                let _ = client
                    .post(format!("{}/api/logs", url))
                    .json(&entry)
                    .timeout(std::time::Duration::from_secs(3))
                    .send();
            }
        });

        Self { tx }
    }
}

impl<S: Subscriber> Layer<S> for LogHubLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let level = match event.metadata().level().as_str() {
            "TRACE" | "DEBUG" => "DEBUG",
            "INFO" => "INFO",
            "WARN" => "WARN",
            "ERROR" => "ERROR",
            _ => "INFO",
        };

        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: visitor.message,
            system: "my-system".to_string(),
            service: "my-service".to_string(),
            extra: if visitor.fields.is_empty() {
                None
            } else {
                Some(serde_json::Value::Object(visitor.fields))
            },
        };

        let _ = self.tx.send(entry);
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(format!("{:?}", value)),
            );
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.insert(
                field.name().to_string(),
                serde_json::Value::String(value.to_string()),
            );
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Bool(value),
        );
    }
}

// 使用示例
fn main() {
    use tracing_subscriber::prelude::*;

    let loghub_layer = LogHubLayer::new(
        "http://localhost:8080",
        "order-system",
        "order-api",
    );

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(loghub_layer)
        .init();

    tracing::info!("Server started on port {}", 8080);
    tracing::error!("Database connection failed: {}", "timeout");
}
```

### Golang 接入

```go
package loghub

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type LogEntry struct {
	Timestamp string                 `json:"timestamp"`
	Level     string                 `json:"level"`
	Message   string                 `json:"message"`
	System    string                 `json:"system"`
	Service   string                 `json:"service"`
	TraceID   string                 `json:"trace_id,omitempty"`
	RequestID string                 `json:"request_id,omitempty"`
	Extra     map[string]interface{} `json:"extra,omitempty"`
}

type LogHubClient struct {
	URL     string
	System  string
	Service string
	client  *http.Client
}

func NewLogHubClient(url, system, service string) *LogHubClient {
	return &LogHubClient{
		URL:     url,
		System:  system,
		Service: service,
		client: &http.Client{
			Timeout: 5 * time.Second,
		},
	}
}

func (c *LogHubClient) SendLog(level, message string, extra map[string]interface{}) error {
	entry := LogEntry{
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Level:     level,
		Message:   message,
		System:    c.System,
		Service:   c.Service,
		Extra:     extra,
	}

	data, err := json.Marshal(entry)
	if err != nil {
		return err
	}

	resp, err := c.client.Post(
		fmt.Sprintf("%s/api/logs", c.URL),
		"application/json",
		bytes.NewBuffer(data),
	)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("log send failed: %d", resp.StatusCode)
	}
	return nil
}

// 使用示例
func main() {
	client := NewLogHubClient("http://localhost:8080", "order-system", "order-api")

	client.SendLog("INFO", "Order created", map[string]interface{}{
		"order_id": "ORD-001",
		"user_id":  42,
	})

	client.SendLog("ERROR", "Payment timeout", map[string]interface{}{
		"order_id": "ORD-001",
		"timeout":  "30s",
	})
}
```

---

## WebSocket 实时日志

连接地址：`ws://localhost:8080/api/logs/ws`

### 发送过滤条件

```json
{
  "system": "order-system",
  "level": ["ERROR", "WARN"]
}
```

### 接收日志

服务端推送新日志时，客户端收到完整日志 JSON 对象。

### 前端示例

```javascript
const ws = new WebSocket(`ws://${window.location.host}/api/logs/ws`);

ws.onopen = () => {
  // 设置过滤条件
  ws.send(JSON.stringify({
    system: "order-system",
    level: ["ERROR", "WARN"],
  }));
};

ws.onmessage = (event) => {
  const log = JSON.parse(event.data);
  console.log(`[${log.level}] ${log.system}/${log.service}: ${log.message}`);
};
```

## 默认账号

| 用户名 | 密码 | 角色 |
|--------|------|------|
| admin | admin123 | admin（管理员） |

首次启动时自动创建，可通过环境变量 `LOG__ADMIN__USERNAME` 和 `LOG__ADMIN__PASSWORD` 修改。

## License

MIT
