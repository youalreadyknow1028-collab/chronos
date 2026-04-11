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

/// Get all graph nodes and edges from KuzuDB for the WebGL GraphView
/// Returns nodes with position hints from KuzuDB spatial properties if available
#[tauri::command]
pub async fn graph_get_nodes() -> Result<GraphDataResponse, String> {
    // TODO: Wire to actual KuzuDB queries
    // For now, return a demo graph so the UI renders immediately
    Ok(GraphDataResponse {
        nodes: vec![
            GraphNodeResponse {
                id: "nexus-1".to_string(),
                label: "Nexus Core".to_string(),
                node_type: "Agent".to_string(),
                confidence: Some(1.0),
                tags: vec!["system".to_string()],
            },
            GraphNodeResponse {
                id: "kuzu-1".to_string(),
                label: "KuzuDB".to_string(),
                node_type: "Concept".to_string(),
                confidence: Some(0.95),
                tags: vec!["database".to_string(), "graph".to_string()],
            },
            GraphNodeResponse {
                id: "qdrant-1".to_string(),
                label: "Qdrant".to_string(),
                node_type: "Concept".to_string(),
                confidence: Some(0.95),
                tags: vec!["database".to_string(), "vector".to_string()],
            },
            GraphNodeResponse {
                id: "minimax-1".to_string(),
                label: "MiniMax AI".to_string(),
                node_type: "Concept".to_string(),
                confidence: Some(0.9),
                tags: vec!["ai".to_string(), "provider".to_string()],
            },
            GraphNodeResponse {
                id: "gemini-1".to_string(),
                label: "Gemini".to_string(),
                node_type: "Concept".to_string(),
                confidence: Some(0.85),
                tags: vec!["ai".to_string(), "provider".to_string()],
            },
            GraphNodeResponse {
                id: "thought-1".to_string(),
                label: "Bitemporal Versioning".to_string(),
                node_type: "Thought".to_string(),
                confidence: Some(0.8),
                tags: vec!["design".to_string(), "temporal".to_string()],
            },
            GraphNodeResponse {
                id: "thought-2".to_string(),
                label: "2PC Commit Protocol".to_string(),
                node_type: "Thought".to_string(),
                confidence: Some(0.85),
                tags: vec!["sync".to_string(), "protocol".to_string()],
            },
            GraphNodeResponse {
                id: "claim-1".to_string(),
                label: "Chronos is production-ready".to_string(),
                node_type: "Claim".to_string(),
                confidence: Some(0.75),
                tags: vec!["status".to_string()],
            },
        ],
        edges: vec![
            GraphEdgeResponse {
                source: "nexus-1".to_string(),
                target: "kuzu-1".to_string(),
                label: Some("USES".to_string()),
                edge_type: "USES".to_string(),
            },
            GraphEdgeResponse {
                source: "nexus-1".to_string(),
                target: "qdrant-1".to_string(),
                label: Some("USES".to_string()),
                edge_type: "USES".to_string(),
            },
            GraphEdgeResponse {
                source: "nexus-1".to_string(),
                target: "minimax-1".to_string(),
                label: Some("CALLS".to_string()),
                edge_type: "CALLS".to_string(),
            },
            GraphEdgeResponse {
                source: "minimax-1".to_string(),
                target: "gemini-1".to_string(),
                label: Some("FALLBACK".to_string()),
                edge_type: "FALLBACK".to_string(),
            },
            GraphEdgeResponse {
                source: "thought-1".to_string(),
                target: "claim-1".to_string(),
                label: Some("SUPPORTS".to_string()),
                edge_type: "SUPPORTS".to_string(),
            },
            GraphEdgeResponse {
                source: "thought-2".to_string(),
                target: "kuzu-1".to_string(),
                label: Some("ABOUT".to_string()),
                edge_type: "ABOUT".to_string(),
            },
        ],
    })
}

/// Get timeline entries from KuzuDB for the virtualized TimelineView
#[tauri::command]
pub async fn graph_get_timeline(limit: usize) -> Result<Vec<TimelineEntryResponse>, String> {
    let limit = if limit == 0 { 100 } else { limit };

    // TODO: Wire to actual KuzuDB TimelineEvent table query
    // SELECT * FROM TimelineEvent ORDER BY timestamp DESC LIMIT {limit}
    let now = chrono::Utc::now().timestamp_millis();

    // Demo timeline entries
    let entries = vec![
        TimelineEntryResponse {
            id: "evt-1".to_string(),
            event_type: "thought_added".to_string(),
            description: "Added: Bitemporal versioning enables time-travel queries across transaction and valid time dimensions".to_string(),
            timestamp: now - 300_000,
            entity_id: "thought-1".to_string(),
            tags: vec!["design".to_string(), "temporal".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-2".to_string(),
            event_type: "wiki_updated".to_string(),
            description: "Wiki updated: KuzuDB schema documentation — 12 node tables, 12 relationship tables".to_string(),
            timestamp: now - 600_000,
            entity_id: "kuzu-1".to_string(),
            tags: vec!["docs".to_string(), "schema".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-3".to_string(),
            event_type: "connection_found".to_string(),
            description: "Connection: MiniMax and Gemini share the same T3 retry budget pool".to_string(),
            timestamp: now - 900_000,
            entity_id: "minimax-1".to_string(),
            tags: vec!["ai".to_string(), "providers".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-4".to_string(),
            event_type: "claim_added".to_string(),
            description: "Claim added: 2PC commit prevents orphaned records — KuzuDB and Qdrant always stay in sync".to_string(),
            timestamp: now - 1200_000,
            entity_id: "claim-1".to_string(),
            tags: vec!["sync".to_string(), "2pc".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-5".to_string(),
            event_type: "thought_added".to_string(),
            description: "CallGuard RAII reserves tokens BEFORE the network call, confirms after success, rolls back on Drop".to_string(),
            timestamp: now - 1800_000,
            entity_id: "thought-2".to_string(),
            tags: vec!["budget".to_string(), "raii".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-6".to_string(),
            event_type: "wiki_updated".to_string(),
            description: "Wiki updated: Qdrant collection configs — vector_size=2560, shard_number=3, on_disk=true".to_string(),
            timestamp: now - 2400_000,
            entity_id: "qdrant-1".to_string(),
            tags: vec!["docs".to_string(), "qdrant".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-7".to_string(),
            event_type: "connection_found".to_string(),
            description: "Connection: BrainPulse dashboard connects to pipeline_budget_status every 15 seconds".to_string(),
            timestamp: now - 3000_000,
            entity_id: "nexus-1".to_string(),
            tags: vec!["ui".to_string(), "dashboard".to_string()],
        },
        TimelineEntryResponse {
            id: "evt-8".to_string(),
            event_type: "thought_added".to_string(),
            description: "The force-directed graph uses 60fps Canvas rendering with requestAnimationFrame".to_string(),
            timestamp: now - 3600_000,
            entity_id: "thought-3".to_string(),
            tags: vec!["graph".to_string(), "performance".to_string()],
        },
    ];

    Ok(entries.into_iter().take(limit).collect())
}
