SELECT 'files' AS table_name, COUNT(*) AS row_count FROM files
UNION ALL
SELECT 'chunks' AS table_name, COUNT(*) AS row_count FROM chunks
UNION ALL
SELECT 'embeddings' AS table_name, COUNT(*) AS row_count FROM embeddings
UNION ALL
SELECT 'action_recommendations' AS table_name, COUNT(*) AS row_count FROM action_recommendations
UNION ALL
SELECT 'action_executions' AS table_name, COUNT(*) AS row_count FROM action_executions
UNION ALL
SELECT 'undo_records' AS table_name, COUNT(*) AS row_count FROM undo_records
UNION ALL
SELECT 'benchmark_runs' AS table_name, COUNT(*) AS row_count FROM benchmark_runs
UNION ALL
SELECT 'benchmark_metrics' AS table_name, COUNT(*) AS row_count FROM benchmark_metrics;
