const COMMANDS: &[&str] = &[
    "list_matters",
    "create_matter",
    "update_matter",
    "delete_matter",
    "ingest_paths",
    "list_documents",
    "search",
    "pending_embeddings",
    "store_embeddings",
    "embedding_status",
    "review_grid",
    "propose_renames",
    "list_actions",
    "set_action_status",
    "execute_approved",
    "undo_action",
    "authorize_model_call",
    "list_audit",
    "verify_audit",
    "export_register_csv",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
