CREATE TABLE IF NOT EXISTS embeddings (
  id TEXT PRIMARY KEY,
  chunk_id TEXT NOT NULL UNIQUE,
  embedding_model TEXT NOT NULL,
  vector_dim INTEGER NOT NULL,
  vector_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (chunk_id) REFERENCES chunks(id)
);

CREATE INDEX IF NOT EXISTS idx_embeddings_chunk_id ON embeddings(chunk_id);
CREATE INDEX IF NOT EXISTS idx_embeddings_model ON embeddings(embedding_model);
