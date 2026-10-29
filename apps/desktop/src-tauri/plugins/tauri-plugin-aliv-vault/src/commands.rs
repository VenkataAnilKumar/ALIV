//! Tauri commands. Slow work (ingestion, export) runs on a blocking thread so
//! the UI stays responsive.

use crate::error::{Result, VaultError};
use crate::model::*;
use crate::vault::Vault;
use std::sync::Arc;
use tauri::State;

type V<'a> = State<'a, Arc<Vault>>;

async fn blocking<T: Send + 'static>(vault: &Arc<Vault>, f: impl FnOnce(&Vault) -> Result<T> + Send + 'static) -> Result<T> {
    let v = vault.clone();
    tauri::async_runtime::spawn_blocking(move || f(&v))
        .await
        .map_err(|e| VaultError::Invalid(format!("background task failed: {e}")))?
}

#[tauri::command]
pub async fn list_matters(vault: V<'_>) -> Result<Vec<Matter>> {
    vault.store.list_matters()
}

#[tauri::command]
pub async fn create_matter(vault: V<'_>, input: MatterInput) -> Result<Matter> {
    vault.store.create_matter(&input)
}

#[tauri::command]
pub async fn update_matter(vault: V<'_>, id: String, input: MatterInput) -> Result<Matter> {
    vault.store.update_matter(&id, &input)
}

#[tauri::command]
pub async fn delete_matter(vault: V<'_>, id: String) -> Result<()> {
    vault.store.delete_matter(&id)
}

#[tauri::command]
pub async fn ingest_paths(vault: V<'_>, matter_id: String, paths: Vec<String>) -> Result<IngestReport> {
    blocking(&vault, move |v| v.ingest_paths(&matter_id, &paths)).await
}

#[tauri::command]
pub async fn list_documents(vault: V<'_>, matter_id: String) -> Result<Vec<Document>> {
    vault.store.list_documents(&matter_id)
}

/// Keyword search, fused with meaning-based search when the app sends the
/// question's embedding.
#[tauri::command]
pub async fn search(
    vault: V<'_>,
    matter_id: String,
    query: String,
    limit: Option<usize>,
    query_vector: Option<Vec<f32>>,
) -> Result<Vec<SearchHit>> {
    let limit = limit.unwrap_or(8).clamp(1, 50);
    blocking(&vault, move |v| v.store.hybrid_search(&matter_id, &query, query_vector.as_deref(), limit)).await
}

#[tauri::command]
pub async fn pending_embeddings(vault: V<'_>, matter_id: String, limit: Option<usize>) -> Result<Vec<PendingPassage>> {
    vault.store.pending_embeddings(&matter_id, limit.unwrap_or(64).clamp(1, 512))
}

#[tauri::command]
pub async fn store_embeddings(vault: V<'_>, embeddings: Vec<PassageEmbedding>) -> Result<usize> {
    blocking(&vault, move |v| v.store.store_embeddings(&embeddings)).await
}

#[tauri::command]
pub async fn vault_diagnostics(vault: V<'_>) -> Result<VaultDiagnostics> {
    blocking(&vault, |v| v.diagnostics()).await
}

#[tauri::command]
pub async fn embedding_status(vault: V<'_>, matter_id: String) -> Result<EmbeddingStatus> {
    vault.store.embedding_status(&matter_id)
}

#[tauri::command]
pub async fn review_grid(vault: V<'_>, matter_id: String) -> Result<Vec<GridRow>> {
    vault.store.review_grid(&matter_id)
}

#[tauri::command]
pub async fn propose_renames(vault: V<'_>, matter_id: String) -> Result<Vec<Action>> {
    blocking(&vault, move |v| v.propose_renames(&matter_id)).await
}

#[tauri::command]
pub async fn list_actions(vault: V<'_>, matter_id: String) -> Result<Vec<Action>> {
    vault.store.list_actions(&matter_id)
}

#[tauri::command]
pub async fn set_action_status(vault: V<'_>, id: String, status: String) -> Result<Action> {
    vault.set_action_status(&id, &status)
}

#[tauri::command]
pub async fn execute_approved(vault: V<'_>, matter_id: String) -> Result<Vec<Action>> {
    blocking(&vault, move |v| v.execute_approved(&matter_id)).await
}

#[tauri::command]
pub async fn undo_action(vault: V<'_>, id: String) -> Result<Action> {
    vault.undo_action(&id)
}

#[tauri::command]
pub async fn authorize_model_call(vault: V<'_>, request: ModelCallRequest) -> Result<ModelCallDecision> {
    vault.authorize_model_call(&request)
}

#[tauri::command]
pub async fn list_audit(vault: V<'_>, matter_id: Option<String>, limit: Option<usize>) -> Result<Vec<AuditEntry>> {
    vault.store.list_audit(matter_id.as_deref(), limit.unwrap_or(200).clamp(1, 2000))
}

#[tauri::command]
pub async fn verify_audit(vault: V<'_>) -> Result<AuditVerification> {
    vault.store.verify_audit()
}

#[tauri::command]
pub async fn export_register_csv(vault: V<'_>, matter_id: String, dest: String) -> Result<usize> {
    blocking(&vault, move |v| v.export_register_csv(&matter_id, std::path::Path::new(&dest))).await
}
