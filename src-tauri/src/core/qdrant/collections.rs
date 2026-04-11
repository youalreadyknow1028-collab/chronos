//! Qdrant Collection Configurations (stub)
//! See EPIC_3 §3.2 + Appendix F (FIX2 scaling config)

pub const VECTOR_SIZE: u64 = 2560;
pub const SHARD_NUMBER: u32 = 3;

pub fn indexed_payload_fields() -> Vec<String> {
    vec![
        "entity_type".to_string(),
        "tx_time_start".to_string(),
        "valid_time_start".to_string(),
        "created_by".to_string(),
        "record_type".to_string(),
        "sync_status".to_string(),
    ]
}
