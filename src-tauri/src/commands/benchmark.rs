use tauri::State;

use crate::{
    core::telemetry::TelemetryService,
    models::contracts::{BenchmarkRequest, CommandAck, RunMetricsRequest},
    state::AppState,
};

#[tauri::command]
pub fn run_benchmark_suite(
    state: State<'_, AppState>,
    request: BenchmarkRequest,
) -> Result<CommandAck, String> {
    let run_id = format!("{}::{}", request.corpus_manifest, request.labels_version);
    let service = TelemetryService;
    state.record_command("run_benchmark_suite");
    state.record_benchmark_run(&run_id);

    Ok(CommandAck {
        command: "run_benchmark_suite".into(),
        status: "ok".into(),
        detail: service.record_run(&run_id),
    })
}

#[tauri::command]
pub fn get_run_metrics(
    state: State<'_, AppState>,
    request: RunMetricsRequest,
) -> Result<CommandAck, String> {
    state.record_command("get_run_metrics");

    Ok(CommandAck {
        command: "get_run_metrics".into(),
        status: if state.has_benchmark_run(&request.run_id) { "ok" } else { "missing" }.into(),
        detail: format!("run metrics requested for {}", request.run_id),
    })
}
