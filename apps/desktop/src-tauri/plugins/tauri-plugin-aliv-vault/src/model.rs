use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Matter {
    pub id: String,
    pub name: String,
    pub client: String,
    /// "legal" | "finance"
    pub kind: String,
    /// "local_only" | "hybrid"
    pub policy: String,
    pub created_at: String,
    pub document_count: i64,
    pub pending_actions: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatterInput {
    pub name: String,
    #[serde(default)]
    pub client: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub matter_id: String,
    pub path: String,
    pub file_name: String,
    pub ext: String,
    pub size: i64,
    /// "indexed" | "skipped" | "failed"
    pub status: String,
    pub error: Option<String>,
    pub doc_type: Option<String>,
    pub char_count: i64,
    pub chunk_count: i64,
    pub ingested_at: String,
    /// Set when the text was read by OCR: the mean word confidence, 0 to 1.
    pub ocr_confidence: Option<f64>,
}

/// One extracted value, always tied back to the passage it came from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub chunk_ordinal: Option<i64>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridRow {
    pub document: Document,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub chunk_id: i64,
    pub document_id: String,
    pub file_name: String,
    pub path: String,
    pub ordinal: i64,
    pub text: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestReport {
    pub scanned: usize,
    pub indexed: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub failed: usize,
    /// Of `indexed`, how many were read by OCR.
    pub ocr: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub matter_id: String,
    pub document_id: String,
    /// "rename"
    pub kind: String,
    pub from_path: String,
    pub to_path: String,
    pub reason: String,
    /// "low" | "medium" | "high"
    pub risk: String,
    /// "proposed" | "approved" | "rejected" | "executed" | "undone" | "failed"
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: i64,
    pub matter_id: Option<String>,
    pub ts: String,
    pub kind: String,
    pub summary: String,
    pub detail: serde_json::Value,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCallRequest {
    pub matter_id: String,
    pub provider: String,
    pub model: String,
    /// "local" | "cloud"
    pub location: String,
    pub chars_sent: i64,
    pub source_count: i64,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCallDecision {
    pub allowed: bool,
    pub reason: String,
    pub audit_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditVerification {
    pub ok: bool,
    pub entries: usize,
    pub first_broken_id: Option<i64>,
}
