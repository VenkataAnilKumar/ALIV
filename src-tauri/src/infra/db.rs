use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, params_from_iter, Connection, OpenFlags, OptionalExtension};
use serde_json::json;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use sqlite_vec::sqlite3_vec_init;

use crate::core::retrieval::RetrievalService;
use crate::infra::python_bridge::ParsedDocument;
use crate::models::contracts::{ConversationStub, ConversationTurnView, MemoryFactView, RetrievalHit};
use crate::models::errors::AlivError;

const MIGRATIONS: [&str; 8] = [
    include_str!("../../../db/migrations/0001_init.sql"),
    include_str!("../../../db/migrations/0002_embeddings_vec.sql"),
    include_str!("../../../db/migrations/0003_actions_audit.sql"),
    include_str!("../../../db/migrations/0004_benchmark_runs.sql"),
    include_str!("../../../db/migrations/0005_ingestion_runs.sql"),
    include_str!("../../../db/migrations/0006_parse_results.sql"),
    include_str!("../../../db/migrations/0007_memory_context.sql"),
    include_str!("../../../db/migrations/0008_embeddings_vec_index.sql"),
];

#[derive(Debug, Clone)]
pub struct IngestionRunSummary {
    pub run_id: String,
    pub discovered_files: usize,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub parsed_files: usize,
    pub chunk_count: usize,
    pub discovered: Vec<IngestionFileRecord>,
}

#[derive(Debug, Clone)]
pub struct IngestionTelemetrySnapshot {
    pub run_count: usize,
    pub discovered_total: usize,
    pub indexed_total: usize,
    pub skipped_total: usize,
    pub parsed_total: usize,
    pub failed_total: usize,
    pub file_count: usize,
    pub parsed_file_count: usize,
    pub chunk_count: usize,
    pub embedding_count: usize,
    pub avg_parser_confidence: f64,
    pub last_run_id: Option<String>,
    pub last_workspace_path: Option<String>,
    pub last_mode: Option<String>,
    pub last_created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IngestionFileRecord {
    pub id: String,
    pub source_path: String,
    pub file_type: String,
    pub should_parse: bool,
}

#[derive(Debug, Clone)]
struct DiscoveredFileRecord {
    id: String,
    workspace_path: String,
    source_path: String,
    file_hash: String,
    file_type: String,
    modified_at: String,
    inferred_doc_type: String,
    created_at: String,
}

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    applied_migrations: usize,
    connection: Mutex<Connection>,
}

impl Database {
    pub fn connect_default() -> Result<Self, AlivError> {
        let base_dir = std::env::current_dir()
            .map_err(|err| AlivError::Database(err.to_string()))?
            .join(".aliv");
        fs::create_dir_all(&base_dir).map_err(|err| AlivError::Database(err.to_string()))?;
        let database_path = base_dir.join("aliv.sqlite3");
        Self::connect(database_path)
    }

    pub fn connect(path: impl AsRef<Path>) -> Result<Self, AlivError> {
        let path = path.as_ref().to_path_buf();

        // Register sqlite-vec for all subsequent SQLite connections in this process.
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_vec_init as *const (),
            )));
        }

        let connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|err| AlivError::Database(err.to_string()))?;

        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let applied_migrations = Self::apply_migrations(&connection)?;

        Ok(Self {
            path,
            applied_migrations,
            connection: Mutex::new(connection),
        })
    }

    pub fn database_path(&self) -> String {
        self.path.display().to_string()
    }

    pub fn applied_migrations(&self) -> usize {
        self.applied_migrations
    }

    pub fn health_check(&self) -> Result<String, AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let workspace_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspace_folders", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(format!(
            "path={}, journal_mode={}, foreign_keys={}, migrations={}, workspace_count={}",
            self.database_path(),
            journal_mode,
            foreign_keys,
            self.applied_migrations,
            workspace_count,
        ))
    }

    pub fn register_workspace_folder(&self, path: &str) -> Result<bool, AlivError> {
        let normalized_path = path.trim();
        let created_at = current_timestamp_string()?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let changed = connection
            .execute(
                "INSERT OR IGNORE INTO workspace_folders (path, created_at) VALUES (?1, ?2)",
                params![normalized_path, created_at],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(changed > 0)
    }

    pub fn workspace_folder_exists(&self, path: &str) -> Result<bool, AlivError> {
        let normalized_path = path.trim();
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let exists = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM workspace_folders WHERE path = ?1)",
                [normalized_path],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(exists == 1)
    }

    pub fn run_ingestion_scan(
        &self,
        workspace_path: &str,
        incremental: bool,
    ) -> Result<IngestionRunSummary, AlivError> {
        let normalized_workspace = workspace_path.trim();
        if normalized_workspace.is_empty() {
            return Err(AlivError::Validation("workspace path cannot be empty".into()));
        }

        let workspace = Path::new(normalized_workspace);
        if !workspace.exists() {
            return Err(AlivError::Validation(format!(
                "workspace path '{}' does not exist",
                normalized_workspace
            )));
        }

        let created_at = current_timestamp_string()?;
        let run_id = stable_hex(&format!(
            "ingestion-run:{}:{}:{}",
            normalized_workspace,
            created_at,
            if incremental { "incremental" } else { "full" }
        ));
        let discovered_files = discover_files(normalized_workspace, &created_at)?;
        let discovered_count = discovered_files.len();

        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;
        let existing_hashes = load_existing_file_hashes(&connection, normalized_workspace)?;

        let mut ingestion_files = Vec::with_capacity(discovered_files.len());
        let mut skipped_files = 0_usize;
        let mut indexed_files = 0_usize;

        for file in &discovered_files {
            let previous_hash = existing_hashes.get(&file.source_path);
            let should_parse = !incremental || previous_hash != Some(&file.file_hash);
            if should_parse {
                indexed_files += 1;
            } else {
                skipped_files += 1;
            }
            ingestion_files.push(IngestionFileRecord {
                id: file.id.clone(),
                source_path: file.source_path.clone(),
                file_type: file.file_type.clone(),
                should_parse,
            });
        }

        let transaction = connection
            .transaction()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let discovered_paths: HashSet<String> = discovered_files
            .iter()
            .map(|file| file.source_path.clone())
            .collect();

        prune_missing_workspace_files(&transaction, normalized_workspace, &discovered_paths)?;

        transaction
            .execute(
                "INSERT INTO ingestion_runs (id, workspace_path, mode, discovered_count, indexed_count, skipped_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    run_id,
                    normalized_workspace,
                    if incremental { "incremental" } else { "full" },
                    discovered_count as i64,
                    indexed_files as i64,
                    skipped_files as i64,
                    created_at,
                ],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        for file in &discovered_files {
            transaction
                .execute(
                    "INSERT INTO files (id, workspace_path, source_path, file_hash, file_type, modified_at, inferred_doc_type, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(id) DO UPDATE SET
                       workspace_path = excluded.workspace_path,
                       source_path = excluded.source_path,
                       file_hash = excluded.file_hash,
                       file_type = excluded.file_type,
                       modified_at = excluded.modified_at,
                       inferred_doc_type = excluded.inferred_doc_type",
                    params![
                        file.id,
                        file.workspace_path,
                        file.source_path,
                        file.file_hash,
                        file.file_type,
                        file.modified_at,
                        file.inferred_doc_type,
                        file.created_at,
                    ],
                )
                .map_err(|err| AlivError::Database(err.to_string()))?;
        }

        transaction
            .commit()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(IngestionRunSummary {
            run_id,
            discovered_files: discovered_count,
            indexed_files,
            skipped_files,
            parsed_files: 0,
            chunk_count: 0,
            discovered: ingestion_files,
        })
    }

    pub fn finalize_ingestion_run(
        &self,
        run_id: &str,
        parsed_count: usize,
        chunk_count: usize,
        embedding_count: usize,
        failed_count: usize,
    ) -> Result<(), AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        connection
            .execute(
                "UPDATE ingestion_runs
                 SET parsed_count = ?2,
                     chunk_count = ?3,
                     embedding_count = ?4,
                     failed_count = ?5
                 WHERE id = ?1",
                params![
                    run_id,
                    parsed_count as i64,
                    chunk_count as i64,
                    embedding_count as i64,
                    failed_count as i64,
                ],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(())
    }

    pub fn replace_chunks_for_file(
        &self,
        file_id: &str,
        parsed: &ParsedDocument,
    ) -> Result<Vec<(String, String)>, AlivError> {
        let created_at = current_timestamp_string()?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;
        let transaction = connection
            .transaction()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        // Remove stale vec0 entries first (requires rowid from embeddings table).
        transaction
            .execute(
                "DELETE FROM embeddings_vec \
                 WHERE rowid IN (\
                   SELECT vec_rowid FROM embeddings \
                   WHERE chunk_id IN (SELECT id FROM chunks WHERE file_id = ?1) \
                   AND vec_rowid IS NOT NULL\
                 )",
                [file_id],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        // Remove embeddings before chunks to satisfy the FK constraint.
        transaction
            .execute(
                "DELETE FROM embeddings WHERE chunk_id IN (SELECT id FROM chunks WHERE file_id = ?1)",
                [file_id],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        transaction
            .execute("DELETE FROM chunks WHERE file_id = ?1", [file_id])
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let warnings_json = serde_json::to_string(&parsed.warnings)
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let summary_json = serde_json::to_string(&json!({
            "page_count": parsed.pages.len(),
            "page_map_count": parsed.page_map.len(),
            "normalized_text_length": parsed.normalized_text.len(),
        }))
        .map_err(|err| AlivError::Database(err.to_string()))?;

        transaction
            .execute(
                "INSERT INTO parse_results (file_id, parser_name, parser_confidence, warning_count, warnings_json, summary_json, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(file_id) DO UPDATE SET
                   parser_name = excluded.parser_name,
                   parser_confidence = excluded.parser_confidence,
                   warning_count = excluded.warning_count,
                   warnings_json = excluded.warnings_json,
                   summary_json = excluded.summary_json,
                   updated_at = excluded.updated_at",
                params![
                    file_id,
                    parsed.parser_name,
                    parsed.parser_confidence,
                    parsed.warnings.len() as i64,
                    warnings_json,
                    summary_json,
                    created_at,
                ],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut chunk_data = Vec::new();
        for page in &parsed.pages {
            if !page.has_text || page.text.trim().is_empty() {
                continue;
            }

            let content_hash = stable_hex(&page.text);
            let chunk_id = stable_hex(&format!(
                "chunk:{}:{}:{}",
                file_id, page.page_number, content_hash
            ));
            let page_ref = format!("page:{}", page.page_number);

            transaction
                .execute(
                    "INSERT INTO chunks (id, file_id, chunk_index, page_ref, content, content_hash, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        chunk_id,
                        file_id,
                        page.page_number as i64,
                        page_ref,
                        page.text,
                        content_hash,
                        created_at,
                    ],
                )
                .map_err(|err| AlivError::Database(err.to_string()))?;
            
            chunk_data.push((chunk_id, page.text.clone()));
        }

        transaction
            .commit()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(chunk_data)
    }

    pub fn search_chunks(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievalHit>, AlivError> {
        let service = RetrievalService;
        let terms = service.search_terms(query);
        if terms.is_empty() {
            return Err(AlivError::Validation(
                "retrieval query cannot be empty".into(),
            ));
        }

        let candidate_limit = std::cmp::max(top_k, 1).saturating_mul(8).min(200);
        let mut sql = String::from(
            "SELECT c.file_id, f.source_path, c.chunk_index, c.page_ref, c.content, pr.parser_confidence, pr.warning_count \
             FROM chunks c \
             INNER JOIN files f ON f.id = c.file_id \
             LEFT JOIN parse_results pr ON pr.file_id = c.file_id \
             WHERE ",
        );

        let patterns: Vec<String> = terms.iter().map(|term| format!("%{}%", term)).collect();
        for index in 0..patterns.len() {
            if index > 0 {
                sql.push_str(" OR ");
            }
            sql.push_str(&format!("lower(c.content) LIKE ?{}", index + 1));
        }
        sql.push_str(&format!(" LIMIT {}", candidate_limit));

        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;
        let mut statement = connection
            .prepare(&sql)
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let rows = statement
            .query_map(params_from_iter(patterns.iter()), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<f64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                ))
            })
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut hits = Vec::new();
        for row in rows {
            let (file_id, source_path, chunk_index, page_ref, content, parser_confidence, warning_count) =
                row.map_err(|err| AlivError::Database(err.to_string()))?;
            let score = service.score_content(&content, &terms);
            if score == 0 {
                continue;
            }

            hits.push(RetrievalHit {
                file_id,
                source_path,
                chunk_index: chunk_index.max(0) as usize,
                page_ref,
                score,
                content_preview: service.build_preview(&content, &terms),
                parser_confidence,
                warning_count: warning_count.unwrap_or(0).max(0) as usize,
            });
        }

        hits.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.source_path.cmp(&right.source_path))
                .then_with(|| left.chunk_index.cmp(&right.chunk_index))
        });
        hits.truncate(std::cmp::max(top_k, 1));

        Ok(hits)
    }

    pub fn store_embeddings(
        &self,
        chunk_vectors: &[(String, Vec<f32>)],
        embedding_model: &str,
    ) -> Result<usize, AlivError> {
        if chunk_vectors.is_empty() {
            return Ok(0);
        }

        let created_at = current_timestamp_string()?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;
        let transaction = connection
            .transaction()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut stored_count = 0;
        let embedding_dim = chunk_vectors.get(0).map(|(_, vec)| vec.len()).unwrap_or(0);

        for (chunk_id, vector) in chunk_vectors {
            let vector_json =
                serde_json::to_string(vector).map_err(|err| AlivError::Database(err.to_string()))?;
            let embedding_id = stable_hex(&format!("embedding:{}:{}", chunk_id, embedding_model));

            let rows_affected = transaction
                .execute(
                    "INSERT INTO embeddings (id, chunk_id, embedding_model, vector_dim, vector_json, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                     ON CONFLICT(chunk_id) DO UPDATE SET \
                       embedding_model = excluded.embedding_model, \
                       vector_json = excluded.vector_json, \
                       created_at = excluded.created_at",
                    params![
                        embedding_id,
                        chunk_id,
                        embedding_model,
                        embedding_dim as i64,
                        vector_json,
                        created_at,
                    ],
                )
                .map_err(|err| AlivError::Database(err.to_string()))?;

            // Insert into vec0 virtual table (ANN index) and record rowid for future cleanup.
            let vec_blob = f32_slice_to_blob(vector);
            transaction
                .execute(
                    "INSERT INTO embeddings_vec(embedding, chunk_id) VALUES (?1, ?2)",
                    params![vec_blob, chunk_id],
                )
                .map_err(|err| AlivError::Database(err.to_string()))?;
            let vec_rowid = transaction.last_insert_rowid();
            transaction
                .execute(
                    "UPDATE embeddings SET vec_rowid = ?1 WHERE chunk_id = ?2",
                    params![vec_rowid, chunk_id],
                )
                .map_err(|err| AlivError::Database(err.to_string()))?;

            if rows_affected > 0 {
                stored_count += 1;
            }
        }

        transaction
            .commit()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(stored_count)
    }

    pub fn ingestion_telemetry_snapshot(&self) -> Result<IngestionTelemetrySnapshot, AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let (run_count, discovered_total, indexed_total, skipped_total, parsed_total, failed_total): (i64, i64, i64, i64, i64, i64) = connection
            .query_row(
            "SELECT COUNT(*),
                COALESCE(SUM(discovered_count), 0),
                COALESCE(SUM(indexed_count), 0),
                COALESCE(SUM(skipped_count), 0),
                COALESCE(SUM(parsed_count), 0),
                COALESCE(SUM(failed_count), 0)
             FROM ingestion_runs",
                [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let file_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let parsed_file_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM parse_results", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let chunk_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let embedding_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))
            .map_err(|err| AlivError::Database(err.to_string()))?;
        let avg_parser_confidence: f64 = connection
            .query_row(
                "SELECT COALESCE(AVG(parser_confidence), 0.0) FROM parse_results",
                [],
                |row| row.get(0),
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let last_run: Option<(String, String, String, String)> = connection
            .query_row(
                "SELECT id, workspace_path, mode, created_at FROM ingestion_runs ORDER BY created_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(IngestionTelemetrySnapshot {
            run_count: run_count.max(0) as usize,
            discovered_total: discovered_total.max(0) as usize,
            indexed_total: indexed_total.max(0) as usize,
            skipped_total: skipped_total.max(0) as usize,
            parsed_total: parsed_total.max(0) as usize,
            failed_total: failed_total.max(0) as usize,
            file_count: file_count.max(0) as usize,
            parsed_file_count: parsed_file_count.max(0) as usize,
            chunk_count: chunk_count.max(0) as usize,
            embedding_count: embedding_count.max(0) as usize,
            avg_parser_confidence,
            last_run_id: last_run.as_ref().map(|value| value.0.clone()),
            last_workspace_path: last_run.as_ref().map(|value| value.1.clone()),
            last_mode: last_run.as_ref().map(|value| value.2.clone()),
            last_created_at: last_run.as_ref().map(|value| value.3.clone()),
        })
    }

    /// Search using sqlite-vec ANN (O(log N) via vec0 virtual table).
    /// Query vector must be 384-dimensional (all-MiniLM-L6-v2 output).
    pub fn search_embeddings(
        &self,
        query_vector: &[f32],
        top_k: usize,
    ) -> Result<Vec<RetrievalHit>, AlivError> {
        if query_vector.is_empty() {
            return Err(AlivError::Validation("query vector cannot be empty".into()));
        }

        let k = std::cmp::max(top_k, 1) as i64;
        let query_blob = f32_slice_to_blob(query_vector);
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        // vec0 MATCH query: returns top-k nearest neighbours via ANN index.
        // distance is cosine distance in [0, 2] (0 = identical).
        let mut statement = connection
            .prepare(
                "SELECT v.chunk_id, v.distance, \
                        c.file_id, f.source_path, c.chunk_index, c.page_ref, c.content, \
                        pr.parser_confidence, pr.warning_count \
                 FROM embeddings_vec v \
                 INNER JOIN chunks c ON c.id = v.chunk_id \
                 INNER JOIN files f ON f.id = c.file_id \
                 LEFT JOIN parse_results pr ON pr.file_id = c.file_id \
                 WHERE v.embedding MATCH ?1 AND k = ?2 \
                 ORDER BY v.distance ASC",
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let rows = statement
            .query_map(params![query_blob, k], |row| {
                Ok((
                    row.get::<_, String>(0)?,    // chunk_id
                    row.get::<_, f64>(1)?,        // distance
                    row.get::<_, String>(2)?,    // file_id
                    row.get::<_, String>(3)?,    // source_path
                    row.get::<_, i64>(4)?,        // chunk_index
                    row.get::<_, Option<String>>(5)?, // page_ref
                    row.get::<_, String>(6)?,    // content
                    row.get::<_, Option<f64>>(7)?,    // parser_confidence
                    row.get::<_, Option<i64>>(8)?,    // warning_count
                ))
            })
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut hits = Vec::new();
        for row in rows {
            let (_chunk_id, distance, file_id, source_path, chunk_index, page_ref, content,
                 parser_confidence, warning_count) =
                row.map_err(|err| AlivError::Database(err.to_string()))?;

            // Convert cosine distance [0,2] → similarity score [0,1000] (higher = more similar).
            let score = ((2.0 - distance) * 500.0) as i64;
            hits.push(RetrievalHit {
                file_id,
                source_path,
                chunk_index: chunk_index.max(0) as usize,
                page_ref,
                score,
                content_preview: content[..content.len().min(140)].to_owned(),
                parser_confidence,
                warning_count: warning_count.unwrap_or(0).max(0) as usize,
            });
        }

        Ok(hits)
    }

    pub fn create_conversation(
        &self,
        id: &str,
        workspace_path: &str,
        title: &str,
    ) -> Result<(), AlivError> {
        let now = current_rfc3339_timestamp();
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        connection
            .execute(
                "INSERT INTO conversations (id, workspace_path, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, workspace_path.trim(), title.trim(), now, now],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(())
    }

    pub fn save_conversation_turn(
        &self,
        id: &str,
        conversation_id: &str,
        role: &str,
        content: &str,
        citations_json: Option<&str>,
        token_count: Option<i64>,
    ) -> Result<(), AlivError> {
        let normalized_role = role.trim().to_ascii_lowercase();
        if normalized_role != "user" && normalized_role != "assistant" {
            return Err(AlivError::Validation(
                "conversation role must be 'user' or 'assistant'".into(),
            ));
        }

        let now = current_rfc3339_timestamp();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;
        let tx = connection
            .transaction()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        tx.execute(
            "INSERT INTO conversation_turns (id, conversation_id, role, content, citations_json, token_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                conversation_id,
                normalized_role,
                content,
                citations_json,
                token_count,
                now,
            ],
        )
        .map_err(|err| AlivError::Database(err.to_string()))?;

        tx.execute(
            "UPDATE conversations SET updated_at = ?2 WHERE id = ?1",
            params![conversation_id, now],
        )
        .map_err(|err| AlivError::Database(err.to_string()))?;

        tx.commit()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(())
    }

    pub fn list_conversations(
        &self,
        workspace_path: &str,
        limit: usize,
    ) -> Result<Vec<ConversationStub>, AlivError> {
        let max_limit = std::cmp::max(limit, 1).min(200) as i64;
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let mut stmt = connection
            .prepare(
                "SELECT c.id, c.title, c.updated_at, COUNT(t.id) AS turn_count
                 FROM conversations c
                 LEFT JOIN conversation_turns t ON t.conversation_id = c.id
                 WHERE c.workspace_path = ?1
                 GROUP BY c.id, c.title, c.updated_at
                 ORDER BY c.updated_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let rows = stmt
            .query_map(params![workspace_path.trim(), max_limit], |row| {
                Ok(ConversationStub {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    updated_at: row.get(2)?,
                    turn_count: row.get::<_, i64>(3)?.max(0) as usize,
                })
            })
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| AlivError::Database(err.to_string()))?);
        }

        Ok(out)
    }

    pub fn get_conversation_turns(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<ConversationTurnView>, AlivError> {
        let max_limit = std::cmp::max(limit, 1).min(500) as i64;
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let mut stmt = connection
            .prepare(
                "SELECT id, role, content, citations_json, token_count, created_at
                 FROM conversation_turns
                 WHERE conversation_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let rows = stmt
            .query_map(params![conversation_id, max_limit], |row| {
                Ok(ConversationTurnView {
                    id: row.get(0)?,
                    role: row.get(1)?,
                    content: row.get(2)?,
                    citations_json: row.get(3)?,
                    token_count: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| AlivError::Database(err.to_string()))?);
        }

        Ok(out)
    }

    pub fn save_memory_fact(
        &self,
        id: &str,
        workspace_path: &str,
        content: &str,
        citation_json: Option<&str>,
        scope: &str,
        expires_at: Option<&str>,
    ) -> Result<(), AlivError> {
        let normalized_scope = scope.trim().to_ascii_lowercase();
        if normalized_scope != "session"
            && normalized_scope != "week"
            && normalized_scope != "permanent"
        {
            return Err(AlivError::Validation(
                "memory scope must be 'session', 'week', or 'permanent'".into(),
            ));
        }

        let now = current_rfc3339_timestamp();
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        connection
            .execute(
                "INSERT INTO memory_facts (id, workspace_path, content, citation_json, scope, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    workspace_path.trim(),
                    content,
                    citation_json,
                    normalized_scope,
                    now,
                    expires_at,
                ],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(())
    }

    pub fn list_memory_facts(
        &self,
        workspace_path: &str,
    ) -> Result<Vec<MemoryFactView>, AlivError> {
        let now = current_rfc3339_timestamp();
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let mut stmt = connection
            .prepare(
                "SELECT id, content, citation_json, scope, created_at, expires_at
                 FROM memory_facts
                 WHERE workspace_path = ?1
                   AND (expires_at IS NULL OR expires_at > ?2)
                 ORDER BY created_at DESC",
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let rows = stmt
            .query_map(params![workspace_path.trim(), now], |row| {
                Ok(MemoryFactView {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    citation_json: row.get(2)?,
                    scope: row.get(3)?,
                    created_at: row.get(4)?,
                    expires_at: row.get(5)?,
                })
            })
            .map_err(|err| AlivError::Database(err.to_string()))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| AlivError::Database(err.to_string()))?);
        }

        Ok(out)
    }

    pub fn delete_memory_fact(&self, fact_id: &str) -> Result<(), AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        connection
            .execute("DELETE FROM memory_facts WHERE id = ?1", [fact_id])
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_document_summary(
        &self,
        id: &str,
        file_id: &str,
        file_hash: &str,
        granularity: &str,
        section_index: Option<i64>,
        content: &str,
        token_count: i64,
        model_version: &str,
    ) -> Result<(), AlivError> {
        let normalized_granularity = granularity.trim().to_ascii_lowercase();
        if normalized_granularity != "document" && normalized_granularity != "section" {
            return Err(AlivError::Validation(
                "summary granularity must be 'document' or 'section'".into(),
            ));
        }

        let now = current_rfc3339_timestamp();
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        connection
            .execute(
                "INSERT INTO document_summaries (id, file_id, file_hash, granularity, section_index, content, token_count, model_version, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    file_id,
                    file_hash,
                    normalized_granularity,
                    section_index,
                    content,
                    token_count,
                    model_version,
                    now,
                ],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(())
    }

    pub fn get_document_summary(
        &self,
        file_id: &str,
        granularity: &str,
    ) -> Result<Option<String>, AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let summary = connection
            .query_row(
                "SELECT content
                 FROM document_summaries
                 WHERE file_id = ?1 AND granularity = ?2
                 ORDER BY created_at DESC
                 LIMIT 1",
                params![file_id, granularity.trim().to_ascii_lowercase()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(summary)
    }

    pub fn invalidate_document_summaries(
        &self,
        file_id: &str,
        new_file_hash: &str,
    ) -> Result<usize, AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let deleted = connection
            .execute(
                "DELETE FROM document_summaries WHERE file_id = ?1 AND file_hash != ?2",
                params![file_id, new_file_hash],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(deleted)
    }

    pub fn get_file_hash(&self, file_id: &str) -> Result<Option<String>, AlivError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))?;

        let value = connection
            .query_row(
                "SELECT file_hash FROM files WHERE id = ?1",
                [file_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| AlivError::Database(err.to_string()))?;

        Ok(value)
    }

    // -- Action Recommendations ------------------------------------------------

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
        self.conn()?
            .execute(
                "INSERT OR REPLACE INTO action_recommendations
         (id, file_id, recommendation_type, source_path, proposed_path, confidence, evidence_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    file_id,
                    recommendation_type,
                    source_path,
                    proposed_path,
                    confidence,
                    evidence_json,
                    now
                ],
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn list_recommendations(
        &self,
        workspace_path: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String, String, String, f64, String)>, AlivError> {
        // Returns: (id, file_id, rec_type, source_path, proposed_path, confidence, evidence_json)
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT ar.id, ar.file_id, ar.recommendation_type, ar.source_path, ar.proposed_path,
                ar.confidence, ar.evidence_json
         FROM action_recommendations ar
         JOIN files f ON f.id = ar.file_id
         WHERE f.workspace_path = ?1
         ORDER BY ar.created_at DESC
         LIMIT ?2",
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![workspace_path, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, f64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| AlivError::Database(e.to_string()))?;

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
        self.conn()?
            .execute(
                "INSERT INTO action_executions (id, recommendation_id, batch_id, status, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, recommendation_id, batch_id, status, now],
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn save_undo_record(
        &self,
        id: &str,
        execution_id: &str,
        inverse_operation_json: &str,
    ) -> Result<(), AlivError> {
        let now = Self::now_ts();
        self.conn()?
            .execute(
                "INSERT INTO undo_records (id, execution_id, inverse_operation_json, created_at)
         VALUES (?1, ?2, ?3, ?4)",
                params![id, execution_id, inverse_operation_json, now],
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_undo_records_for_batch(
        &self,
        batch_id: &str,
    ) -> Result<Vec<(String, String, String)>, AlivError> {
        // Returns: (undo_record_id, execution_id, inverse_operation_json)
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT ur.id, ur.execution_id, ur.inverse_operation_json
         FROM undo_records ur
         JOIN action_executions ae ON ae.id = ur.execution_id
         WHERE ae.batch_id = ?1",
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![batch_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| AlivError::Database(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AlivError::Database(e.to_string()))
    }

    pub fn list_audit_log(
        &self,
        limit: usize,
    ) -> Result<Vec<(String, String, String, String, String, String, String, String)>, AlivError>
    {
        // Returns: (execution_id, recommendation_id, batch_id, source_path, proposed_path, rec_type, status, executed_at)
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT ae.id, ae.recommendation_id, ae.batch_id,
                ar.source_path, ar.proposed_path, ar.recommendation_type,
                ae.status, ae.executed_at
         FROM action_executions ae
         JOIN action_recommendations ar ON ar.id = ae.recommendation_id
         ORDER BY ae.executed_at DESC
         LIMIT ?1",
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
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
            })
            .map_err(|e| AlivError::Database(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AlivError::Database(e.to_string()))
    }

    pub fn list_files_needing_rename(
        &self,
        workspace_path: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String, Option<f64>)>, AlivError> {
        // Returns: (file_id, source_path, inferred_doc_type, parser_confidence)
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.source_path, f.inferred_doc_type, pr.parser_confidence
         FROM files f
         LEFT JOIN parse_results pr ON pr.file_id = f.id
         WHERE f.workspace_path = ?1
         AND f.inferred_doc_type IS NOT NULL
         ORDER BY pr.parser_confidence DESC
         LIMIT ?2",
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![workspace_path, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<f64>>(3)?,
                ))
            })
            .map_err(|e| AlivError::Database(e.to_string()))?;

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
        self.conn()?
            .execute(
                "UPDATE action_executions SET status = ?1 WHERE id = ?2",
                params![status, execution_id],
            )
            .map_err(|e| AlivError::Database(e.to_string()))?;
        Ok(())
    }

    fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AlivError> {
        self.connection
            .lock()
            .map_err(|_| AlivError::Database("database mutex poisoned".into()))
    }

    fn now_ts() -> String {
        current_timestamp_string().unwrap_or_else(|_| "0".into())
    }

    fn apply_migrations(connection: &Connection) -> Result<usize, AlivError> {
        for migration in MIGRATIONS {
            connection
                .execute_batch(migration)
                .map_err(|err| AlivError::Database(err.to_string()))?;
        }

        Ok(MIGRATIONS.len())
    }
}

fn current_timestamp_string() -> Result<String, AlivError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AlivError::Database(err.to_string()))?;
    Ok(duration.as_secs().to_string())
}

fn current_rfc3339_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn discover_files(
    workspace_path: &str,
    created_at: &str,
) -> Result<Vec<DiscoveredFileRecord>, AlivError> {
    let mut files = Vec::new();

    for entry in WalkDir::new(workspace_path) {
        let entry = entry.map_err(|err| AlivError::Database(err.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let source_path = entry.path().to_string_lossy().to_string();
        let file_hash = content_hash(entry.path())?;
        let file_type = entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| "unknown".into());
        let modified_at = file_modified_timestamp(entry.path())?;
        let file_id = stable_hex(&format!("file:{}", source_path));

        files.push(DiscoveredFileRecord {
            id: file_id,
            workspace_path: workspace_path.to_string(),
            source_path,
            file_hash,
            file_type: file_type.clone(),
            modified_at,
            inferred_doc_type: file_type,
            created_at: created_at.to_string(),
        });
    }

    Ok(files)
}

fn content_hash(path: &Path) -> Result<String, AlivError> {
    let mut file = fs::File::open(path).map_err(|err| AlivError::Database(err.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| AlivError::Database(err.to_string()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn file_modified_timestamp(path: &Path) -> Result<String, AlivError> {
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|err| AlivError::Database(err.to_string()))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AlivError::Database(err.to_string()))?;
    Ok(duration.as_secs().to_string())
}

fn stable_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn load_existing_file_hashes(
    connection: &Connection,
    workspace_path: &str,
) -> Result<std::collections::HashMap<String, String>, AlivError> {
    let mut statement = connection
        .prepare("SELECT source_path, file_hash FROM files WHERE workspace_path = ?1")
        .map_err(|err| AlivError::Database(err.to_string()))?;
    let rows = statement
        .query_map([workspace_path], |row| {
            let source_path: String = row.get(0)?;
            let file_hash: String = row.get(1)?;
            Ok((source_path, file_hash))
        })
        .map_err(|err| AlivError::Database(err.to_string()))?;

    let mut hashes = std::collections::HashMap::new();
    for row in rows {
        let (source_path, file_hash) = row.map_err(|err| AlivError::Database(err.to_string()))?;
        hashes.insert(source_path, file_hash);
    }

    Ok(hashes)
}

fn prune_missing_workspace_files(
    transaction: &rusqlite::Transaction<'_>,
    workspace_path: &str,
    discovered_paths: &HashSet<String>,
) -> Result<(), AlivError> {
    let mut statement = transaction
        .prepare("SELECT id, source_path FROM files WHERE workspace_path = ?1")
        .map_err(|err| AlivError::Database(err.to_string()))?;
    let rows = statement
        .query_map([workspace_path], |row| {
            let file_id: String = row.get(0)?;
            let source_path: String = row.get(1)?;
            Ok((file_id, source_path))
        })
        .map_err(|err| AlivError::Database(err.to_string()))?;

    let mut stale_file_ids = Vec::new();
    for row in rows {
        let (file_id, source_path) = row.map_err(|err| AlivError::Database(err.to_string()))?;
        if !discovered_paths.contains(&source_path) {
            stale_file_ids.push(file_id);
        }
    }

    for file_id in stale_file_ids {
        transaction
            .execute(
                "DELETE FROM embeddings WHERE chunk_id IN (SELECT id FROM chunks WHERE file_id = ?1)",
                [&file_id],
            )
            .map_err(|err| AlivError::Database(err.to_string()))?;
        transaction
            .execute("DELETE FROM chunks WHERE file_id = ?1", [&file_id])
            .map_err(|err| AlivError::Database(err.to_string()))?;
        transaction
            .execute("DELETE FROM parse_results WHERE file_id = ?1", [&file_id])
            .map_err(|err| AlivError::Database(err.to_string()))?;
        transaction
            .execute("DELETE FROM files WHERE id = ?1", [&file_id])
            .map_err(|err| AlivError::Database(err.to_string()))?;
    }

    Ok(())
}

/// Encode a float vector as a little-endian binary blob for sqlite-vec storage.
fn f32_slice_to_blob(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for &f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut dot_product = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    for (av, bv) in a.iter().zip(b.iter()) {
        dot_product += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}
