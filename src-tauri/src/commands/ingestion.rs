use tauri::State;

use crate::{
    core::ingestion::IngestionService,
    infra::db::Database,
    infra::python_bridge::PythonBridge,
    models::contracts::{CommandAck, IngestionRequest},
    state::AppState,
};

#[tauri::command]
pub fn run_ingestion(
    state: State<'_, AppState>,
    database: State<'_, Database>,
    python_bridge: State<'_, PythonBridge>,
    request: IngestionRequest,
) -> Result<CommandAck, String> {
    log::info!(
        "ingestion requested workspace={} incremental={}",
        request.workspace_path,
        request.incremental
    );

    if !database
        .workspace_folder_exists(&request.workspace_path)
        .map_err(|err| err.to_string())?
    {
        log::error!("ingestion blocked for unregistered workspace={}", request.workspace_path);
        return Err(format!(
            "workspace '{}' is not registered",
            request.workspace_path
        ));
    }

    state.register_workspace(&request.workspace_path);
    state.record_command("run_ingestion");
    state.set_last_ingestion_target(&request.workspace_path);

    let service = IngestionService;
    let result = service
        .run(&database, &python_bridge, &request)
        .map_err(|err| {
            log::error!(
                "ingestion failed workspace={} error={}",
                request.workspace_path,
                err
            );
            err.to_string()
        })?;

    log::info!(
        "ingestion finished workspace={} discovered={} indexed={} parsed={} chunks={}",
        request.workspace_path,
        result.discovered_files,
        result.indexed_files,
        result.parsed_files,
        result.chunk_count
    );

    Ok(CommandAck {
        command: "run_ingestion".into(),
        status: "ok".into(),
        detail: service.summary_detail(&result),
    })
}

