use tauri::State;

use crate::{
    core::workspace::WorkspaceService,
    infra::db::Database,
    models::contracts::{CommandAck, WorkspaceFolderInput},
    state::AppState,
};

#[tauri::command]
pub fn register_workspace_folder(
    state: State<'_, AppState>,
    database: State<'_, Database>,
    input: WorkspaceFolderInput,
) -> Result<CommandAck, String> {
    let normalized_path = input.path.trim();
    if normalized_path.is_empty() {
        return Err("workspace path cannot be empty".into());
    }

    let service = WorkspaceService;
    let inserted = database
        .register_workspace_folder(normalized_path)
        .map_err(|err| err.to_string())?;
    state.register_workspace(normalized_path);
    state.record_command("register_workspace_folder");

    Ok(CommandAck {
        command: "register_workspace_folder".into(),
        status: if inserted { "ok" } else { "exists" }.into(),
        detail: service.register_path(normalized_path),
    })
}
