//! ALIV matter vault: the trusted core that owns matter data on disk.
//!
//! Documents are parsed, split into passages and indexed locally (SQLite
//! FTS5). Extracted fields keep a pointer to the passage they came from.
//! File changes are proposals until approved, and every ingestion, file
//! action and model call is written to a hash-chained audit log.

pub mod chunk;
#[cfg(feature = "tauri")]
mod commands;
pub mod error;
pub mod extract;
pub mod model;
pub mod ocr;
pub mod store;
pub mod vault;

pub use error::VaultError;
pub use vault::Vault;

#[cfg(feature = "tauri")]
pub fn init<R: tauri::Runtime>(db_path: std::path::PathBuf) -> tauri::plugin::TauriPlugin<R> {
    use tauri::Manager;

    tauri::plugin::Builder::new("aliv-vault")
        .invoke_handler(tauri::generate_handler![
            commands::list_matters,
            commands::create_matter,
            commands::update_matter,
            commands::delete_matter,
            commands::ingest_paths,
            commands::list_documents,
            commands::search,
            commands::review_grid,
            commands::propose_renames,
            commands::list_actions,
            commands::set_action_status,
            commands::execute_approved,
            commands::undo_action,
            commands::authorize_model_call,
            commands::list_audit,
            commands::verify_audit,
            commands::export_register_csv,
        ])
        .setup(move |app, _api| {
            let store = store::Store::open(&db_path).map_err(|e| e.to_string())?;
            app.manage(std::sync::Arc::new(vault::Vault::new(store)));
            Ok(())
        })
        .build()
}
