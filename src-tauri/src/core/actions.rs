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

        // Low confidence on vendor/date fields -> filename accuracy risk
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
            "from": destination,
            "to": source,
        })
        .to_string()
    }

    /// Execute a single rename: move source_path -> proposed_path (collision-safe).
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

        std::fs::rename(source_path, &safe_dest).map_err(|e| format!("rename failed: {e}"))?;

        Ok(safe_dest.to_string_lossy().to_string())
    }
}
