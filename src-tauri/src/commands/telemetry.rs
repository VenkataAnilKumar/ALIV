use tauri::State;

use crate::{infra::db::Database, models::contracts::CommandAck, state::AppState};

#[tauri::command]
pub fn get_runtime_status(
    state: State<'_, AppState>,
    database: State<'_, Database>,
) -> Result<CommandAck, String> {
    state.record_command("get_runtime_status");
    let snapshot = state.snapshot();
    let database_status = database.health_check().map_err(|err| err.to_string())?;

    Ok(CommandAck {
        command: "get_runtime_status".into(),
        status: "ok".into(),
        detail: format!(
            "workspaces={}, commands={}, preview_queue={}, benchmark_runs={}, last_ingestion={}, db={}",
            snapshot.workspace_count,
            snapshot.command_count,
            snapshot.preview_queue_count,
            snapshot.benchmark_run_count,
            snapshot.last_ingestion_target.unwrap_or_else(|| "none".into()),
            database_status
        ),
    })
}

#[tauri::command]
pub fn get_ingestion_telemetry(database: State<'_, Database>) -> Result<CommandAck, String> {
    let snapshot = database
        .ingestion_telemetry_snapshot()
        .map_err(|err| err.to_string())?;

    Ok(CommandAck {
        command: "get_ingestion_telemetry".into(),
        status: "ok".into(),
        detail: format!(
            "runs={} discovered_total={} indexed_total={} skipped_total={} parsed_total={} failed_total={} files={} parsed={} chunks={} embeddings={} avg_parser_confidence={:.4} last_run={} last_workspace={} last_mode={} last_created_at={}",
            snapshot.run_count,
            snapshot.discovered_total,
            snapshot.indexed_total,
            snapshot.skipped_total,
            snapshot.parsed_total,
            snapshot.failed_total,
            snapshot.file_count,
            snapshot.parsed_file_count,
            snapshot.chunk_count,
            snapshot.embedding_count,
            snapshot.avg_parser_confidence,
            snapshot.last_run_id.unwrap_or_else(|| "none".into()),
            snapshot.last_workspace_path.unwrap_or_else(|| "none".into()),
            snapshot.last_mode.unwrap_or_else(|| "none".into()),
            snapshot.last_created_at.unwrap_or_else(|| "none".into()),
        ),
    })
}
