CREATE TABLE IF NOT EXISTS parse_results (
  file_id TEXT PRIMARY KEY,
  parser_name TEXT NOT NULL,
  parser_confidence REAL NOT NULL,
  warning_count INTEGER NOT NULL,
  warnings_json TEXT NOT NULL,
  summary_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (file_id) REFERENCES files(id)
);

CREATE INDEX IF NOT EXISTS idx_parse_results_confidence ON parse_results(parser_confidence);