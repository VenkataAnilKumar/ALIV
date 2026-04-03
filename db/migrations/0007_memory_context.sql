-- Migration 0005: Memory and Long Context Tables
-- Adds conversation history, document summary cache, and user-pinned memory facts.

-- Conversation sessions (one per multi-turn thread per workspace)
CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  workspace_path TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT 'Untitled',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conversations_workspace
  ON conversations (workspace_path, updated_at DESC);

-- Individual message turns within a conversation
CREATE TABLE IF NOT EXISTS conversation_turns (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
  content TEXT NOT NULL,
  citations_json TEXT,         -- JSON array: [{file_id, chunk_id, page_ref, span}]
  token_count INTEGER,         -- estimated token count for budget management
  created_at TEXT NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_turns_conversation_time
  ON conversation_turns (conversation_id, created_at DESC);

-- Pre-computed document summaries at multiple granularities
-- Invalidated by file hash change (re-ingestion sets new summary row)
CREATE TABLE IF NOT EXISTS document_summaries (
  id TEXT PRIMARY KEY,
  file_id TEXT NOT NULL,
  file_hash TEXT NOT NULL,     -- SHA-256 of source file; stale if != files.content_hash
  granularity TEXT NOT NULL CHECK (granularity IN ('document', 'section')),
  section_index INTEGER,       -- NULL for document-level; 0-based index for section-level
  content TEXT NOT NULL,
  token_count INTEGER NOT NULL,
  model_version TEXT NOT NULL, -- Ollama model used to generate summary
  created_at TEXT NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_summaries_file_granularity
  ON document_summaries (file_id, granularity);

-- User-pinned memory facts with source citations and expiry scope
CREATE TABLE IF NOT EXISTS memory_facts (
  id TEXT PRIMARY KEY,
  workspace_path TEXT NOT NULL,
  content TEXT NOT NULL,
  source_file_id TEXT,
  source_chunk_id TEXT,
  citation_json TEXT,          -- {file_id, chunk_id, page_ref, excerpt}
  scope TEXT NOT NULL DEFAULT 'permanent' CHECK (scope IN ('session', 'week', 'permanent')),
  embedding_json TEXT,         -- 384-dim vector for semantic recall (nullable until embedded)
  created_at TEXT NOT NULL,
  expires_at TEXT,             -- NULL for permanent; ISO-8601 for session/week
  FOREIGN KEY (source_file_id) REFERENCES files(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_memory_facts_workspace
  ON memory_facts (workspace_path, expires_at);
