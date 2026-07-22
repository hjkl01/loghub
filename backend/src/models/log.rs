use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IngestLogRequest {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    /// Service name (e.g. "order-api", "payment-worker")
    pub service: String,
    pub trace_id: Option<String>,
    /// Source file name
    pub file_name: Option<String>,
    /// Source function name
    pub function_name: Option<String>,
    /// Source line number
    pub line_number: Option<i32>,
    pub extra: Option<Value>,
}
