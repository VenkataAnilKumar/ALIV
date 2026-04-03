-- Migration 0008: sqlite-vec ANN index for fast embedding search.
-- Requires the sqlite-vec extension to be loaded before this migration runs.
-- Provides O(log N) approximate nearest-neighbour search via vec0 virtual table.

-- Track the integer rowid assigned by embeddings_vec so we can delete stale entries.
ALTER TABLE embeddings ADD COLUMN vec_rowid INTEGER;

-- vec0 virtual table: 384-dim float vectors with chunk_id as auxiliary column.
-- The +chunk_id auxiliary column is stored alongside the index and returned in MATCH queries.
CREATE VIRTUAL TABLE IF NOT EXISTS embeddings_vec USING vec0(
  embedding float[384],
  +chunk_id TEXT
);
