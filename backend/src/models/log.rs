use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IngestLogRequest {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    /// System name (e.g. "order-system", "payment-service")
    pub system: String,
    /// Service name (e.g. "order-api", "payment-worker")
    pub service: String,
    pub trace_id: Option<String>,
    pub request_id: Option<String>,
    pub extra: Option<Value>,
}
