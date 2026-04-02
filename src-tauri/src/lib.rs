pub mod commands;
pub mod core;
pub mod infra;
pub mod models;
pub mod state;

use commands::{
    actions::{
        approve_actions, evaluate_action_risk, execute_actions,
        generate_action_recommendations, get_action_audit_log, get_action_preview_queue,
        undo_action_batch,
    },
    benchmark::{get_run_metrics, run_benchmark_suite},
    extraction::run_extraction,
    ingestion::run_ingestion,
    memory::{
        build_long_context, delete_memory_fact, get_conversation, get_document_summary,
        list_conversations, list_memory_facts, save_memory_fact, start_conversation,
    },
    retrieval::run_retrieval_query,
    telemetry::{get_ingestion_telemetry, get_runtime_status},
    workspace::register_workspace_folder,
};
use infra::db::Database;
use infra::python_bridge::PythonBridge;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database = Database::connect_default().expect("failed to initialize ALIV database");
            let python_bridge = PythonBridge::new().expect("failed to initialize ALIV python bridge");
            app.manage(database);
            app.manage(python_bridge);
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            register_workspace_folder,
            run_ingestion,
            run_retrieval_query,
            run_extraction,
            start_conversation,
            list_conversations,
            get_conversation,
            save_memory_fact,
            list_memory_facts,
            delete_memory_fact,
            get_document_summary,
            build_long_context,
            generate_action_recommendations,
            get_action_preview_queue,
            evaluate_action_risk,
            approve_actions,
            execute_actions,
            undo_action_batch,
            get_action_audit_log,
            run_benchmark_suite,
            get_run_metrics,
            get_runtime_status,
            get_ingestion_telemetry,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ALIV desktop app");
}
