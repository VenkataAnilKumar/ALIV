CREATE TABLE IF NOT EXISTS benchmark_runs (
  id TEXT PRIMARY KEY,
  corpus_version TEXT NOT NULL,
  labels_version TEXT NOT NULL,
  model_versions_json TEXT NOT NULL,
  machine_profile_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS benchmark_metrics (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  metric_name TEXT NOT NULL,
  metric_value REAL NOT NULL,
  threshold_value REAL,
  passed INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES benchmark_runs(id)
);
