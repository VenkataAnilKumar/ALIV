CREATE TABLE IF NOT EXISTS action_recommendations (
  id TEXT PRIMARY KEY,
  file_id TEXT NOT NULL,
  recommendation_type TEXT NOT NULL,
  source_path TEXT NOT NULL,
  proposed_path TEXT NOT NULL,
  confidence REAL NOT NULL,
  evidence_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE TABLE IF NOT EXISTS action_executions (
  id TEXT PRIMARY KEY,
  recommendation_id TEXT NOT NULL,
  batch_id TEXT NOT NULL,
  status TEXT NOT NULL,
  executed_at TEXT NOT NULL,
  FOREIGN KEY (recommendation_id) REFERENCES action_recommendations(id)
);

CREATE TABLE IF NOT EXISTS undo_records (
  id TEXT PRIMARY KEY,
  execution_id TEXT NOT NULL,
  inverse_operation_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (execution_id) REFERENCES action_executions(id)
);
