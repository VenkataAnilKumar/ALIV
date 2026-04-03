use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::errors::AlivError;

#[derive(Debug, Clone)]
pub struct PythonBridge {
    python_executable: String,
    worker_version: String,
    python_root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct WorkerRequestEnvelope {
    request_id: String,
    worker_type: String,
    payload: Value,
    timeout_ms: u64,
    worker_version: String,
}

#[derive(Debug, Deserialize)]
struct WorkerResponseEnvelope {
    request_id: String,
    status: String,
    #[serde(default)]
    result: Value,
    error_code: Option<String>,
    error_detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedPage {
    pub page_number: usize,
    pub text: String,
    pub block_count: usize,
    pub has_text: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedDocument {
    pub file_path: String,
    #[serde(default = "default_parser_name")]
    pub parser_name: String,
    pub normalized_text: String,
    #[serde(default)]
    pub page_map: Vec<Value>,
    #[serde(default)]
    pub pages: Vec<ParsedPage>,
    pub parser_confidence: f64,
    #[serde(default)]
    pub warnings: Vec<String>,
}

fn default_parser_name() -> String {
    "unknown".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResult {
    pub vectors: Vec<Vec<f32>>,
    pub embedding_model: String,
    pub embedding_dim: usize,
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SummarizationResult {
    pub summary: String,
    pub token_count: usize,
    pub model_version: String,
    pub granularity: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractionResult {
    #[serde(default)]
    pub field_value_map: Value,
    #[serde(default)]
    pub field_confidence_map: Value,
    #[serde(default)]
    pub citations: Vec<Value>,
    pub template: String,
    pub query: String,
}

impl PythonBridge {
    pub fn new() -> Result<Self, AlivError> {
        let python_root = std::env::current_dir()
            .map_err(|err| AlivError::PythonBridge(err.to_string()))?
            .join("python");

        Ok(Self {
            python_executable: std::env::var("ALIV_PYTHON").unwrap_or_else(|_| "python".into()),
            worker_version: "0.1.0".into(),
            python_root,
        })
    }

    pub fn worker_handshake(&self, worker_type: &str) -> String {
        format!("python bridge ready for {} worker via {}", worker_type, self.python_executable)
    }

    pub fn parse_document(&self, file_path: &str, file_type: &str) -> Result<ParsedDocument, AlivError> {
        let response = self.run_worker(
            "workers.parse_worker",
            "parse",
            json!({ "file_path": file_path, "file_type": file_type }),
        )?;

        serde_json::from_value(response.result)
            .map_err(|err| AlivError::PythonBridge(err.to_string()))
    }

    pub fn embed_documents(&self, texts: &[String]) -> Result<EmbeddingResult, AlivError> {
        if texts.is_empty() {
            return Err(AlivError::PythonBridge(
                "embed_documents requires non-empty text list".into(),
            ));
        }

        let response = self.run_worker(
            "workers.embedding_worker",
            "embed",
            json!({ "chunks": texts }),
        )?;

        serde_json::from_value(response.result)
            .map_err(|err| AlivError::PythonBridge(err.to_string()))
    }

    pub fn summarize_text(
        &self,
        text: &str,
        granularity: &str,
        model: &str,
    ) -> Result<SummarizationResult, AlivError> {
        if text.trim().is_empty() {
            return Err(AlivError::PythonBridge(
                "summarize_text requires non-empty text".into(),
            ));
        }

        let response = self.run_worker(
            "workers.summarization_worker",
            "summarize",
            json!({
                "text": text,
                "granularity": granularity,
                "model": model,
            }),
        )?;

        serde_json::from_value(response.result)
            .map_err(|err| AlivError::PythonBridge(err.to_string()))
    }

    pub fn extract_fields(
        &self,
        query: &str,
        template: &str,
        chunks: &[Value],
    ) -> Result<ExtractionResult, AlivError> {
        if query.trim().is_empty() {
            return Err(AlivError::PythonBridge(
                "extract_fields requires non-empty query".into(),
            ));
        }
        if template.trim().is_empty() {
            return Err(AlivError::PythonBridge(
                "extract_fields requires non-empty template".into(),
            ));
        }

        let response = self.run_worker(
            "workers.extraction_worker",
            "extract",
            json!({
                "query": query,
                "template": template,
                "chunks": chunks,
            }),
        )?;

        serde_json::from_value(response.result)
            .map_err(|err| AlivError::PythonBridge(err.to_string()))
    }

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
                    let delay_ms = 500u64 * u64::from(attempt);
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                }
            }
        }

        Err(last_error)
    }

    fn run_worker_once(
        &self,
        module_name: &str,
        worker_type: &str,
        payload: Value,
    ) -> Result<WorkerResponseEnvelope, AlivError> {
        let request = WorkerRequestEnvelope {
            request_id: format!("{}-{}", worker_type, current_unix_timestamp()),
            worker_type: worker_type.to_string(),
            payload,
            timeout_ms: 30_000,
            worker_version: self.worker_version.clone(),
        };

        let mut child = Command::new(&self.python_executable)
            .args(["-m", module_name])
            .current_dir(&self.python_root)
            .env("PYTHONIOENCODING", "utf-8")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| AlivError::PythonBridge(err.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            let payload = serde_json::to_string(&request)
                .map_err(|err| AlivError::PythonBridge(err.to_string()))?;
            stdin
                .write_all(payload.as_bytes())
                .and_then(|_| stdin.write_all(b"\n"))
                .map_err(|err| AlivError::PythonBridge(err.to_string()))?;
        }

        // Enforce a hard timeout so corrupt files cannot hang the ingestion run.
        let timeout = std::time::Duration::from_millis(30_000);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(child.wait_with_output());
        });

        let output = match rx.recv_timeout(timeout) {
            Ok(Ok(out)) => out,
            Ok(Err(err)) => return Err(AlivError::PythonBridge(err.to_string())),
            Err(_) => {
                return Err(AlivError::PythonBridge(format!(
                    "worker '{}' timed out after 30s",
                    worker_type
                )))
            }
        };

        if !output.status.success() {
            return Err(AlivError::PythonBridge(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|err| AlivError::PythonBridge(err.to_string()))?;
        let line = stdout
            .lines()
            .find(|value| !value.trim().is_empty())
            .ok_or_else(|| AlivError::PythonBridge("worker returned empty output".into()))?;
        let response: WorkerResponseEnvelope = serde_json::from_str(line)
            .map_err(|err| AlivError::PythonBridge(err.to_string()))?;

        if response.status != "ok" {
            return Err(AlivError::PythonBridge(format!(
                "worker={} request_id={} error_code={} error_detail={}",
                worker_type,
                response.request_id,
                response.error_code.unwrap_or_else(|| "unknown".into()),
                response.error_detail.unwrap_or_else(|| "unknown".into())
            )));
        }

        Ok(response)
    }
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
