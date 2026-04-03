use tauri::State;

use crate::{
    core::memory::{ContextBlock, MemoryService, TokenBudget},
    core::retrieval::RetrievalService,
    infra::db::Database,
    infra::python_bridge::PythonBridge,
    models::contracts::{RetrievalRequest, RetrievalResponse},
    state::AppState,
};

#[tauri::command]
pub fn run_retrieval_query(
    state: State<'_, AppState>,
    database: State<'_, Database>,
    bridge: State<'_, PythonBridge>,
    request: RetrievalRequest,
) -> Result<RetrievalResponse, String> {
    state.record_command("run_retrieval_query");
    let retrieval = RetrievalService;
    let top_k = std::cmp::max(request.top_k, 1);

    // Stage 1: lexical metadata search (always available).
    let lexical_hits = database
        .search_chunks(&request.query, top_k.saturating_mul(3))
        .map_err(|err| err.to_string())?;

    // Stage 2: semantic ANN search from query embedding.
    // If embedding fails, degrade gracefully to lexical-only retrieval.
    let semantic_hits = match bridge.embed_documents(std::slice::from_ref(&request.query)) {
        Ok(embedding) => {
            let query_vector = embedding
                .vectors
                .first()
                .ok_or_else(|| "embedding worker returned no vectors".to_string())?;

            database
                .search_embeddings(query_vector, top_k.saturating_mul(3))
                .unwrap_or_default()
        }
        Err(_) => Vec::new(),
    };

    let hits = retrieval.merge_hybrid_hits(lexical_hits, semantic_hits, top_k);

    let mut assembled_prompt = None;
    let mut prompt_token_estimate = None;

    if let (Some(conversation_id), Some(workspace_path)) = (
        request.conversation_id.as_ref(),
        request.workspace_path.as_ref(),
    ) {
        let budget = match request.token_budget {
            Some(window) => TokenBudget {
                model_context_window: window,
                ..TokenBudget::default()
            },
            None => TokenBudget::default(),
        };

        let raw_turns = database
            .get_conversation_turns(conversation_id, 20)
            .map_err(|err| err.to_string())?;
        let raw_facts = database
            .list_memory_facts(workspace_path)
            .map_err(|err| err.to_string())?;

        let history_blocks: Vec<ContextBlock> = raw_turns
            .into_iter()
            .map(|turn| ContextBlock {
                block_type: "history_turn".into(),
                content: format!("{}: {}", turn.role.to_uppercase(), turn.content),
                token_estimate: MemoryService::estimate_tokens(&turn.content),
                source_id: Some(turn.id),
                relevance_score: None,
            })
            .collect();

        let fact_blocks: Vec<ContextBlock> = raw_facts
            .into_iter()
            .map(|fact| ContextBlock {
                block_type: "memory_fact".into(),
                content: fact.content.clone(),
                token_estimate: MemoryService::estimate_tokens(&fact.content),
                source_id: Some(fact.id),
                relevance_score: None,
            })
            .collect();

        let chunk_blocks: Vec<ContextBlock> = hits
            .iter()
            .map(|hit| ContextBlock {
                block_type: "chunk".into(),
                content: hit.content_preview.clone(),
                token_estimate: MemoryService::estimate_tokens(&hit.content_preview),
                source_id: Some(format!("{}#{}", hit.file_id, hit.chunk_index)),
                relevance_score: Some(hit.score as f32),
            })
            .collect();

        let instruction = "You are a precise document analysis assistant for legal and financial documents. \
            Answer based only on the provided context. Always cite your sources. \
            If the answer is not in the context, say so.";

        let context = MemoryService::assemble_context(
            &request.query,
            conversation_id,
            instruction,
            fact_blocks,
            history_blocks,
            chunk_blocks,
            vec![],
            &budget,
        );

        assembled_prompt = Some(MemoryService::format_prompt(&context));
        prompt_token_estimate = Some(context.total_token_estimate);
    }

    Ok(RetrievalResponse {
        command: "run_retrieval_query".into(),
        status: "ok".into(),
        query: request.query,
        top_k,
        total_hits: hits.len(),
        assembled_prompt,
        prompt_token_estimate,
        hits,
    })
}
