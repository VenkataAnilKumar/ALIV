CREATE TABLE IF NOT EXISTS ingestion_runs (
  id TEXT PRIMARY KEY,
  workspace_path TEXT NOT NULL,
  mode TEXT NOT NULL,
  discovered_count INTEGER NOT NULL,
  indexed_count INTEGER NOT NULL,
  skipped_count INTEGER NOT NULL DEFAULT 0,
  parsed_count INTEGER NOT NULL DEFAULT 0,
  chunk_count INTEGER NOT NULL DEFAULT 0,
  embedding_count INTEGER NOT NULL DEFAULT 0,
  failed_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ingestion_runs_workspace_path ON ingestion_runs(workspace_path);