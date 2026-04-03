use tauri::State;
use uuid::Uuid;

use crate::{
    core::memory::{MemoryService, TokenBudget},
    infra::db::Database,
    models::contracts::{
        BuildLongContextRequest, CommandAck, ConversationStub, ConversationTurnView,
        DeleteMemoryFactRequest, GetConversationRequest, GetDocumentSummaryRequest,
        ListConversationsRequest, LongContextResponse, MemoryFactView, SaveMemoryFactRequest,
        StartConversationRequest,
    },
};

/// Start a new conversation session for the given workspace.
/// Returns the conversation_id to pass to subsequent query commands.
#[tauri::command]
pub fn start_conversation(
    database: State<'_, Database>,
    request: StartConversationRequest,
) -> Result<ConversationStub, String> {
    let id = Uuid::new_v4().to_string();
    database
        .create_conversation(&id, &request.workspace_path, "Untitled")
        .map_err(|e| e.to_string())?;
    Ok(ConversationStub {
        id,
        title: "Untitled".into(),
        turn_count: 0,
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// List recent conversations for a workspace.
#[tauri::command]
pub fn list_conversations(
    database: State<'_, Database>,
    request: ListConversationsRequest,
) -> Result<Vec<ConversationStub>, String> {
    database
        .list_conversations(&request.workspace_path, request.limit)
        .map_err(|e| e.to_string())
}

/// Fetch recent turns for a conversation.
#[tauri::command]
pub fn get_conversation(
    database: State<'_, Database>,
    request: GetConversationRequest,
) -> Result<Vec<ConversationTurnView>, String> {
    database
        .get_conversation_turns(&request.conversation_id, request.limit)
        .map_err(|e| e.to_string())
}

/// Pin a fact to memory so it is recalled in future queries.
#[tauri::command]
pub fn save_memory_fact(
    database: State<'_, Database>,
    request: SaveMemoryFactRequest,
) -> Result<CommandAck, String> {
    let id = Uuid::new_v4().to_string();
    let scope = match request.scope.as_str() {
        "session" | "week" | "permanent" => request.scope.clone(),
        other => {
            return Err(format!(
                "invalid scope '{}': must be session, week, or permanent",
                other
            ))
        }
    };
    let expires_at = match scope.as_str() {
        "session" => Some(chrono::Utc::now().to_rfc3339()), // expires immediately on restart
        "week" => Some(
            (chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339(),
        ),
        _ => None,
    };
    database
        .save_memory_fact(
            &id,
            &request.workspace_path,
            &request.content,
            request.citation_json.as_deref(),
            &scope,
            expires_at.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    Ok(CommandAck {
        command: "save_memory_fact".into(),
        status: "ok".into(),
        detail: format!("fact_id={}", id),
    })
}

/// List all non-expired memory facts for a workspace.
#[tauri::command]
pub fn list_memory_facts(
    database: State<'_, Database>,
    request: ListConversationsRequest, // reuses workspace_path + limit
) -> Result<Vec<MemoryFactView>, String> {
    database
        .list_memory_facts(&request.workspace_path)
        .map_err(|e| e.to_string())
}

/// Delete a pinned memory fact.
#[tauri::command]
pub fn delete_memory_fact(
    database: State<'_, Database>,
    request: DeleteMemoryFactRequest,
) -> Result<CommandAck, String> {
    database
        .delete_memory_fact(&request.fact_id)
        .map_err(|e| e.to_string())?;
    Ok(CommandAck {
        command: "delete_memory_fact".into(),
        status: "ok".into(),
        detail: format!("deleted fact_id={}", request.fact_id),
    })
}

/// Fetch cached document summary (document or section level).
#[tauri::command]
pub fn get_document_summary(
    database: State<'_, Database>,
    request: GetDocumentSummaryRequest,
) -> Result<CommandAck, String> {
    let summary = database
        .get_document_summary(&request.file_id, &request.granularity)
        .map_err(|e| e.to_string())?;
    match summary {
        Some(text) => Ok(CommandAck {
            command: "get_document_summary".into(),
            status: "ok".into(),
            detail: text,
        }),
        None => Ok(CommandAck {
            command: "get_document_summary".into(),
            status: "not_found".into(),
            detail: format!(
                "no {} summary cached for file_id={}",
                request.granularity, request.file_id
            ),
        }),
    }
}

/// Assemble a long-context prompt for a query by combining conversation history,
/// pinned memory facts, retrieved chunks, and document summaries within a token budget.
#[tauri::command]
pub fn build_long_context(
    database: State<'_, Database>,
    request: BuildLongContextRequest,
) -> Result<LongContextResponse, String> {
    let budget = match request.token_budget {
        Some(window) => TokenBudget {
            model_context_window: window,
            ..TokenBudget::default()
        },
        None => TokenBudget::default(),
    };

    // Load conversation history (last 20 turns, newest-first)
    let raw_turns = database
        .get_conversation_turns(&request.conversation_id, 20)
        .map_err(|e| e.to_string())?;

    // Load non-expired memory facts
    let raw_facts = database
        .list_memory_facts(&request.workspace_path)
        .map_err(|e| e.to_string())?;

    // Convert to ContextBlocks
    let history_blocks = raw_turns
        .into_iter()
        .map(|t| crate::core::memory::ContextBlock {
            block_type: "history_turn".into(),
            content: format!("{}: {}", t.role.to_uppercase(), t.content),
            token_estimate: MemoryService::estimate_tokens(&t.content),
            source_id: Some(t.id),
            relevance_score: None,
        })
        .collect();

    let fact_blocks = raw_facts
        .into_iter()
        .map(|f| crate::core::memory::ContextBlock {
            block_type: "memory_fact".into(),
            content: f.content.clone(),
            token_estimate: MemoryService::estimate_tokens(&f.content),
            source_id: Some(f.id),
            relevance_score: None,
        })
        .collect();

    let instruction = "You are a precise document analysis assistant for legal and financial documents. \
        Answer based only on the provided context. Always cite your sources. \
        If the answer is not in the context, say so.";

    let ctx = MemoryService::assemble_context(
        &request.query,
        &request.conversation_id,
        instruction,
        fact_blocks,
        history_blocks,
        vec![], // chunk_blocks: caller populates after retrieval
        vec![], // summary_blocks: caller populates for long docs
        &budget,
    );

    let prompt = MemoryService::format_prompt(&ctx);

    Ok(LongContextResponse {
        conversation_id: ctx.conversation_id,
        assembled_prompt: prompt,
        total_token_estimate: ctx.total_token_estimate,
        token_budget: ctx.token_budget,
        overflow_dropped: ctx.overflow_dropped,
    })
}
