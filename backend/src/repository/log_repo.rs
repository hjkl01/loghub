use anyhow::Result;
use chrono::{DateTime, Local};
use serde_json::Value;
use sqlx::PgPool;

#[allow(clippy::too_many_arguments)]
pub async fn insert_log(
    pool: &PgPool,
    log_time: DateTime<Local>,
    level: &str,
    message: &str,
    service: &str,
    trace_id: Option<&str>,
    file_name: Option<&str>,
    function_name: Option<&str>,
    line_number: Option<i32>,
    extra: &Value,
) -> Result<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO logs (log_time, ingest_time, level, message, service, trace_id, file_name, function_name, line_number, extra)
        VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id"#,
    )
    .bind(log_time)
    .bind(level)
    .bind(message)
    .bind(service)
    .bind(trace_id)
    .bind(file_name)
    .bind(function_name)
    .bind(line_number)
    .bind(extra)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn list_services(pool: &PgPool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"SELECT DISTINCT service FROM logs WHERE service IS NOT NULL AND service <> '' ORDER BY service"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct LogQueryRow {
    pub id: i64,
    pub log_time: DateTime<Local>,
    pub ingest_time: DateTime<Local>,
    pub level: String,
    pub message: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub file_name: Option<String>,
    pub function_name: Option<String>,
    pub line_number: Option<i32>,
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

    if params.service.is_some() {
        where_clauses.push(format!("service ILIKE ${idx}"));
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
    if params.file_name.is_some() {
        where_clauses.push(format!("file_name ILIKE ${idx}"));
        idx += 1;
    }
    if params.function_name.is_some() {
        where_clauses.push(format!("function_name ILIKE ${idx}"));
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
        r#"SELECT id, log_time, ingest_time, level, message, service, trace_id, file_name, function_name, line_number, extra
        FROM logs
        {}
        ORDER BY log_time DESC
        LIMIT ${} OFFSET ${}"#,
        where_sql, limit_idx, offset_idx,
    );

    let pattern = params.keyword.as_ref().map(|k| format!("%{}%", k));

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    let mut data_query = sqlx::query_as::<_, LogQueryRow>(&data_sql);

    if let Some(ref service) = params.service {
        let p = format!("%{}%", service);
        count_query = count_query.bind(p.clone());
        data_query = data_query.bind(p);
    }
    if let Some(ref level) = params.level {
        count_query = count_query.bind(level);
        data_query = data_query.bind(level);
    }
    if let Some(ref p) = pattern {
        count_query = count_query.bind(p);
        data_query = data_query.bind(p);
    }
    if let Some(ref file_name) = params.file_name {
        let p = format!("%{}%", file_name);
        count_query = count_query.bind(p.clone());
        data_query = data_query.bind(p);
    }
    if let Some(ref function_name) = params.function_name {
        let p = format!("%{}%", function_name);
        count_query = count_query.bind(p.clone());
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
    pub service: Option<String>,
    pub level: Option<String>,
    pub keyword: Option<String>,
    pub file_name: Option<String>,
    pub function_name: Option<String>,
    pub start_time: Option<DateTime<Local>>,
    pub end_time: Option<DateTime<Local>>,
    pub page: Option<i64>,
    pub size: Option<i64>,
}
