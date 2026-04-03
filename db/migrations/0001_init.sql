PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS workspace_folders (
  path TEXT PRIMARY KEY,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS files (
  id TEXT PRIMARY KEY,
  workspace_path TEXT NOT NULL,
  source_path TEXT NOT NULL,
  file_hash TEXT NOT NULL,
  file_type TEXT NOT NULL,
  modified_at TEXT NOT NULL,
  inferred_doc_type TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chunks (
  id TEXT PRIMARY KEY,
  file_id TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  page_ref TEXT,
  content TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE INDEX IF NOT EXISTS idx_workspace_folders_created_at ON workspace_folders(created_at);
CREATE INDEX IF NOT EXISTS idx_files_hash ON files(file_hash);
CREATE INDEX IF NOT EXISTS idx_chunks_file_id ON chunks(file_id);
