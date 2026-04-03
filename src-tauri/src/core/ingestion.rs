use crate::{
    infra::{db::Database, python_bridge::PythonBridge},
    models::{contracts::IngestionRequest, errors::AlivError},
};
use sha2::{Digest, Sha256};

const SUPPORTED_TYPES: [&str; 10] =
    ["pdf", "docx", "eml", "png", "jpg", "jpeg", "tif", "tiff", "bmp", "webp"];

#[derive(Debug)]
pub struct IngestionPipelineResult {
    pub run_id: String,
    pub workspace_path: String,
    pub incremental: bool,
    pub discovered_files: usize,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub parsed_files: usize,
    pub chunk_count: usize,
    pub embedding_count: usize,
    pub failed_files: usize,
}

#[derive(Debug, Default)]
pub struct IngestionService;

impl IngestionService {
    /// Run the full ingestion pipeline: scan → parse → chunk → embed (batched).
    pub fn run(
        &self,
        database: &Database,
        bridge: &PythonBridge,
        request: &IngestionRequest,
    ) -> Result<IngestionPipelineResult, AlivError> {
        let mut run = database.run_ingestion_scan(&request.workspace_path, request.incremental)?;

        let mut parsed_files = 0usize;
        let mut chunk_count = 0usize;
        let mut failed_files = 0usize;
        let mut parsed_documents: Vec<(String, String)> = Vec::new(); // (file_id, normalized_text)

        // Collect ALL chunk texts + ids from every file in one pass, then embed in one batch.
        let mut all_chunks: Vec<(String, String)> = Vec::new(); // (chunk_id, text)

        for file in &run.discovered {
            if !file.should_parse || !SUPPORTED_TYPES.contains(&file.file_type.as_str()) {
                continue;
            }

            let parsed = match bridge.parse_document(&file.source_path, &file.file_type) {
                Ok(p) => p,
                Err(_) => {
                    failed_files += 1;
                    continue;
                }
            };

            let chunk_data = match database.replace_chunks_for_file(&file.id, &parsed) {
                Ok(cd) => cd,
                Err(_) => {
                    failed_files += 1;
                    continue;
                }
            };

            parsed_files += 1;
            chunk_count += chunk_data.len();
            all_chunks.extend(chunk_data);
            parsed_documents.push((file.id.clone(), parsed.normalized_text.clone()));
        }

        // Single embed call for all chunks across the entire run.
        let embedding_count = if !all_chunks.is_empty() {
            let texts: Vec<String> = all_chunks.iter().map(|(_, t)| t.clone()).collect();
            match bridge.embed_documents(&texts) {
                Ok(result) => {
                    let chunk_vectors: Vec<(String, Vec<f32>)> = all_chunks
                        .iter()
                        .zip(result.vectors.iter())
                        .map(|((id, _), vec)| (id.clone(), vec.clone()))
                        .collect();
                    database.store_embeddings(&chunk_vectors, &result.embedding_model).unwrap_or(0)
                }
                Err(_) => 0,
            }
        } else {
            0
        };

        run.parsed_files = parsed_files;
        run.chunk_count = chunk_count;

        database.finalize_ingestion_run(
            &run.run_id,
            parsed_files,
            chunk_count,
            embedding_count,
            failed_files,
        )?;

        // Post-ingestion summary cache generation (best-effort, non-blocking for ingestion success).
        for (file_id, normalized_text) in parsed_documents {
            if normalized_text.trim().is_empty() {
                continue;
            }

            let file_hash = match database.get_file_hash(&file_id)? {
                Some(hash) => hash,
                None => continue,
            };

            let _ = database.invalidate_document_summaries(&file_id, &file_hash);

            if database.get_document_summary(&file_id, "document")?.is_some() {
                continue;
            }

            let summary = match bridge.summarize_text(&normalized_text, "document", "llama3.2:3b") {
                Ok(value) => value,
                Err(_) => continue,
            };

            let summary_id = stable_summary_id(&file_id, &file_hash, &summary.granularity);
            let _ = database.save_document_summary(
                &summary_id,
                &file_id,
                &file_hash,
                &summary.granularity,
                None,
                &summary.summary,
                summary.token_count as i64,
                &summary.model_version,
            );
        }

        Ok(IngestionPipelineResult {
            run_id: run.run_id,
            workspace_path: request.workspace_path.clone(),
            incremental: request.incremental,
            discovered_files: run.discovered_files,
            indexed_files: run.indexed_files,
            skipped_files: run.skipped_files,
            parsed_files,
            chunk_count,
            embedding_count,
            failed_files,
        })
    }

    pub fn summary_detail(&self, r: &IngestionPipelineResult) -> String {
        let mode = if r.incremental { "incremental" } else { "full" };
        format!(
            "{mode} ingestion run {run_id} completed for {ws} \
             discovered={d} indexed={i} skipped={s} parsed={p} chunks={c} embeddings={e} failed={f}",
            run_id = r.run_id,
            ws = r.workspace_path,
            d = r.discovered_files,
            i = r.indexed_files,
            s = r.skipped_files,
            p = r.parsed_files,
            c = r.chunk_count,
            e = r.embedding_count,
            f = r.failed_files,
        )
    }
}

fn stable_summary_id(file_id: &str, file_hash: &str, granularity: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_id.as_bytes());
    hasher.update(file_hash.as_bytes());
    hasher.update(granularity.as_bytes());
    format!("{:x}", hasher.finalize())
}
