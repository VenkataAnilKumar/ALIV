use tauri::State;

use serde_json::json;

use crate::{
    core::retrieval::RetrievalService,
    infra::{db::Database, python_bridge::PythonBridge},
    models::contracts::{CommandAck, ExtractionRequest},
    state::AppState,
};

#[tauri::command]
pub fn run_extraction(
    state: State<'_, AppState>,
    database: State<'_, Database>,
    bridge: State<'_, PythonBridge>,
    request: ExtractionRequest,
) -> Result<CommandAck, String> {
    state.record_command("run_extraction");

    let query = request.query.trim();
    let template = request.template.trim();
    if query.is_empty() {
        return Err("extraction query cannot be empty".into());
    }
    if template.is_empty() {
        return Err("extraction template cannot be empty".into());
    }

    let retrieval = RetrievalService;
    let candidate_k = 12_usize;

    let lexical_hits = database
        .search_chunks(query, candidate_k.saturating_mul(2))
        .map_err(|err| err.to_string())?;

    let semantic_hits = match bridge.embed_documents(&[query.to_string()]) {
        Ok(embedding) => {
            let Some(query_vector) = embedding.vectors.first() else {
                return Err("embedding worker returned no vectors".into());
            };
            database
                .search_embeddings(query_vector, candidate_k.saturating_mul(2))
                .unwrap_or_default()
        }
        Err(_) => Vec::new(),
    };

    let hits = retrieval.merge_hybrid_hits(lexical_hits, semantic_hits, candidate_k);
    let chunk_payload: Vec<serde_json::Value> = hits
        .iter()
        .map(|hit| {
            json!({
                "chunk_id": format!("{}#{}", hit.file_id, hit.chunk_index),
                "file_id": hit.file_id,
                "source_path": hit.source_path,
                "chunk_index": hit.chunk_index,
                "page_ref": hit.page_ref,
                "content": hit.content_preview,
                "score": hit.score,
            })
        })
        .collect();

    let extraction = bridge
        .extract_fields(query, template, &chunk_payload)
        .map_err(|err| err.to_string())?;

    let value = extraction
        .field_value_map
        .get(template)
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let confidence = extraction
        .field_confidence_map
        .get(template)
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let detail = json!({
        "template": extraction.template,
        "query": extraction.query,
        "value": value,
        "confidence": confidence,
        "citations": extraction.citations,
        "retrieval_hits": hits.len(),
    })
    .to_string();

    Ok(CommandAck {
        command: "run_extraction".into(),
        status: "ok".into(),
        detail,
    })
}
