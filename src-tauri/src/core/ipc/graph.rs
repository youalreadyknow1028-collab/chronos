//! Graph commands — KuzuDB query for graph view and timeline
//! Wires GraphView and TimelineView to KuzuDB

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeResponse {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub confidence: Option<f32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeResponse {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub edge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDataResponse {
    pub nodes: Vec<GraphNodeResponse>,
    pub edges: Vec<GraphEdgeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntryResponse {
    pub id: String,
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub description: String,
    pub timestamp: i64,
    #[serde(rename = "entity_id")]
    pub entity_id: String,
    pub tags: Vec<String>,
}

/// Get all graph nodes and edges from KuzuDB for the GraphView
/// Returns empty graph when KuzuDB is unavailable
#[tauri::command]
pub async fn graph_get_nodes() -> Result<GraphDataResponse, String> {
    // TODO: Wire to actual KuzuDB queries when kuzu crate is available
    // Temporary: return empty graph — frontend shows empty state gracefully
    tracing::warn!("graph_get_nodes: KuzuDB not connected, returning empty graph");
    Ok(GraphDataResponse {
        nodes: vec![],
        edges: vec![],
    })
}

/// Get timeline entries from KuzuDB for the TimelineView
/// Returns empty timeline when KuzuDB is unavailable
#[tauri::command]
pub async fn graph_get_timeline(limit: usize) -> Result<Vec<TimelineEntryResponse>, String> {
    let _limit = if limit == 0 { 100 } else { limit };
    // TODO: Wire to actual KuzuDB TimelineEvent table query
    // SELECT * FROM TimelineEvent ORDER BY timestamp DESC LIMIT {limit}
    tracing::warn!("graph_get_timeline: KuzuDB not connected, returning empty timeline");
    Ok(vec![])
}
