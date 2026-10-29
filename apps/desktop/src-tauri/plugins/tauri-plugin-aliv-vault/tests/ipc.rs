//! Drives the plugin through Tauri's IPC layer with the exact payloads the
//! web app sends (web-app/src/aliv/tauri-service.ts), so a renamed command
//! or argument fails here instead of at runtime.
#![cfg(feature = "tauri")]

use serde_json::{json, Value};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::WebviewWindowBuilder;

fn call(webview: &tauri::WebviewWindow<tauri::test::MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: format!("plugin:aliv-vault|{cmd}"),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|r| r.deserialize::<Value>().unwrap())
}

/// A mock context whose ACL holds this plugin's real permission files and a
/// capability granting `aliv-vault:default` to the main window, like the
/// app's capabilities/default.json. A command missing from the default
/// permission set is therefore rejected here too.
fn context_granting_default_permissions() -> tauri::Context<tauri::test::MockRuntime> {
    use std::collections::BTreeMap;
    use tauri_utils::acl::{capability::Capability, manifest::{Manifest, PermissionFile}, resolved::Resolved};
    use tauri_utils::platform::Target;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("permissions");
    let files: Vec<PermissionFile> = walk(&root)
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .map(|p| toml::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap())
        .collect();
    let acl = BTreeMap::from([("aliv-vault".to_string(), Manifest::new(files, None))]);
    let capability: Capability = serde_json::from_value(json!({
        "identifier": "test",
        "windows": ["main"],
        "permissions": ["aliv-vault:default"]
    }))
    .unwrap();
    let resolved = Resolved::resolve(&acl, BTreeMap::from([("test".to_string(), capability)]), Target::current()).unwrap();
    let mut ctx = mock_context(noop_assets());
    *ctx.runtime_authority_mut() = tauri::ipc::RuntimeAuthority::new(acl, resolved);
    ctx
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}

#[test]
fn web_app_payloads_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("lease.txt"),
        "COMMERCIAL LEASE AGREEMENT\nVendor / Landlord: Acme Commercial Properties LLC\nClient / Tenant: Redstone Enterprises Inc.\nEffective Date: 2025-03-01\nExpiry Date: 2026-03-01\nHoldover penalty of $2,800 per day.\n",
    )
    .unwrap();

    let app = mock_builder()
        .plugin(tauri_plugin_aliv_vault::init(dir.path().join("db").join("vault.db")))
        .build(context_granting_default_permissions())
        .unwrap();
    let w = WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();

    let m = call(&w, "create_matter", json!({ "input": { "name": "Redstone", "client": "Redstone Inc.", "kind": "legal", "policy": "local_only" } })).unwrap();
    let id = m["id"].as_str().unwrap().to_string();
    assert_eq!(m["policy"], "local_only");
    assert_eq!(m["documentCount"], 0);

    let r = call(&w, "ingest_paths", json!({ "matterId": id, "paths": [dir.path().join("lease.txt")] })).unwrap();
    assert_eq!(r["indexed"], 1);

    let docs = call(&w, "list_documents", json!({ "matterId": id })).unwrap();
    assert_eq!(docs[0]["fileName"], "lease.txt");
    assert_eq!(docs[0]["docType"], "Lease");

    let hits = call(&w, "search", json!({ "matterId": id, "query": "holdover penalty", "limit": 5 })).unwrap();
    assert_eq!(hits[0]["fileName"], "lease.txt");
    assert!(hits[0]["chunkId"].is_number());

    let grid = call(&w, "review_grid", json!({ "matterId": id })).unwrap();
    let fields = grid[0]["fields"].as_array().unwrap();
    assert!(fields.iter().any(|f| f["key"] == "expiry_date" && f["value"] == "2026-03-01" && f["chunkOrdinal"] == 0));

    let decision = call(
        &w,
        "authorize_model_call",
        json!({ "request": { "matterId": id, "provider": "anthropic", "model": "claude", "location": "cloud", "charsSent": 900, "sourceCount": 3, "purpose": "answer" } }),
    )
    .unwrap();
    assert_eq!(decision["allowed"], false);
    assert!(decision["auditId"].is_number());

    let updated = call(&w, "update_matter", json!({ "id": id, "input": { "name": "", "client": "Redstone Inc.", "policy": "hybrid" } })).unwrap();
    assert_eq!(updated["policy"], "hybrid");

    let proposals = call(&w, "propose_renames", json!({ "matterId": id })).unwrap();
    let action = proposals[0]["id"].as_str().unwrap().to_string();
    assert_eq!(call(&w, "execute_approved", json!({ "matterId": id })).unwrap(), json!([]));
    call(&w, "set_action_status", json!({ "id": action, "status": "approved" })).unwrap();
    let done = call(&w, "execute_approved", json!({ "matterId": id })).unwrap();
    assert_eq!(done[0]["status"], "executed");
    assert!(dir.path().join("Lease_AcmeCommercialProperties_2025-03-01.txt").exists());
    let undone = call(&w, "undo_action", json!({ "id": action })).unwrap();
    assert_eq!(undone["status"], "undone");
    assert_eq!(call(&w, "list_actions", json!({ "matterId": id })).unwrap()[0]["status"], "undone");

    let csv = dir.path().join("out.csv");
    assert_eq!(call(&w, "export_register_csv", json!({ "matterId": id, "dest": csv })).unwrap(), 1);

    let audit = call(&w, "list_audit", json!({ "matterId": id, "limit": 50 })).unwrap();
    assert!(audit.as_array().unwrap().iter().any(|a| a["kind"] == "model_call_blocked"));
    let all = call(&w, "list_audit", json!({ "matterId": null })).unwrap();
    assert!(all.as_array().unwrap().len() >= audit.as_array().unwrap().len());
    assert_eq!(call(&w, "verify_audit", json!({})).unwrap()["ok"], true);

    // Errors reach the webview as readable strings.
    let err = call(&w, "set_action_status", json!({ "id": "nope", "status": "approved" })).unwrap_err();
    assert!(err.as_str().unwrap().contains("not found"), "{err}");

    call(&w, "delete_matter", json!({ "id": id })).unwrap();
    assert_eq!(call(&w, "list_matters", json!({})).unwrap(), json!([]));
}
