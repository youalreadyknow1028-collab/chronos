import { invoke } from "@tauri-apps/api/core";
import type {
  HealthStatus,
  IngestRequest,
  IngestResponse,
  DiaryWriteRequest,
  DiaryWriteResponse,
  BudgetStatus,
  CronRunResult,
  EntryStatus,
  GraphData,
  TimelineEntry,
  InsightCard,
} from "../types/chronos";

// === Health ===
export async function getHealth(): Promise<HealthStatus> {
  return invoke<HealthStatus>("health_check");
}

// === Vault ===
export async function vaultIngest(
  path: string,
  title: string,
  tags: string[]
) {
  return invoke<{ note_id: string; status: string; created_at: number }>(
    "vault_ingest_file",
    { path, title, tags }
  );
}

export async function vaultSearch(query: string, limit: number = 20) {
  return invoke<{ notes: Array<{ note_id: string; title: string; snippet: string; score: number; created_at: number }>; total: number }>(
    "vault_search_notes",
    { query, limit }
  );
}

export async function vaultDelete(noteId: string) {
  return invoke<boolean>("vault_delete_note", { noteId });
}

// === Pipeline ===
export async function pipelineIngest(request: IngestRequest): Promise<IngestResponse> {
  return invoke<IngestResponse>("pipeline_ingest", { request });
}

export async function pipelineDiaryWrite(
  content: string,
  date: string
): Promise<DiaryWriteResponse> {
  return invoke<DiaryWriteResponse>("pipeline_diary_write", { content, date });
}

export async function pipelineBudgetStatus(): Promise<BudgetStatus> {
  return invoke<BudgetStatus>("pipeline_budget_status");
}

export async function pipelineStatus(ids: string[]): Promise<EntryStatus[]> {
  return invoke<EntryStatus[]>("pipeline_status", { ids });
}

export async function pipelineTriggerCron(): Promise<CronRunResult> {
  return invoke<CronRunResult>("pipeline_trigger_cron");
}

// === Graph (IPC) ===
export async function graphGetNodes(): Promise<GraphData> {
  return invoke<GraphData>("graph_get_nodes");
}

export async function graphGetTimeline(limit: number = 100): Promise<TimelineEntry[]> {
  return invoke<TimelineEntry[]>("graph_get_timeline", { limit });
}

// === Insights (IPC) ===
export async function insightsGetPending(): Promise<InsightCard[]> {
  return invoke<InsightCard[]>("insights_get_pending");
}

// === Settings ===
export async function saveApiKey(keyName: string, keyValue: string): Promise<void> {
  return invoke("save_api_key", { keyName, keyValue });
}

export async function loadApiKey(keyName: string): Promise<string> {
  return invoke<string>("load_api_key", { keyName });
}

export async function listSettings(): Promise<Array<[string, string]>> {
  return invoke<Array<[string, string]>>("list_settings");
}

export interface SyncResult {
  processed: number;
  errors: string[];
}

export async function triggerVaultSync(directoryPath: string): Promise<SyncResult> {
  return invoke<SyncResult>("trigger_vault_sync", { directoryPath });
}
