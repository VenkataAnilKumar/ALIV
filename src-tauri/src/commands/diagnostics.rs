use std::{
    fs,
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde_json::json;
use tauri::{AppHandle, Manager, State};

use crate::{infra::db::Database, state::AppState};

#[tauri::command]
pub fn get_diagnostics_report(
    app: AppHandle,
    db: State<'_, Database>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let db_snapshot = db.diagnostics_snapshot().map_err(|err| err.to_string())?;
    let runtime = app_state.snapshot();
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|err| format!("failed to resolve log dir: {err}"))?;
    let log_file = newest_log_file(&log_dir);
    let report = json!({
        "aliv_version": env!("CARGO_PKG_VERSION"),
        "platform": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
        "ollama": {
            "reachable": ollama_reachable(),
            "endpoint": "http://127.0.0.1:11434",
        },
        "db": db_snapshot,
        "runtime": {
            "workspace_count": runtime.workspace_count,
            "command_count": runtime.command_count,
            "preview_queue_count": runtime.preview_queue_count,
            "benchmark_run_count": runtime.benchmark_run_count,
            "last_ingestion_target": runtime.last_ingestion_target,
        },
        "logs": {
            "directory": log_dir.display().to_string(),
            "active_file": log_file
                .as_ref()
                .map(|path| path.display().to_string()),
            "tail": log_file
                .as_ref()
                .map(|path| read_log_tail(path, 80))
                .unwrap_or_else(|| "No log file found yet.".into()),
        },
        "generated_at": chrono::Utc::now().to_rfc3339(),
    });

    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn open_log_directory(app: AppHandle) -> Result<String, String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|err| format!("failed to resolve log dir: {err}"))?;

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)
            .map_err(|err| format!("failed to create log directory: {err}"))?;
    }

    let path = log_dir.display().to_string();
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("explorer");
        cmd.arg(&path);
        cmd
    } else if cfg!(target_os = "macos") {
        let mut cmd = Command::new("open");
        cmd.arg(&path);
        cmd
    } else {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(&path);
        cmd
    };

    command
        .spawn()
        .map_err(|err| format!("failed to open log directory: {err}"))?;

    Ok(path)
}

fn ollama_reachable() -> bool {
    ["127.0.0.1:11434", "localhost:11434"].iter().any(|candidate| {
        candidate.to_socket_addrs().ok().is_some_and(|addrs| {
            addrs.into_iter().any(|addr| {
                TcpStream::connect_timeout(&addr, Duration::from_millis(400)).is_ok()
            })
        })
    })
}

fn newest_log_file(log_dir: &Path) -> Option<PathBuf> {
    let mut entries = fs::read_dir(log_dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("log"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    });
    entries.pop()
}

fn read_log_tail(path: &Path, max_lines: usize) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let lines = contents.lines().collect::<Vec<_>>();
            let start = lines.len().saturating_sub(max_lines);
            lines[start..].join("\n")
        }
        Err(err) => format!("Unable to read log file: {err}"),
    }
}