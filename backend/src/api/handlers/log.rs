use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Local};

use crate::models::log::IngestLogRequest;
use crate::repository::log_repo;
use crate::websocket::LogEvent;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct LogQueryString {
    /// Service name filter
    pub service: Option<String>,
    /// Log level filter (e.g. ERROR, WARN, INFO)
    pub level: Option<String>,
    /// Keyword search (ILIKE on message)
    pub keyword: Option<String>,
    /// File name filter (ILIKE)
    pub file_name: Option<String>,
    /// Function name filter (ILIKE)
    pub function_name: Option<String>,
    /// Start time (RFC3339)
    pub start_time: Option<String>,
    /// End time (RFC3339)
    pub end_time: Option<String>,
    /// Page number (default 1)
    pub page: Option<i64>,
    /// Page size (default 20, max 100)
    pub size: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/logs",
    tag = "logs",
    request_body = IngestLogRequest,
    responses(
        (status = 200, description = "Log ingested successfully", body = Value),
        (status = 500, description = "Internal server error", body = Value),
    )
)]
pub async fn ingest(
    State(state): State<AppState>,
    Json(req): Json<IngestLogRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let extra = req.extra.unwrap_or(serde_json::Value::Object(Default::default()));

    let log_id = log_repo::insert_log(
        &state.pool,
        req.timestamp,
        &req.level,
        &req.message,
        &req.service,
        req.trace_id.as_deref(),
        req.file_name.as_deref(),
        req.function_name.as_deref(),
        req.line_number,
        &extra,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert log: {e}");
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": 20001,
                "message": "Failed to ingest log",
            })),
        )
    })?;

    let event = LogEvent {
        id: log_id,
        log_time: req.timestamp,
        level: req.level.clone(),
        message: req.message.clone(),
        service: req.service.clone(),
        trace_id: req.trace_id.clone(),
        file_name: req.file_name.clone(),
        function_name: req.function_name.clone(),
        line_number: req.line_number,
        extra: extra.clone(),
    };

    if let Err(e) = state.ws_state.tx.send(event) {
        tracing::warn!("WebSocket broadcast failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "ok",
        "data": {
            "id": log_id,
        }
    })))
}

#[utoipa::path(
    get,
    path = "/api/logs",
    tag = "logs",
    params(LogQueryString),
    responses(
        (status = 200, description = "Log list with pagination", body = Value),
        (status = 400, description = "Invalid parameters", body = Value),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<LogQueryString>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let start_time = match params.start_time {
        Some(ref s) if !s.is_empty() => Some(DateTime::parse_from_rfc3339(s).map_err(|_| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": 10004,
                    "message": "Invalid start_time format, use RFC3339",
                })),
            )
        })?.with_timezone(&Local)),
        _ => None,
    };

    let end_time = match params.end_time {
        Some(ref s) if !s.is_empty() => Some(DateTime::parse_from_rfc3339(s).map_err(|_| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": 10004,
                    "message": "Invalid end_time format, use RFC3339",
                })),
            )
        })?.with_timezone(&Local)),
        _ => None,
    };

    let query_params = log_repo::LogQueryParams {
        service: params.service,
        level: params.level,
        keyword: params.keyword,
        file_name: params.file_name,
        function_name: params.function_name,
        start_time,
        end_time,
        page: params.page,
        size: params.size,
    };

    let (rows, total) = log_repo::query_logs(&state.pool, &query_params).await.map_err(|e| {
        tracing::error!("Failed to query logs: {e}");
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": 20001,
                "message": "Failed to query logs",
            })),
        )
    })?;

    let data: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "time": row.log_time,
                "ingest_time": row.ingest_time,
                "level": row.level,
                "message": row.message,
                "service": row.service,
                "trace_id": row.trace_id,
                "file_name": row.file_name,
                "function_name": row.function_name,
                "line_number": row.line_number,
                "extra": row.extra,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "code": 0,
        "message": "ok",
        "data": {
            "total": total,
            "data": data,
        }
    })))
}
