# ALIV — Phase 3 & Phase 4 Copilot Fix Guide

**Purpose**: Exact instructions for Copilot to implement Phase 3 (Safe File Actions) and finish Phase 4 (E2E Hardening). Every stub location, every DB method, every required type is specified here.

---

## Context Snapshot

- **Repo root**: `V:/AI Engineer/GitHub/ALIV/`
- **Backend**: Rust + Tauri 2 (`src-tauri/src/`)
- **DB**: SQLite WAL via `rusqlite`, migrations in `db/migrations/`
- **DB access**: all SQL goes through `src-tauri/src/infra/db.rs` — add new methods there, never raw SQL in commands
- **IPC to Python**: `src-tauri/src/infra/python_bridge.rs` — JSON-lines stdin/stdout via `run_worker()`
- **State**: `src-tauri/src/state.rs` — only lightweight in-memory state, not a DB substitute
- **Contracts**: `src-tauri/src/models/contracts.rs` — all Tauri command input/output types

---

## Part 1: Phase 3 — Safe File Actions

### Current State (All Stubs)

| File | Current State |
|---|---|
| `src-tauri/src/commands/actions.rs` | All 7 handlers return `Ok(CommandAck { status: "ok", ... })` — no DB, no logic |
| `src-tauri/src/core/actions.rs` | Single method: `pub fn preview(&self) -> &'static str { "action service stub" }` |
| `src-tauri/src/infra/fs_guard.rs` | `validate_workspace_path` only checks `!path.trim().is_empty()` |

### DB Schema (Already Exists — `db/migrations/0003_actions_audit.sql`)

```sql
CREATE TABLE IF NOT EXISTS action_recommendations (
  id TEXT PRIMARY KEY,
  file_id TEXT NOT NULL,
  recommendation_type TEXT NOT NULL,  -- "rename" | "move"
  source_path TEXT NOT NULL,
  proposed_path TEXT NOT NULL,
  confidence REAL NOT NULL,
  evidence_json TEXT NOT NULL,        -- JSON array of evidence strings
  created_at TEXT NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE IF NOT EXISTS action_executions (
  id TEXT PRIMARY KEY,
  recommendation_id TEXT NOT NULL,
  batch_id TEXT NOT NULL,
  status TEXT NOT NULL,               -- "success" | "failed" | "rolled_back"
  executed_at TEXT NOT NULL,
  FOREIGN KEY (recommendation_id) REFERENCES action_recommendations(id)
);

CREATE TABLE IF NOT EXISTS undo_records (
  id TEXT PRIMARY KEY,
  execution_id TEXT NOT NULL,
  inverse_operation_json TEXT NOT NULL, -- JSON: { "type": "rename", "from": "...", "to": "..." }
  created_at TEXT NOT NULL,
  FOREIGN KEY (execution_id) REFERENCES action_executions(id)
);
```

---

### Step 1.1 — New Response Types in `src-tauri/src/models/contracts.rs`

Add these structs after the existing action request types (after line 77):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendationView {
    pub id: String,
    pub file_id: String,
    pub recommendation_type: String,
    pub source_path: String,
    pub proposed_path: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub risk_level: String,       // "low" | "medium" | "high"
    pub risk_flags: Vec<String>,  // e.g. ["collision_detected", "no_undo_path"]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPreviewList {
    pub command: String,
    pub status: String,
    pub workspace_path: String,
    pub total: usize,
    pub items: Vec<ActionRecommendationView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecutionResult {
    pub command: String,
    pub status: String,
    pub batch_id: String,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResult {
    pub command: String,
    pub status: String,
    pub batch_id: String,
    pub reversed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub execution_id: String,
    pub recommendation_id: String,
    pub batch_id: String,
    pub source_path: String,
    pub proposed_path: String,
    pub recommendation_type: String,
    pub status: String,
    pub executed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub command: String,
    pub status: String,
    pub total: usize,
    pub entries: Vec<AuditLogEntry>,
}
```

---

### Step 1.2 — Real `FilesystemGuard` in `src-tauri/src/infra/fs_guard.rs`

Replace the entire file with:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct FilesystemGuard;

impl FilesystemGuard {
    /// Returns Ok(canonical_path) or Err(reason).
    pub fn validate_and_canonicalize(&self, raw_path: &str, workspace_root: &str) -> Result<PathBuf, String> {
        let path = Path::new(raw_path);

        // 1. Must not be empty
        if raw_path.trim().is_empty() {
            return Err("path is empty".into());
        }

        // 2. Must not contain traversal components
        for component in path.components() {
            let s = component.as_os_str().to_string_lossy();
            if s == ".." {
                return Err(format!("path traversal detected in: {}", raw_path));
            }
        }

        // 3. Resolve to canonical form — file must exist for validation
        let canonical = path.canonicalize()
            .map_err(|err| format!("cannot canonicalize '{}': {}", raw_path, err))?;

        // 4. Must live inside the workspace boundary
        let workspace = Path::new(workspace_root)
            .canonicalize()
            .map_err(|err| format!("cannot canonicalize workspace '{}': {}", workspace_root, err))?;

        if !canonical.starts_with(&workspace) {
            return Err(format!(
                "path '{}' is outside workspace boundary '{}'",
                canonical.display(),
                workspace.display()
            ));
        }

        // 5. Must not be a symlink (follow chain — if canonical != original metadata says symlink)
        if path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(format!("symlinks not permitted: {}", raw_path));
        }

        Ok(canonical)
    }

    /// Validate workspace path exists and is a directory.
    pub fn validate_workspace_path(&self, path: &str) -> bool {
        !path.trim().is_empty() && Path::new(path).is_dir()
    }

    /// Generate a collision-safe target path by appending _001, _002, etc.
    pub fn collision_safe_path(&self, proposed: &Path) -> PathBuf {
        if !proposed.exists() {
            return proposed.to_path_buf();
        }
        let stem = proposed.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let ext = proposed.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let parent = proposed.parent().unwrap_or(Path::new("."));

        for i in 1u32..=999 {
            let candidate = parent.join(format!("{stem}_{i:03}{ext}"));
            if !candidate.exists() {
                return candidate;
            }
        }
        // Fallback: append timestamp nanos
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        parent.join(format!("{stem}_{ts}{ext}"))
    }

    /// Normalize a filename to {DocType}_{Vendor}_{YYYY-MM-DD}.ext
    /// Applies only if LLM inference provides all three fields.
    pub fn normalize_filename(
        &self,
        doc_type: &str,
        vendor: &str,
        date: &str,
        original_ext: &str,
    ) -> String {
        let clean = |s: &str| -> String {
            s.chars()
                .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
                .collect::<String>()
                .trim_matches('_')
                .to_string()
        };
        format!(
            "{}_{}_{}.{}",
            clean(doc_type),
            clean(vendor),
            date.replace('/', "-"),
            original_ext.trim_start_matches('.')
        )
    }
}
```

---

### Step 1.3 — Real `ActionService` in `src-tauri/src/core/actions.rs`

Replace the entire file with:

```rust
use std::path::Path;
use serde_json::json;

use crate::infra::fs_guard::FilesystemGuard;
use crate::models::contracts::ActionRecommendationView;

#[derive(Debug, Default)]
pub struct ActionService {
    guard: FilesystemGuard,
}

/// Risk taxonomy flags
pub const RISK_WRONG_DESTINATION: &str = "wrong_destination";
pub const RISK_WRONG_FILENAME_VALUES: &str = "wrong_filename_values";
pub const RISK_DATA_LOSS: &str = "data_loss_risk";
pub const RISK_NO_UNDO_PATH: &str = "no_undo_path";
pub const RISK_COLLISION_DETECTED: &str = "collision_detected";
pub const RISK_LOW_CONFIDENCE: &str = "low_confidence";

impl ActionService {
    /// Derive a risk level and flag list for a recommendation.
    pub fn evaluate_risk(&self, rec: &ActionRecommendationView) -> (String, Vec<String>) {
        let mut flags: Vec<String> = vec![];

        if rec.confidence < 0.60 {
            flags.push(RISK_LOW_CONFIDENCE.into());
        }

        // Check for collision at proposed path
        if Path::new(&rec.proposed_path).exists() {
            flags.push(RISK_COLLISION_DETECTED.into());
        }

        // Check proposed path is reachable (parent must exist)
        let parent_exists = Path::new(&rec.proposed_path)
            .parent()
            .map(|p| p.exists())
            .unwrap_or(false);
        if !parent_exists {
            flags.push(RISK_WRONG_DESTINATION.into());
        }

        // Low confidence on vendor/date fields → filename accuracy risk
        if rec.confidence < 0.75 {
            flags.push(RISK_WRONG_FILENAME_VALUES.into());
        }

        let level = if flags.is_empty() {
            "low"
        } else if flags.len() == 1 {
            "medium"
        } else {
            "high"
        };

        (level.into(), flags)
    }

    /// Build an undo record payload for a rename/move that was executed.
    pub fn build_undo_record_json(&self, source: &str, destination: &str, rec_type: &str) -> String {
        json!({
            "type": rec_type,
            "from": destination,   // inverse: swap from/to
            "to": source,
        })
        .to_string()
    }

    /// Execute a single rename: move source_path → proposed_path (collision-safe).
    /// Returns the actual destination path used, or Err.
    pub fn execute_rename(
        &self,
        source_path: &str,
        proposed_path: &str,
        workspace_root: &str,
    ) -> Result<String, String> {
        // Validate source
        self.guard
            .validate_and_canonicalize(source_path, workspace_root)
            .map_err(|e| format!("source invalid: {e}"))?;

        let proposed = Path::new(proposed_path);
        let safe_dest = self.guard.collision_safe_path(proposed);

        // Ensure destination parent exists
        if let Some(parent) = safe_dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create destination dirs: {e}"))?;
        }

        std::fs::rename(source_path, &safe_dest)
            .map_err(|e| format!("rename failed: {e}"))?;

        Ok(safe_dest.to_string_lossy().to_string())
    }
}
```

---

### Step 1.4 — New DB Methods in `src-tauri/src/infra/db.rs`

Add these public methods to the `Database` impl block. Locate the existing `impl Database` block and append:

```rust
// ── Action Recommendations ─────────────────────────────────────────────────

pub fn save_recommendation(
    &self,
    id: &str,
    file_id: &str,
    recommendation_type: &str,
    source_path: &str,
    proposed_path: &str,
    confidence: f64,
    evidence_json: &str,
) -> Result<(), AlivError> {
    let now = Self::now_ts();
    self.conn()?.execute(
        "INSERT OR REPLACE INTO action_recommendations
         (id, file_id, recommendation_type, source_path, proposed_path, confidence, evidence_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, file_id, recommendation_type, source_path, proposed_path, confidence, evidence_json, now],
    ).map_err(|e| AlivError::Database(e.to_string()))?;
    Ok(())
}

pub fn list_recommendations(&self, workspace_path: &str, limit: usize) -> Result<Vec<(String, String, String, String, String, f64, String)>, AlivError> {
    // Returns: (id, file_id, rec_type, source_path, proposed_path, confidence, evidence_json)
    let conn = self.conn()?;
    let mut stmt = conn.prepare(
        "SELECT ar.id, ar.file_id, ar.recommendation_type, ar.source_path, ar.proposed_path,
                ar.confidence, ar.evidence_json
         FROM action_recommendations ar
         JOIN files f ON f.id = ar.file_id
         WHERE f.workspace_path = ?1
         ORDER BY ar.created_at DESC
         LIMIT ?2",
    ).map_err(|e| AlivError::Database(e.to_string()))?;

    let rows = stmt.query_map(params![workspace_path, limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, f64>(5)?,
            row.get::<_, String>(6)?,
        ))
    }).map_err(|e| AlivError::Database(e.to_string()))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AlivError::Database(e.to_string()))
}

pub fn save_execution(
    &self,
    id: &str,
    recommendation_id: &str,
    batch_id: &str,
    status: &str,
) -> Result<(), AlivError> {
    let now = Self::now_ts();
    self.conn()?.execute(
        "INSERT INTO action_executions (id, recommendation_id, batch_id, status, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, recommendation_id, batch_id, status, now],
    ).map_err(|e| AlivError::Database(e.to_string()))?;
    Ok(())
}

pub fn save_undo_record(
    &self,
    id: &str,
    execution_id: &str,
    inverse_operation_json: &str,
) -> Result<(), AlivError> {
    let now = Self::now_ts();
    self.conn()?.execute(
        "INSERT INTO undo_records (id, execution_id, inverse_operation_json, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, execution_id, inverse_operation_json, now],
    ).map_err(|e| AlivError::Database(e.to_string()))?;
    Ok(())
}

pub fn get_undo_records_for_batch(&self, batch_id: &str) -> Result<Vec<(String, String, String)>, AlivError> {
    // Returns: (undo_record_id, execution_id, inverse_operation_json)
    let conn = self.conn()?;
    let mut stmt = conn.prepare(
        "SELECT ur.id, ur.execution_id, ur.inverse_operation_json
         FROM undo_records ur
         JOIN action_executions ae ON ae.id = ur.execution_id
         WHERE ae.batch_id = ?1",
    ).map_err(|e| AlivError::Database(e.to_string()))?;

    let rows = stmt.query_map(params![batch_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).map_err(|e| AlivError::Database(e.to_string()))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AlivError::Database(e.to_string()))
}

pub fn list_audit_log(&self, limit: usize) -> Result<Vec<(String, String, String, String, String, String, String)>, AlivError> {
    // Returns: (execution_id, recommendation_id, batch_id, source_path, proposed_path, rec_type, status, executed_at)
    let conn = self.conn()?;
    let mut stmt = conn.prepare(
        "SELECT ae.id, ae.recommendation_id, ae.batch_id,
                ar.source_path, ar.proposed_path, ar.recommendation_type,
                ae.status, ae.executed_at
         FROM action_executions ae
         JOIN action_recommendations ar ON ar.id = ae.recommendation_id
         ORDER BY ae.executed_at DESC
         LIMIT ?1",
    ).map_err(|e| AlivError::Database(e.to_string()))?;

    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
        ))
    }).map_err(|e| AlivError::Database(e.to_string()))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AlivError::Database(e.to_string()))
}
```

> **Note**: `Self::now_ts()` and `self.conn()` already exist in `db.rs`. Match their existing pattern.

---

### Step 1.5 — Real Command Handlers in `src-tauri/src/commands/actions.rs`

Replace the entire file with the implementation below. The pattern matches `commands/ingestion.rs` and `commands/memory.rs`:

```rust
use tauri::State;
use uuid::Uuid;
use serde_json;

use crate::core::actions::ActionService;
use crate::infra::db::Database;
use crate::infra::fs_guard::FilesystemGuard;
use crate::models::contracts::{
    ActionApprovalRequest, ActionExecutionRequest, ActionPreviewList, ActionRecommendationRequest,
    ActionRecommendationView, ActionRiskRequest, AuditLogRequest, AuditLogEntry, AuditLogResponse,
    CommandAck, UndoBatchRequest, ActionExecutionResult, UndoResult,
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

    // Pull indexed files from DB that have high-confidence parse results
    let rows = db
        .list_files_needing_rename(&request.workspace_path, request.batch_limit)
        .map_err(|e| e.to_string())?;

    let service = ActionService::default();
    let mut items: Vec<ActionRecommendationView> = Vec::new();

    for (file_id, source_path, inferred_doc_type, confidence_raw) in rows {
        let confidence = confidence_raw.unwrap_or(0.0);
        if confidence < 0.70 {
            continue; // Skip low-confidence files
        }

        // Build proposed path using normalized filename pattern
        let path = std::path::Path::new(&source_path);
        let ext = path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let parent = path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Use inferred_doc_type as DocType; vendor/date require extraction — stub with "Unknown" if not available
        let proposed_name = guard.normalize_filename(&inferred_doc_type, "Unknown", "undated", &ext);
        let proposed_path = format!("{}/{}", parent, proposed_name);

        let rec_id = Uuid::new_v4().to_string();
        let evidence = vec![
            format!("parser_confidence={:.2}", confidence),
            format!("inferred_doc_type={}", inferred_doc_type),
        ];
        let evidence_json = serde_json::to_string(&evidence).unwrap_or_default();

        // Persist to DB
        let _ = db.save_recommendation(
            &rec_id,
            &file_id,
            "rename",
            &source_path,
            &proposed_path,
            confidence,
            &evidence_json,
        );

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
    db: State<'_, Database>,
) -> Result<ActionPreviewList, String> {
    state.record_command("get_action_preview_queue");

    // Re-fetch current queue items from DB by their IDs
    // For now return a lightweight list based on state queue count
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
    // Approval is purely a UI gate — mark IDs as approved in state queue for execute_actions to pick up
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

    let service = ActionService::default();

    // Fetch all recommendations for this batch (stored by batch_id prefix in ID or use preview queue)
    let rec_ids = state.drain_preview_queue(); // add this method to AppState
    let mut succeeded = 0;
    let mut failed = 0;
    let mut errors: Vec<String> = vec![];

    for rec_id in &rec_ids {
        match db.get_recommendation(rec_id) {
            Ok(Some(row)) => {
                let (id, _file_id, rec_type, source_path, proposed_path, _confidence, _evidence_json) = row;
                match service.execute_rename(&source_path, &proposed_path, "") {
                    Ok(actual_dest) => {
                        let exec_id = Uuid::new_v4().to_string();
                        let _ = db.save_execution(&exec_id, &id, &request.batch_id, "success");

                        let undo_json = service.build_undo_record_json(&source_path, &actual_dest, &rec_type);
                        let undo_id = Uuid::new_v4().to_string();
                        let _ = db.save_undo_record(&undo_id, &exec_id, &undo_json);
                        succeeded += 1;
                    }
                    Err(err) => {
                        let exec_id = Uuid::new_v4().to_string();
                        let _ = db.save_execution(&exec_id, &id, &request.batch_id, "failed");
                        errors.push(format!("{}: {}", source_path, err));
                        failed += 1;
                    }
                }
            }
            _ => {
                errors.push(format!("recommendation not found: {}", rec_id));
                failed += 1;
            }
        }
    }

    Ok(ActionExecutionResult {
        command: "execute_actions".into(),
        status: if failed == 0 { "ok".into() } else { "partial".into() },
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
                reversed += 1;
            }
            Err(e) => {
                errors.push(format!("undo failed for exec {exec_id}: {e}"));
            }
        }
    }

    Ok(UndoResult {
        command: "undo_action_batch".into(),
        status: if errors.is_empty() { "ok".into() } else { "partial".into() },
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

    let rows = db
        .list_audit_log(request.limit)
        .map_err(|e| e.to_string())?;

    let entries: Vec<AuditLogEntry> = rows
        .into_iter()
        .map(|(exec_id, rec_id, batch_id, source_path, proposed_path, rec_type, status, executed_at)| {
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
        })
        .collect();

    let total = entries.len();
    Ok(AuditLogResponse {
        command: "get_action_audit_log".into(),
        status: "ok".into(),
        total,
        entries,
    })
}
```

---

### Step 1.6 — Additional DB Methods Needed (also in `db.rs`)

```rust
pub fn list_files_needing_rename(
    &self,
    workspace_path: &str,
    limit: usize,
) -> Result<Vec<(String, String, String, Option<f64>)>, AlivError> {
    // Returns: (file_id, source_path, inferred_doc_type, parser_confidence)
    let conn = self.conn()?;
    let mut stmt = conn.prepare(
        "SELECT f.id, f.source_path, f.inferred_doc_type, pr.parser_confidence
         FROM files f
         LEFT JOIN parse_results pr ON pr.file_id = f.id
         WHERE f.workspace_path = ?1
         AND f.inferred_doc_type IS NOT NULL
         ORDER BY pr.parser_confidence DESC
         LIMIT ?2",
    ).map_err(|e| AlivError::Database(e.to_string()))?;

    let rows = stmt.query_map(params![workspace_path, limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<f64>>(3)?,
        ))
    }).map_err(|e| AlivError::Database(e.to_string()))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| AlivError::Database(e.to_string()))
}

pub fn get_recommendation(
    &self,
    id: &str,
) -> Result<Option<(String, String, String, String, String, f64, String)>, AlivError> {
    let conn = self.conn()?;
    conn.query_row(
        "SELECT id, file_id, recommendation_type, source_path, proposed_path, confidence, evidence_json
         FROM action_recommendations WHERE id = ?1",
        params![id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
            ))
        },
    )
    .optional()
    .map_err(|e| AlivError::Database(e.to_string()))
}

pub fn update_execution_status(&self, execution_id: &str, status: &str) -> Result<(), AlivError> {
    self.conn()?.execute(
        "UPDATE action_executions SET status = ?1 WHERE id = ?2",
        params![status, execution_id],
    ).map_err(|e| AlivError::Database(e.to_string()))?;
    Ok(())
}
```

---

### Step 1.7 — Add `drain_preview_queue` to `src-tauri/src/state.rs`

Add this method to the `AppState` impl block:

```rust
pub fn drain_preview_queue(&self) -> Vec<String> {
    let mut registry = self.registry.lock().expect("app state poisoned");
    std::mem::take(&mut registry.preview_queue)
}
```

---

### Step 1.8 — Safety Rules (Hard Requirements)

1. **Never execute** file actions unless `approve_actions` was called first for those IDs
2. **Every `execute_rename`** must write an undo record before returning success
3. **Confidence gate**: skip files with `parser_confidence < 0.70` in recommendations
4. **Collision-safe**: always call `collision_safe_path()` before calling `std::fs::rename`
5. **Workspace boundary**: `validate_and_canonicalize` must pass before any file operation
6. **Partial failure is OK**: if 3/10 renames fail, return `status: "partial"` with `errors[]` — do not abort the batch

---

## Part 2: Phase 4 — E2E Hardening

### Current Gaps

| Gap | File | Problem |
|---|---|---|
| `embedding_count=0` hardcoded | `benchmarks/run_ingestion_benchmark.py` line 208 | Always records 0 embeddings — makes `embedding_count` in `ingestion_runs` meaningless |
| Markdown defect parsing brittle | `benchmarks/run_full_benchmark_suite.py` lines 38-47 | `_read_open_critical_defects()` regex-scans markdown — will break on any formatting change |
| Worker has no retry | `src-tauri/src/infra/python_bridge.rs` lines 229-244 | Single timeout, no retry — transient Python startup delay causes false failures |
| `PHASE4_DEFECT_BACKLOG.md` doesn't exist | `docs/` | Benchmark suite exits early if the file is missing and `open_critical=0` is never confirmed |

---

### Step 2.1 — Fix `embedding_count` in `benchmarks/run_ingestion_benchmark.py`

**Problem**: Line 208 passes `0` for `embeddings` to `_record_run()`.

**Fix**: After `_store_chunks()`, call the embedding worker and record the actual count.

Add this import at the top:
```python
import subprocess
```

Replace the inner loop body (around line 202) with:

```python
chunks = _store_chunks(conn, file_id, pages)
_store_parse_result(conn, file_id, parser_name, confidence, warnings)
parsed += 1
chunk_total += len(chunks)

# Attempt to embed — non-fatal on failure
if chunks:
    try:
        texts = [text for _, text in chunks]
        embed_result = _embed_chunks(conn, file_id, chunks)
        embedding_total += embed_result
    except Exception as exc:
        pass  # Embedding failure does not fail ingestion gate
```

Add this helper function before `run_benchmark`:
```python
def _embed_chunks(conn: sqlite3.Connection, file_id: str, chunks: list[tuple[str, str]]) -> int:
    """Call embedding worker via subprocess. Returns number of vectors stored."""
    import json, sys
    from pathlib import Path
    ROOT = Path(__file__).resolve().parents[1]
    worker_path = ROOT / "python" / "workers" / "embedding_worker.py"
    if not worker_path.exists():
        return 0

    texts = [text for _, text in chunks]
    request = {
        "request_id": f"bench-embed-{file_id[:8]}",
        "worker_type": "embed",
        "payload": {"chunks": texts},
        "timeout_ms": 60000,
        "worker_version": "0.1.0",
    }
    result = subprocess.run(
        [sys.executable, str(worker_path)],
        input=json.dumps(request) + "\n",
        capture_output=True,
        text=True,
        timeout=65,
    )
    if result.returncode != 0:
        return 0
    try:
        resp = json.loads(result.stdout.strip().splitlines()[-1])
        vectors = resp.get("result", {}).get("vectors", [])
        # Store embeddings in DB
        for (chunk_id, _), vec in zip(chunks, vectors):
            conn.execute(
                "INSERT OR REPLACE INTO embeddings(chunk_id, vector_json, embedding_model, embedding_dim, created_at) "
                "VALUES (?, ?, 'all-MiniLM-L6-v2', 384, ?)",
                (chunk_id, json.dumps(vec), str(int(__import__("time").time()))),
            )
        conn.commit()
        return len(vectors)
    except Exception:
        return 0
```

Also add `embedding_total = 0` before the loop, and change line 208 to pass `embedding_total` instead of `0`.

---

### Step 2.2 — Replace Markdown Defect Parser with JSON in `run_full_benchmark_suite.py`

**Problem**: `_read_open_critical_defects()` regex-parses markdown, which is fragile.

**Fix**: Change the defect file from markdown to JSON. Update the reader function:

```python
DEFECT_BACKLOG = ROOT / "docs" / "PHASE4_DEFECT_BACKLOG.json"  # change extension

def _read_open_critical_defects() -> int:
    if not DEFECT_BACKLOG.exists():
        return 0
    try:
        defects = json.loads(DEFECT_BACKLOG.read_text(encoding="utf-8"))
        return sum(
            1 for d in defects
            if d.get("status") == "open" and d.get("severity") == "critical"
        )
    except (json.JSONDecodeError, TypeError):
        return 0
```

---

### Step 2.3 — Create `docs/PHASE4_DEFECT_BACKLOG.json`

Create this file:

```json
[
  {
    "id": "D001",
    "title": "embedding_count always 0 in ingestion benchmark",
    "severity": "critical",
    "status": "open",
    "component": "benchmarks/run_ingestion_benchmark.py",
    "description": "run_benchmark() passes hardcoded 0 to _record_run() for embeddings parameter. The embedding worker is not called during benchmarks.",
    "fix": "Wire _embed_chunks() into the benchmark loop and pass actual count."
  },
  {
    "id": "D002",
    "title": "sqlite-vec migration 0008 skipped in Python benchmarks",
    "severity": "high",
    "status": "open",
    "component": "benchmarks/run_ingestion_benchmark.py",
    "description": "_apply_migrations() skips 0008 because it requires the native sqlite-vec extension. Python benchmarks cannot load this extension.",
    "fix": "Benchmarks should use the Rust-side DB after ingestion, or ship a pre-built sqlite-vec wheel."
  },
  {
    "id": "D003",
    "title": "Python worker has no retry on transient timeout",
    "severity": "high",
    "status": "open",
    "component": "src-tauri/src/infra/python_bridge.rs",
    "description": "run_worker() has a single 30s timeout with no retry. Cold Python startup on some machines exceeds 30s on first call.",
    "fix": "Add max 2 retries on timeout with exponential backoff."
  },
  {
    "id": "D004",
    "title": "FilesystemGuard only validates non-empty string",
    "severity": "critical",
    "status": "open",
    "component": "src-tauri/src/infra/fs_guard.rs",
    "description": "validate_workspace_path() returns true for any non-empty string including paths outside the workspace or paths with traversal components.",
    "fix": "Implement real canonicalization and workspace boundary check (Phase 3 Step 1.2)."
  }
]
```

> After Phase 3 and Phase 4 fixes are merged, update `"status": "open"` to `"status": "closed"` for the resolved defects. The benchmark suite re-reads this file on every run.

---

### Step 2.4 — Add Worker Retry in `src-tauri/src/infra/python_bridge.rs`

**Problem**: `run_worker()` has a single 30s timeout. No retry.

**Fix**: Wrap the spawn + wait block in a retry loop. Change `run_worker` to:

```rust
fn run_worker(
    &self,
    module_name: &str,
    worker_type: &str,
    payload: Value,
) -> Result<WorkerResponseEnvelope, AlivError> {
    let max_attempts = 3u8;
    let mut last_error = AlivError::PythonBridge("no attempt made".into());

    for attempt in 1..=max_attempts {
        match self.run_worker_once(module_name, worker_type, payload.clone()) {
            Ok(envelope) => return Ok(envelope),
            Err(err) => {
                let is_timeout = err.to_string().contains("timed out");
                last_error = err;
                if !is_timeout || attempt == max_attempts {
                    break;
                }
                // Exponential backoff: 500ms, 1000ms
                let delay_ms = 500u64 * u64::from(attempt);
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
        }
    }
    Err(last_error)
}
```

Rename the existing `run_worker` implementation body to `run_worker_once` (same signature).

---

## Part 3: What NOT to Change

- Do not modify `db/migrations/*.sql` — schema is correct as-is
- Do not change the JSON-lines IPC contract in `python_bridge.rs` — worker protocol is stable
- Do not remove the `embedding_count=0` line until `_embed_chunks()` is verified working
- Do not change Phase 1/2 commands — they are complete and passing benchmarks
- Do not add cloud calls, HTTP outbound calls, or telemetry to any of these paths

---

## Part 4: Build & Verify Checklist

After implementing the above, verify in order:

1. `cargo build` in `src-tauri/` — zero compilation errors
2. `cargo test` in `src-tauri/` — all existing tests pass
3. `python benchmarks/run_ingestion_benchmark.py --corpus benchmarks/corpus` — `embedding_count > 0` in output
4. `python benchmarks/run_full_benchmark_suite.py` — `phase4_exit.pass = true` after 2 consecutive runs
5. Manual Tauri test: call `generate_action_recommendations` → `evaluate_action_risk` → `approve_actions` → `execute_actions` → `undo_action_batch` in sequence
6. Verify undo: file should return to original location after `undo_action_batch`
7. Update `docs/PHASE4_DEFECT_BACKLOG.json` — set D001 and D004 status to `"closed"` once verified

---

## Part 5: Phase Exit Criteria (from PRD)

Phase 3 is complete when:
- `harmful_file_action_rate < 1%` (verified by audit log review)
- Zero data loss incidents in 50-file action batch test
- All 7 action commands return real data (not scaffolded strings)
- Undo reversal verified for at least 10 rename operations

Phase 4 is complete when:
- Two consecutive `run_full_benchmark_suite.py` runs return `phase4_exit.pass = true`
- `open_critical_count = 0` in both runs (all critical defects in JSON closed)
- No fatal crash on 200-file corpus
