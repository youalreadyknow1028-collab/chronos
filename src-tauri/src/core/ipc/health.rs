use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub kuzu_connected: bool,
    pub qdrant_connected: bool,
    pub timestamp: i64,
}

#[tauri::command]
pub async fn health_check() -> Result<HealthResponse, String> {
    let now = chrono::Utc::now().timestamp_millis();
    Ok(HealthResponse {
        status: "ok".to_string(),
        kuzu_connected: false,
        qdrant_connected: false,
        timestamp: now,
    })
}
