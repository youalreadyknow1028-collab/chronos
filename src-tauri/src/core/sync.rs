//! Two-Phase Commit — Atomic Sync (stub)
//! See EPIC_3 §4.1 + §VAULT_2PC
//!
//! NOTE: kuzu/qdrant_client not in Cargo.toml — stub for compile check.
//! In production, add crates and uncomment the real implementation.

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPoint {
    pub id: String,
    pub entity_type: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub tags: Vec<String>,
    pub tx_time_start: i64,
    pub tx_time_end: i64,
    pub valid_time_start: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncPhase {
    Phase1Prepare,
    Phase2aQdrantCommit,
    Phase2bKuzuCommit,
    Committed,
    RolledBack,
}

/// Perform two-phase commit for a single note (stub)
/// In production: real 2PC with kuzu + qdrant
pub async fn sync_note_2pc(
    _point: SyncPoint,
) -> Result<(SyncPhase, String), String> {
    info!("[2PC stub] sync_note_2pc called");
    Ok((SyncPhase::Committed, _point.id))
}

/// Delete operation — also uses 2PC (stub)
pub async fn delete_note_2pc(
    _note_id: &str,
    _entity_type: &str,
) -> Result<(), String> {
    Ok(())
}
