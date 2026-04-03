use tauri::State;
use uuid::Uuid;

use crate::core::actions::ActionService;
use crate::infra::db::Database;
use crate::infra::fs_guard::FilesystemGuard;
use crate::models::contracts::{
    ActionApprovalRequest, ActionExecutionRequest, ActionExecutionResult, ActionPreviewList,
    ActionRecommendationRequest, ActionRecommendationView, ActionRiskRequest, AuditLogEntry,
    AuditLogRequest, AuditLogResponse, CommandAck, UndoBatchRequest, UndoResult,
};
use crate::state::AppState;

#[tauri::command]
pub fn generate_action_recommendations(
    state: State<'_, AppState>,
    db: State<'_, Database>,
    request: ActionRecommendationRequest,
) -> Result<ActionPreviewList, String> {
    state.record_command("generate_action_recommendations");

    let guard = FilesystemGuard::default();
    if !guard.validate_workspace_path(&request.workspace_path) {
        return Err(format!("workspace path invalid: {}", request.workspace_path));
    }

    let rows = db
        .list_files_needing_rename(&request.workspace_path, request.batch_limit)
        .map_err(|e| e.to_string())?;

    let service = ActionService::default();
    let mut items: Vec<ActionRecommendationView> = Vec::new();

    for (file_id, source_path, inferred_doc_type, confidence_raw) in rows {
        let confidence = confidence_raw.unwrap_or(0.0);
        if confidence < 0.70 {
            continue;
        }

        let path = std::path::Path::new(&source_path);
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let parent = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let proposed_name = guard.normalize_filename(&inferred_doc_type, "Unknown", "undated", &ext);
        let proposed_path = format!("{}/{}", parent, proposed_name);

        let rec_id = Uuid::new_v4().to_string();
        let evidence = vec![
            format!("parser_confidence={:.2}", confidence),
            format!("inferred_doc_type={}", inferred_doc_type),
        ];
        let evidence_json = serde_json::to_string(&evidence).unwrap_or_default();

        db.save_recommendation(
            &rec_id,
            &file_id,
            "rename",
            &source_path,
            &proposed_path,
            confidence,
            &evidence_json,
        )
        .map_err(|e| e.to_string())?;

        let mut rec = ActionRecommendationView {
            id: rec_id,
            file_id: file_id.clone(),
            recommendation_type: "rename".into(),
            source_path: source_path.clone(),
            proposed_path: proposed_path.clone(),
            confidence,
            evidence,
            risk_level: "low".into(),
            risk_flags: vec![],
            created_at: String::new(),
        };

        let (risk_level, risk_flags) = service.evaluate_risk(&rec);
        rec.risk_level = risk_level;
        rec.risk_flags = risk_flags;

        items.push(rec);
    }

    let total = items.len();
    let ids: Vec<String> = items.iter().map(|r| r.id.clone()).collect();
    state.replace_preview_queue(ids);
    state.clear_action_approval();

    Ok(ActionPreviewList {
        command: "generate_action_recommendations".into(),
        status: "ok".into(),
        workspace_path: request.workspace_path,
        total,
        items,
    })
}

#[tauri::command]
pub fn get_action_preview_queue(
    state: State<'_, AppState>,
    _db: State<'_, Database>,
) -> Result<ActionPreviewList, String> {
    state.record_command("get_action_preview_queue");

    Ok(ActionPreviewList {
        command: "get_action_preview_queue".into(),
        status: "ok".into(),
        workspace_path: String::new(),
        total: state.preview_queue_len(),
        items: vec![],
    })
}

#[tauri::command]
pub fn evaluate_action_risk(
    state: State<'_, AppState>,
    db: State<'_, Database>,
    request: ActionRiskRequest,
) -> Result<ActionPreviewList, String> {
    state.record_command("evaluate_action_risk");

    let service = ActionService::default();
    let mut items: Vec<ActionRecommendationView> = Vec::new();

    for rec_id in &request.recommendation_ids {
        if let Ok(Some(row)) = db.get_recommendation(rec_id) {
            let (id, file_id, rec_type, source_path, proposed_path, confidence, evidence_json) = row;
            let evidence: Vec<String> = serde_json::from_str(&evidence_json).unwrap_or_default();
            let mut rec = ActionRecommendationView {
                id,
                file_id,
                recommendation_type: rec_type,
                source_path,
                proposed_path,
                confidence,
                evidence,
                risk_level: "low".into(),
                risk_flags: vec![],
                created_at: String::new(),
            };
            let (risk_level, risk_flags) = service.evaluate_risk(&rec);
            rec.risk_level = risk_level;
            rec.risk_flags = risk_flags;
            items.push(rec);
        }
    }

    Ok(ActionPreviewList {
        command: "evaluate_action_risk".into(),
        status: "ok".into(),
        workspace_path: String::new(),
        total: items.len(),
        items,
    })
}

#[tauri::command]
pub fn approve_actions(
    state: State<'_, AppState>,
    request: ActionApprovalRequest,
) -> Result<CommandAck, String> {
    state.record_command("approve_actions");
    state.set_approved_preview_queue(request.recommendation_ids.clone());

    Ok(CommandAck {
        command: "approve_actions".into(),
        status: "ok".into(),
        detail: format!("{} recommendations approved", request.recommendation_ids.len()),
    })
}

#[tauri::command]
pub fn execute_actions(
    state: State<'_, AppState>,
    db: State<'_, Database>,
    request: ActionExecutionRequest,
) -> Result<ActionExecutionResult, String> {
    state.record_command("execute_actions");
    log::info!("executing action batch {}", request.batch_id);

    let service = ActionService::default();

    if !state.is_action_approval_active() {
        return Err("approve_actions must be called before execute_actions".into());
    }

    let rec_ids = state.drain_preview_queue();
    let mut succeeded = 0;
    let mut failed = 0;
    let mut errors: Vec<String> = vec![];

    if rec_ids.is_empty() {
        return Err("no approved recommendations available for execution".into());
    }

    for rec_id in &rec_ids {
        match db.get_recommendation(rec_id) {
            Ok(Some(row)) => {
                let (id, _file_id, rec_type, source_path, proposed_path, _confidence, _evidence_json) = row;
                let workspace_root = std::path::Path::new(&source_path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                match service.execute_rename(&source_path, &proposed_path, &workspace_root) {
                    Ok(actual_dest) => {
                        log::info!(
                            "rename executed batch={} from={} to={}",
                            request.batch_id,
                            source_path,
                            actual_dest
                        );
                        let exec_id = Uuid::new_v4().to_string();
                        if let Err(e) = db.save_execution(&exec_id, &id, &request.batch_id, "success") {
                            log::error!(
                                "failed to save successful execution batch={} source={} error={}",
                                request.batch_id,
                                source_path,
                                e
                            );
                            errors.push(format!("{}: {}", source_path, e));
                            failed += 1;
                            continue;
                        }

                        let undo_json =
                            service.build_undo_record_json(&source_path, &actual_dest, &rec_type);
                        let undo_id = Uuid::new_v4().to_string();
                        if let Err(e) = db.save_undo_record(&undo_id, &exec_id, &undo_json) {
                            let _ = std::fs::rename(&actual_dest, &source_path);
                            let _ = db.update_execution_status(&exec_id, "failed");
                            log::error!(
                                "failed to persist undo record batch={} source={} error={}",
                                request.batch_id,
                                source_path,
                                e
                            );
                            errors.push(format!("{}: undo record failed: {}", source_path, e));
                            failed += 1;
                            continue;
                        }

                        succeeded += 1;
                    }
                    Err(err) => {
                        let exec_id = Uuid::new_v4().to_string();
                        let _ = db.save_execution(&exec_id, &id, &request.batch_id, "failed");
                        log::error!(
                            "rename failed batch={} source={} target={} error={}",
                            request.batch_id,
                            source_path,
                            proposed_path,
                            err
                        );
                        errors.push(format!("{}: {}", source_path, err));
                        failed += 1;
                    }
                }
            }
            _ => {
                log::error!(
                    "recommendation missing during execution batch={} recommendation_id={}",
                    request.batch_id,
                    rec_id
                );
                errors.push(format!("recommendation not found: {}", rec_id));
                failed += 1;
            }
        }
    }

    Ok(ActionExecutionResult {
        command: "execute_actions".into(),
        status: if failed == 0 {
            "ok".into()
        } else {
            "partial".into()
        },
        batch_id: request.batch_id,
        total: rec_ids.len(),
        succeeded,
        failed,
        errors,
    })
}

#[tauri::command]
pub fn undo_action_batch(
    state: State<'_, AppState>,
    db: State<'_, Database>,
    request: UndoBatchRequest,
) -> Result<UndoResult, String> {
    state.record_command("undo_action_batch");
    log::info!("undo requested for batch {}", request.execution_batch_id);

    let undo_records = db
        .get_undo_records_for_batch(&request.execution_batch_id)
        .map_err(|e| e.to_string())?;

    let mut reversed = 0;
    let mut errors: Vec<String> = vec![];

    for (_undo_id, exec_id, inverse_json) in &undo_records {
        let op: serde_json::Value = match serde_json::from_str(inverse_json) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("invalid undo record {exec_id}: {e}"));
                continue;
            }
        };

        let from = op["from"].as_str().unwrap_or_default();
        let to = op["to"].as_str().unwrap_or_default();

        if from.is_empty() || to.is_empty() {
            errors.push(format!("incomplete undo op for exec {exec_id}"));
            continue;
        }

        match std::fs::rename(from, to) {
            Ok(_) => {
                let _ = db.update_execution_status(exec_id, "rolled_back");
                log::info!(
                    "undo succeeded batch={} from={} to={}",
                    request.execution_batch_id,
                    from,
                    to
                );
                reversed += 1;
            }
            Err(e) => {
                log::error!(
                    "undo failed batch={} from={} to={} error={}",
                    request.execution_batch_id,
                    from,
                    to,
                    e
                );
                errors.push(format!("undo failed for exec {exec_id}: {e}"));
            }
        }
    }

    Ok(UndoResult {
        command: "undo_action_batch".into(),
        status: if errors.is_empty() {
            "ok".into()
        } else {
            "partial".into()
        },
        batch_id: request.execution_batch_id,
        reversed,
        errors,
    })
}

#[tauri::command]
pub fn get_action_audit_log(
    state: State<'_, AppState>,
    db: State<'_, Database>,
    request: AuditLogRequest,
) -> Result<AuditLogResponse, String> {
    state.record_command("get_action_audit_log");

    let rows = db.list_audit_log(request.limit).map_err(|e| e.to_string())?;

    let entries: Vec<AuditLogEntry> = rows
        .into_iter()
        .map(
            |(exec_id, rec_id, batch_id, source_path, proposed_path, rec_type, status, executed_at)| {
                AuditLogEntry {
                    execution_id: exec_id,
                    recommendation_id: rec_id,
                    batch_id,
                    source_path,
                    proposed_path,
                    recommendation_type: rec_type,
                    status,
                    executed_at,
                }
            },
        )
        .collect();

    let total = entries.len();

    Ok(AuditLogResponse {
        command: "get_action_audit_log".into(),
        status: "ok".into(),
        total,
        entries,
    })
}
