//! Insights commands — pending synthesis items for SynergyStream

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightCardResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(rename = "insight_type")]
    pub insight_type: String,
    pub confidence: f32,
    pub created_at: i64,
    pub tags: Vec<String>,
}

/// Get pending synthesis items for the SynergyStream
#[tauri::command]
pub async fn insights_get_pending() -> Result<Vec<InsightCardResponse>, String> {
    // TODO: Wire to actual KuzuDB + Qdrant query
    // SELECT FROM WikiEntry WHERE synthesis_status = 'pending' ORDER BY created_at DESC
    let now = chrono::Utc::now().timestamp_millis();

    Ok(vec![
        InsightCardResponse {
            id: "insight-1".to_string(),
            title: "Synthesis: Dual-Provider AI Routing".to_string(),
            content: "The MiniMax + Gemini fallback strategy provides resilience at the T3 tier. When MiniMax fails, the system automatically retries with Gemini without user intervention or additional budget burn.".to_string(),
            insight_type: "synthesis".to_string(),
            confidence: 0.92,
            created_at: now - 300_000,
            tags: vec!["ai".to_string(), "routing".to_string(), "synthesis".to_string()],
        },
        InsightCardResponse {
            id: "insight-2".to_string(),
            title: "Connection: KuzuDB ↔ Qdrant 2PC".to_string(),
            content: "The Two-Phase Commit ensures KuzuDB and Qdrant never drift. If Qdrant upsert fails, KuzuDB is rolled back to pre-write state. No orphaned records possible.".to_string(),
            insight_type: "connection".to_string(),
            confidence: 0.88,
            created_at: now - 600_000,
            tags: vec!["sync".to_string(), "2pc".to_string(), "database".to_string()],
        },
        InsightCardResponse {
            id: "insight-3".to_string(),
            title: "Prediction: Graph Growth Trajectory".to_string(),
            content: "Based on current brain dump frequency of ~8 entries/day, the knowledge graph will reach 1000 nodes within 90 days. Consider enabling automated synthesis at 500 nodes.".to_string(),
            insight_type: "prediction".to_string(),
            confidence: 0.71,
            created_at: now - 900_000,
            tags: vec!["prediction".to_string(), "graph".to_string(), "growth".to_string()],
        },
        InsightCardResponse {
            id: "insight-4".to_string(),
            title: "Contradiction: CallGuard Budget Model".to_string(),
            content: "Budget is reserved at call time (estimated tokens), then reconciled on confirm. If the call fails, Drop rolls back. This prevents the 'increment-before-call' bug but may slightly overestimate usage mid-flight.".to_string(),
            insight_type: "contradiction".to_string(),
            confidence: 0.83,
            created_at: now - 1200_000,
            tags: vec!["budget".to_string(), "raii".to_string(), "design".to_string()],
        },
    ])
}
