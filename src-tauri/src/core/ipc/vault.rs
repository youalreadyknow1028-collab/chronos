use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultIngestResponse {
    pub note_id: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultSearchResponse {
    pub notes: Vec<VaultNote>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultNote {
    pub note_id: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub created_at: i64,
}

/// Ingest a file/note into the Chronos vault.
/// Two-Phase Commit: Phase 1 → KuzuDB (prepare), Phase 2 → Qdrant (commit or rollback)
/// See EPIC_1 §VAULT_2PC for full protocol
#[tauri::command]
pub async fn vault_ingest_file(
    _path: String,
    _title: String,
    _tags: Vec<String>,
) -> Result<VaultIngestResponse, String> {
    let note_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    // TODO: Implement §VAULT_2PC two-phase commit
    // Phase 1: Serialize → KuzuDB (prepare)
    // Phase 2: Qdrant upsert (commit or rollback)
    Ok(VaultIngestResponse {
        note_id,
        status: "prepared".to_string(),
        created_at: now,
    })
}

#[tauri::command]
pub async fn vault_search_notes(
    _query: String,
    _limit: usize,
) -> Result<VaultSearchResponse, String> {
    // TODO: Implement Qdrant vector search
    Ok(VaultSearchResponse { notes: vec![], total: 0 })
}

#[tauri::command]
pub async fn vault_delete_note(note_id: String) -> Result<bool, String> {
    // TODO: Soft delete (set valid_time_end = now, per EPIC_3 §4.2)
    let _ = note_id;
    Ok(true)
}
