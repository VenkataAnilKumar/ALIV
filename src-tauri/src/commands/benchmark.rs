use std::{fs, path::{Path, PathBuf}, process::Command};

use serde_json::Value;
use tauri::State;

use crate::{
    models::contracts::{BenchmarkRequest, CommandAck, RunMetricsRequest},
    state::AppState,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

fn resolve_python_executable(root: &Path) -> String {
    if let Ok(value) = std::env::var("ALIV_PYTHON") {
        return value;
    }

    let windows_venv = root.join(".venv").join("Scripts").join("python.exe");
    if windows_venv.exists() {
        return windows_venv.to_string_lossy().to_string();
    }

    let unix_venv = root.join(".venv").join("bin").join("python");
    if unix_venv.exists() {
        return unix_venv.to_string_lossy().to_string();
    }

    "python".into()
}

fn extract_run_id(detail: &str) -> Option<String> {
    let payload: Value = serde_json::from_str(detail).ok()?;
    payload
        .get("result_files")
        .and_then(|value| value.get("stamped"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn resolve_report_path(run_id: &str) -> PathBuf {
    let root = repo_root();
    let results_dir = root.join("benchmarks").join("results");
    if run_id.trim().is_empty() || run_id == "latest" {
        return results_dir.join("full_suite_latest.json");
    }

    let raw = PathBuf::from(run_id);
    if raw.exists() {
        return raw;
    }

    results_dir.join(run_id)
}

#[tauri::command]
pub fn run_benchmark_suite(
    state: State<'_, AppState>,
    request: BenchmarkRequest,
) -> Result<CommandAck, String> {
    state.record_command("run_benchmark_suite");

    let root = repo_root();
    let python = resolve_python_executable(&root);
    let questions_path = if request.labels_version.trim().is_empty() {
        "benchmarks/labels/questions_gold.v1.json".to_string()
    } else if request.labels_version.ends_with(".json") {
        format!("benchmarks/labels/{}", request.labels_version)
    } else {
        format!("benchmarks/labels/{}.json", request.labels_version)
    };

    let output = Command::new(&python)
        .args([
            "benchmarks/run_full_benchmark_suite.py",
            "--questions",
            &questions_path,
        ])
        .current_dir(&root)
        .output()
        .map_err(|err| format!("failed to run benchmark suite via '{}': {}", python, err))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if stdout.is_empty() && !output.status.success() {
        return Err(if stderr.is_empty() {
            "benchmark suite failed without output".into()
        } else {
            stderr
        });
    }

    if let Some(run_id) = extract_run_id(&stdout) {
        state.record_benchmark_run(&run_id);
    }

    Ok(CommandAck {
        command: "run_benchmark_suite".into(),
        status: if output.status.success() { "ok" } else { "partial" }.into(),
        detail: if stdout.is_empty() { stderr } else { stdout },
    })
}

#[tauri::command]
pub fn get_run_metrics(
    state: State<'_, AppState>,
    request: RunMetricsRequest,
) -> Result<CommandAck, String> {
    state.record_command("get_run_metrics");

    let report_path = resolve_report_path(&request.run_id);
    let detail = fs::read_to_string(&report_path)
        .map_err(|err| format!("failed to read benchmark report '{}': {}", report_path.display(), err))?;

    let status = serde_json::from_str::<Value>(&detail)
        .ok()
        .and_then(|value| value.get("status").and_then(|status| status.as_str()).map(str::to_string))
        .unwrap_or_else(|| {
            if state.has_benchmark_run(&request.run_id) || report_path.exists() {
                "ok".into()
            } else {
                "missing".into()
            }
        });

    Ok(CommandAck {
        command: "get_run_metrics".into(),
        status,
        detail,
    })
}
