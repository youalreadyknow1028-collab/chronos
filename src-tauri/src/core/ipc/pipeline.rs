//! Pipeline IPC commands — wraps core::pipeline with Tauri command handlers

use crate::core::minimax::{TierRouter, BudgetStatus};
use crate::core::pipeline::{
    IngestRequest, DiaryWriteRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static ROUTER: OnceLock<TierRouter> = OnceLock::new();

fn get_router() -> &'static TierRouter {
    ROUTER.get_or_init(|| TierRouter::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineIngestRequest {
    pub content: String,
    pub source: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineIngestResponse {
    pub note_id: String,
    pub wiki_entry_id: String,
    pub tier_used: String,
    pub provider_used: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDiaryWriteResponse {
    pub note_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatusResponse {
    pub id: String,
    pub status: String,
    pub tier_done: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCronResult {
    pub entries_processed: u32,
    pub synthesis_count: u32,
    pub errors: Vec<String>,
}

/// Main ingestion pipeline — mind dump → T1+T2+T3 → wiki entry → 2PC
#[tauri::command]
pub async fn pipeline_ingest(request: PipelineIngestRequest) -> Result<PipelineIngestResponse, String> {
    let router = get_router();
    let req = IngestRequest {
        content: request.content,
        source: request.source,
        tags: request.tags,
    };

    let result = crate::core::pipeline::pipeline_ingest(req, router)
        .await
        .map_err(|e| e.message)?
        .ok_or_else(|| "Pipeline ingest returned None (embedding dim mismatch or other graceful failure)".to_string())?;

    Ok(PipelineIngestResponse {
        note_id: result.note_id,
        wiki_entry_id: result.wiki_entry_id,
        tier_used: result.tier_used,
        provider_used: result.provider_used,
        status: result.status,
    })
}

/// Write to daily diary — raw immutable input
#[tauri::command]
pub async fn pipeline_diary_write(content: String, date: String) -> Result<PipelineDiaryWriteResponse, String> {
    let req = DiaryWriteRequest { content, date };
    let result = crate::core::pipeline::pipeline_diary_write(req)
        .await
        .map_err(|e| e.message)?;

    Ok(PipelineDiaryWriteResponse {
        note_id: result.note_id,
        status: result.status,
    })
}

/// Get current budget status for BrainPulse dashboard
#[tauri::command]
pub async fn pipeline_budget_status() -> Result<BudgetStatus, String> {
    Ok(get_router().budget_status().await)
}

/// Check status of specific entries
#[tauri::command]
pub async fn pipeline_status(ids: Vec<String>) -> Result<Vec<PipelineStatusResponse>, String> {
    let results = crate::core::pipeline::pipeline_status(ids)
        .await
        .map_err(|e| e.message)?;

    Ok(results.into_iter().map(|r| PipelineStatusResponse {
        id: r.id,
        status: r.status,
        tier_done: r.tier_done,
    }).collect())
}

/// Trigger synthesis cron cycle
#[tauri::command]
pub async fn pipeline_trigger_cron() -> Result<PipelineCronResult, String> {
    let router = get_router();
    let result = crate::core::pipeline::pipeline_trigger_cron(router)
        .await
        .map_err(|e| e.message)?;

    Ok(PipelineCronResult {
        entries_processed: result.entries_processed,
        synthesis_count: result.synthesis_count,
        errors: result.errors,
    })
}
