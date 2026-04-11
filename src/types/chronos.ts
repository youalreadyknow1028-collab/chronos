// Chronos TypeScript types — mirrors Rust IPC types

export interface HealthStatus {
  status: string;
  kuzu_connected: boolean;
  qdrant_connected: boolean;
  timestamp: number;
}

export interface VaultIngestRequest {
  path: string;
  title: string;
  tags: string[];
}

export interface VaultIngestResponse {
  note_id: string;
  status: string;
  created_at: number;
}

export interface VaultNote {
  note_id: string;
  title: string;
  snippet: string;
  score: number;
  created_at: number;
}

export interface VaultSearchResponse {
  notes: VaultNote[];
  total: number;
}

// Pipeline types
export interface IngestRequest {
  content: string;
  source: 'mind_dump' | 'url' | 'file' | 'voice';
  tags: string[];
}

export interface IngestResponse {
  note_id: string;
  wiki_entry_id: string;
  tier_used: string;
  provider_used: string;
  status: string;
}

export interface DiaryWriteRequest {
  content: string;
  date: string;
}

export interface DiaryWriteResponse {
  note_id: string;
  status: string;
}

export interface BudgetStatus {
  minimax_calls: number;
  minimax_calls_limit: number;
  minimax_tokens: number;
  minimax_tokens_limit: number;
  gemini_calls: number;
  gemini_calls_limit: number;
  gemini_tokens: number;
  gemini_tokens_limit: number;
}

export interface EntryStatus {
  id: string;
  status: string;
  tier_done: string;
}

export interface CronRunResult {
  entries_processed: number;
  synthesis_count: number;
  errors: string[];
}

// Graph types
export interface GraphNode {
  id: string;
  label: string;
  type: 'Thought' | 'Claim' | 'WikiEntry' | 'Concept' | 'Agent';
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
  confidence?: number;
  tags?: string[];
}

export interface GraphEdge {
  source: string;
  target: string;
  label?: string;
  type: string;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

// Timeline types
export interface TimelineEntry {
  id: string;
  event_type: string;
  description: string;
  timestamp: number;
  entity_id: string;
  tags: string[];
}

// Insight card types
export interface InsightCard {
  id: string;
  title: string;
  content: string;
  insight_type: 'synthesis' | 'prediction' | 'contradiction' | 'connection';
  confidence: number;
  created_at: number;
  tags: string[];
}
