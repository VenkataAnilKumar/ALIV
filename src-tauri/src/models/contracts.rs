use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolderInput {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionRequest {
    pub workspace_path: String,
    pub incremental: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRequest {
    pub query: String,
    pub top_k: usize,
    pub conversation_id: Option<String>,
    pub workspace_path: Option<String>,
    pub token_budget: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHit {
    pub file_id: String,
    pub source_path: String,
    pub chunk_index: usize,
    pub page_ref: Option<String>,
    pub score: i64,
    pub content_preview: String,
    pub parser_confidence: Option<f64>,
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResponse {
    pub command: String,
    pub status: String,
    pub query: String,
    pub top_k: usize,
    pub total_hits: usize,
    pub assembled_prompt: Option<String>,
    pub prompt_token_estimate: Option<usize>,
    pub hits: Vec<RetrievalHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRequest {
    pub query: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendationRequest {
    pub workspace_path: String,
    pub batch_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRiskRequest {
    pub recommendation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionApprovalRequest {
    pub recommendation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecutionRequest {
    pub batch_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoBatchRequest {
    pub execution_batch_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRequest {
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRequest {
    pub corpus_manifest: String,
    pub labels_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetricsRequest {
    pub run_id: String,
}

// ── Memory and Long Context ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartConversationRequest {
    pub workspace_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStub {
    pub id: String,
    pub title: String,
    pub turn_count: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurnView {
    pub id: String,
    pub role: String,
    pub content: String,
    pub citations_json: Option<String>,
    pub token_count: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConversationRequest {
    pub conversation_id: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConversationsRequest {
    pub workspace_path: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMemoryFactRequest {
    pub workspace_path: String,
    pub content: String,
    pub citation_json: Option<String>,
    pub scope: String, // "session" | "week" | "permanent"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFactView {
    pub id: String,
    pub content: String,
    pub citation_json: Option<String>,
    pub scope: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMemoryFactRequest {
    pub fact_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDocumentSummaryRequest {
    pub file_id: String,
    pub granularity: String, // "document" | "section"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLongContextRequest {
    pub query: String,
    pub conversation_id: String,
    pub workspace_path: String,
    pub token_budget: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongContextResponse {
    pub conversation_id: String,
    pub assembled_prompt: String,
    pub total_token_estimate: usize,
    pub token_budget: usize,
    pub overflow_dropped: usize,
}

// ── General ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAck {
    pub command: String,
    pub status: String,
    pub detail: String,
}
