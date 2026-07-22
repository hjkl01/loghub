use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

pub async fn insert_log(
    pool: &PgPool,
    log_time: DateTime<Utc>,
    level: &str,
    message: &str,
    system: &str,
    service: &str,
    trace_id: Option<&str>,
    request_id: Option<&str>,
    extra: &Value,
) -> Result<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO logs (log_time, ingest_time, level, message, system, service, trace_id, request_id, extra)
        VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8)
        RETURNING id"#,
    )
    .bind(log_time)
    .bind(level)
    .bind(message)
    .bind(system)
    .bind(service)
    .bind(trace_id)
    .bind(request_id)
    .bind(extra)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct LogQueryRow {
    pub id: i64,
    pub log_time: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub system: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub request_id: Option<String>,
    pub extra: Value,
}

pub async fn query_logs(
    pool: &PgPool,
    params: &LogQueryParams,
) -> Result<(Vec<LogQueryRow>, i64)> {
    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * size;

    let mut where_clauses: Vec<String> = Vec::new();
    let mut idx = 1u32;

    if params.system.is_some() {
        where_clauses.push(format!("system = ${idx}"));
        idx += 1;
    }
    if params.service.is_some() {
        where_clauses.push(format!("service = ${idx}"));
        idx += 1;
    }
    if params.level.is_some() {
        where_clauses.push(format!("level = ${idx}"));
        idx += 1;
    }
    if params.keyword.is_some() {
        where_clauses.push(format!("message ILIKE ${idx}"));
        idx += 1;
    }
    if params.start_time.is_some() {
        where_clauses.push(format!("log_time >= ${idx}"));
        idx += 1;
    }
    if params.end_time.is_some() {
        where_clauses.push(format!("log_time <= ${idx}"));
        idx += 1;
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM logs {}",
        where_sql,
    );

    let limit_idx = idx;
    let offset_idx = idx + 1;

    let data_sql = format!(
        r#"SELECT id, log_time, level, message, system, service, trace_id, request_id, extra
        FROM logs
        {}
        ORDER BY log_time DESC
        LIMIT ${} OFFSET ${}"#,
        where_sql, limit_idx, offset_idx,
    );

    let pattern = params.keyword.as_ref().map(|k| format!("%{}%", k));

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    let mut data_query = sqlx::query_as::<_, LogQueryRow>(&data_sql);

    if let Some(ref system) = params.system {
        count_query = count_query.bind(system);
        data_query = data_query.bind(system);
    }
    if let Some(ref service) = params.service {
        count_query = count_query.bind(service);
        data_query = data_query.bind(service);
    }
    if let Some(ref level) = params.level {
        count_query = count_query.bind(level);
        data_query = data_query.bind(level);
    }
    if let Some(ref p) = pattern {
        count_query = count_query.bind(p);
        data_query = data_query.bind(p);
    }
    if let Some(start_time) = params.start_time {
        count_query = count_query.bind(start_time);
        data_query = data_query.bind(start_time);
    }
    if let Some(end_time) = params.end_time {
        count_query = count_query.bind(end_time);
        data_query = data_query.bind(end_time);
    }

    data_query = data_query.bind(size).bind(offset);

    let total = count_query.fetch_one(pool).await.unwrap_or(0);
    let rows = data_query.fetch_all(pool).await?;

    Ok((rows, total))
}

#[derive(Debug, serde::Deserialize)]
pub struct LogQueryParams {
    pub system: Option<String>,
    pub service: Option<String>,
    pub level: Option<String>,
    pub keyword: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub size: Option<i64>,
}
