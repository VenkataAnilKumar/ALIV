#[derive(Debug, Default)]
pub struct TelemetryService;

impl TelemetryService {
    pub fn record_run(&self, run_id: &str) -> String {
        format!("telemetry recorded for {}", run_id)
    }
}
