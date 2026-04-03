export interface WorkspaceFolderInput { path: string }
export interface IngestionRequest { workspace_path: string; incremental: boolean }

export interface RetrievalRequest {
  query: string
  top_k: number
  conversation_id?: string
  workspace_path?: string
  token_budget?: number
}
export interface RetrievalHit {
  file_id: string
  source_path: string
  chunk_index: number
  page_ref?: string
  score: number
  content_preview: string
  parser_confidence?: number
  warning_count: number
}
export interface RetrievalResponse {
  command: string
  status: string
  query: string
  top_k: number
  total_hits: number
  assembled_prompt?: string
  prompt_token_estimate?: number
  hits: RetrievalHit[]
}

export interface ExtractionRequest { query: string; template: string }

export interface ActionRecommendationRequest { workspace_path: string; batch_limit: number }
export interface ActionRiskRequest { recommendation_ids: string[] }
export interface ActionApprovalRequest { recommendation_ids: string[] }
export interface ActionExecutionRequest { batch_id: string }
export interface UndoBatchRequest { execution_batch_id: string }
export interface AuditLogRequest { limit: number }

export interface ActionRecommendationView {
  id: string
  file_id: string
  recommendation_type: string
  source_path: string
  proposed_path: string
  confidence: number
  evidence: string[]
  risk_level: 'low' | 'medium' | 'high'
  risk_flags: string[]
  created_at: string
}
export interface ActionPreviewList {
  command: string
  status: string
  workspace_path: string
  total: number
  items: ActionRecommendationView[]
}
export interface ActionExecutionResult {
  command: string
  status: string
  batch_id: string
  total: number
  succeeded: number
  failed: number
  errors: string[]
}
export interface UndoResult {
  command: string
  status: string
  batch_id: string
  reversed: number
  errors: string[]
}
export interface AuditLogEntry {
  execution_id: string
  recommendation_id: string
  batch_id: string
  source_path: string
  proposed_path: string
  recommendation_type: string
  status: string
  executed_at: string
}
export interface AuditLogResponse {
  command: string
  status: string
  total: number
  entries: AuditLogEntry[]
}

export interface StartConversationRequest { workspace_path: string }
export interface ConversationStub { id: string; title: string; turn_count: number; updated_at: string }
export interface ConversationTurnView {
  id: string
  role: string
  content: string
  citations_json?: string
  token_count?: number
  created_at: string
}
export interface GetConversationRequest { conversation_id: string; limit: number }
export interface ListConversationsRequest { workspace_path: string; limit: number }
export interface SaveMemoryFactRequest {
  workspace_path: string
  content: string
  citation_json?: string
  scope: 'session' | 'week' | 'permanent'
}
export interface MemoryFactView {
  id: string
  content: string
  citation_json?: string
  scope: string
  created_at: string
  expires_at?: string
}
export interface DeleteMemoryFactRequest { fact_id: string }
export interface BuildLongContextRequest {
  query: string
  conversation_id: string
  workspace_path: string
  token_budget?: number
}
export interface LongContextResponse {
  conversation_id: string
  assembled_prompt: string
  total_token_estimate: number
  token_budget: number
  overflow_dropped: number
}

export interface BenchmarkRequest { corpus_manifest: string; labels_version: string }
export interface RunMetricsRequest { run_id: string }

export interface CommandAck { command: string; status: string; detail: string }
export interface RuntimeSnapshot {
  workspace_count: number
  command_count: number
  preview_queue_count: number
  benchmark_run_count: number
  last_ingestion_target?: string
}

export interface DiagnosticsReport {
  aliv_version: string
  platform: { os: string; arch: string }
  ollama: { reachable: boolean; endpoint: string }
  db: {
    workspace_folders: number
    files: number
    chunks: number
    embeddings: number
    memory_facts: number
    action_recommendations: number
    benchmark_runs: number
  }
  runtime: {
    workspace_count: number
    command_count: number
    preview_queue_count: number
    benchmark_run_count: number
    last_ingestion_target?: string
  }
  logs: {
    directory: string
    active_file?: string
    tail: string
  }
  generated_at: string
}

export interface IngestionTelemetry {
  runs: number
  discovered_total: number
  indexed_total: number
  skipped_total: number
  parsed_total: number
  failed_total: number
  files: number
  parsed: number
  chunks: number
  embeddings: number
  avg_parser_confidence: number
  last_run?: string
  last_workspace?: string
  last_mode?: string
  last_created_at?: string
}

export interface ExtractionFieldRow {
  field: string
  value: string
  confidence: number | null
  citations: unknown[]
}

export interface BenchmarkMetricSummary {
  run_id: string
  status: string
  detail: string
}

export interface BenchmarkCheck {
  name: string
  threshold: number
  actual: number
  pass: boolean
  run_id: string
}

export interface BenchmarkIngestionMetrics {
  status: string
  checks: BenchmarkCheck[]
}

export interface BenchmarkRetrievalMetrics {
  status: string
  threshold: number
  actual: number
  passed: number
  total: number
  query_latency_p50_sec: number
  query_latency_p95_sec: number
  query_latency_p95_threshold_sec: number
  latency_pass: boolean
  failed_ids: string[]
}

export interface BenchmarkExtractionMetrics {
  status: string
  threshold: number
  actual: number
  correct: number
  total: number
  skipped: number
  failed_ids: string[]
}

export interface BenchmarkSuiteRun {
  timestamp_utc: string
  metrics: {
    ingestion: BenchmarkIngestionMetrics
    retrieval: BenchmarkRetrievalMetrics
    extraction: BenchmarkExtractionMetrics
  }
  suite: {
    status: string
    checks: Record<string, boolean>
  }
  trend?: {
    note?: string
    retrieval_top5_delta?: number
    extraction_precision_delta?: number
    query_latency_p95_delta_sec?: number
  }
  run_index: number
}

export interface BenchmarkSuiteReport {
  status: string
  runs: BenchmarkSuiteRun[]
  phase4_exit: {
    status: string
    reason: string
    pass: boolean
  }
  result_files?: {
    latest?: string
    stamped?: string
  }
}
