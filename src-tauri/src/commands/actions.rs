use tauri::State;

use crate::models::contracts::{
    ActionApprovalRequest, ActionExecutionRequest, ActionRecommendationRequest,
    ActionRiskRequest, AuditLogRequest, CommandAck, UndoBatchRequest,
};
use crate::state::AppState;

#[tauri::command]
pub fn generate_action_recommendations(
    state: State<'_, AppState>,
    request: ActionRecommendationRequest,
) -> Result<CommandAck, String> {
    let queue = (0..request.batch_limit)
        .map(|index| format!("preview-{}", index + 1))
        .collect::<Vec<_>>();
    state.record_command("generate_action_recommendations");
    state.replace_preview_queue(queue);

    Ok(CommandAck {
        command: "generate_action_recommendations".into(),
        status: "ok".into(),
        detail: format!(
            "generated {} preview recommendations for {}",
            state.preview_queue_len(),
            request.workspace_path
        ),
    })
}

#[tauri::command]
pub fn get_action_preview_queue(state: State<'_, AppState>) -> Result<CommandAck, String> {
    state.record_command("get_action_preview_queue");

    Ok(CommandAck {
        command: "get_action_preview_queue".into(),
        status: "ok".into(),
        detail: format!("preview queue contains {} items", state.preview_queue_len()),
    })
}

#[tauri::command]
pub fn evaluate_action_risk(
    state: State<'_, AppState>,
    request: ActionRiskRequest,
) -> Result<CommandAck, String> {
    state.record_command("evaluate_action_risk");

    Ok(CommandAck {
        command: "evaluate_action_risk".into(),
        status: "ok".into(),
        detail: format!("risk evaluation scaffolded for {} recommendations", request.recommendation_ids.len()),
    })
}

#[tauri::command]
pub fn approve_actions(
    state: State<'_, AppState>,
    request: ActionApprovalRequest,
) -> Result<CommandAck, String> {
    state.record_command("approve_actions");

    Ok(CommandAck {
        command: "approve_actions".into(),
        status: "ok".into(),
        detail: format!("approval scaffolded for {} recommendations", request.recommendation_ids.len()),
    })
}

#[tauri::command]
pub fn execute_actions(
    state: State<'_, AppState>,
    request: ActionExecutionRequest,
) -> Result<CommandAck, String> {
    state.record_command("execute_actions");

    Ok(CommandAck {
        command: "execute_actions".into(),
        status: "ok".into(),
        detail: format!("execution scaffolded for batch {}", request.batch_id),
    })
}

#[tauri::command]
pub fn undo_action_batch(
    state: State<'_, AppState>,
    request: UndoBatchRequest,
) -> Result<CommandAck, String> {
    state.record_command("undo_action_batch");

    Ok(CommandAck {
        command: "undo_action_batch".into(),
        status: "ok".into(),
        detail: format!("undo scaffolded for batch {}", request.execution_batch_id),
    })
}

#[tauri::command]
pub fn get_action_audit_log(
    state: State<'_, AppState>,
    request: AuditLogRequest,
) -> Result<CommandAck, String> {
    state.record_command("get_action_audit_log");

    Ok(CommandAck {
        command: "get_action_audit_log".into(),
        status: "ok".into(),
        detail: format!("audit log scaffolded with limit {}", request.limit),
    })
}
