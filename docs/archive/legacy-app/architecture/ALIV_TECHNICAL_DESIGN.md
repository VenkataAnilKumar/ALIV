# ALIV Technical Design Specification

## 1. Purpose
Define implementation-level structure, interfaces, and execution contracts for PRD-001 to PRD-003.

## 2. Repository Structure (Target)
1. src-tauri for Rust core and command handlers.
2. src for React and TypeScript UI.
3. python for parsing, OCR, and extraction workers.
4. db for schema and migrations.
5. benchmarks for corpus manifests and run outputs.

## 3. Rust Module Design
1. core.workspace
- Folder registration, scan settings, ignore patterns.

2. core.ingestion
- Discovery orchestration and parser dispatch.
- Post-ingestion hook: triggers summarization_worker.py per file.

3. core.index
- Chunk storage, embedding storage, incremental update logic.

4. core.retrieval
- Hybrid query planner and ranker.
- Accepts optional conversation_id; delegates context assembly to core.memory.

5. core.memory (PRD-004)
- MemoryService: token estimator, title deriver, context assembler, prompt formatter.
- TokenBudget: configurable context window, output reserve, system prompt allocation.
- BudgetAllocation: splits available tokens across memory facts, history, chunks, summaries.
- AssembledContext: ordered ContextBlock list with overflow telemetry.
- Key methods: estimate_tokens(), assemble_context(), format_prompt().

6. core.actions
- Recommendation planner, preview queue, executor, rollback.

7. core.telemetry
- Metrics aggregator, run tracking, failure taxonomy.

8. infra.db
- SQLite access, migrations, transaction boundaries.
- Memory methods (Phase 2): create_conversation, save_conversation_turn, list_conversations,
  get_conversation_turns, save_memory_fact, list_memory_facts, delete_memory_fact,
  save_document_summary, get_document_summary, invalidate_document_summaries.

9. infra.python_bridge
- Process lifecycle, stdin and stdout protocol, timeout and retry.

## 4. Python Worker Contracts
### 4.1 Parse Worker
Input:
1. file path
2. file type
3. parser options

Output:
1. normalized text
2. page or section map
3. parser confidence
4. warnings and errors

### 4.2 OCR Worker
Input:
1. image or scanned PDF path
2. OCR profile

Output:
1. recognized text
2. confidence map
3. quality flags

### 4.3 Extraction Worker
Input:
1. selected chunks
2. extraction template

Output:
1. field-value map
2. field confidence map
3. citations by chunk id

## 5. IPC Protocol (Rust to Python)
Message format:
1. JSON lines over stdin and stdout.

Required fields:
1. request_id
2. worker_type
3. payload
4. timeout_ms
5. worker_version

Response fields:
1. request_id
2. status
3. result
4. error_code
5. error_detail

## 6. Database Design (Initial)
### 6.1 Core Tables
1. files
2. chunks
3. embeddings
4. queries
5. extraction_results
6. action_recommendations
7. action_executions
8. undo_records
9. benchmark_runs
10. benchmark_metrics

### 6.2 Data Constraints
1. Unique file hash per workspace scope.
2. Chunk ids unique per file version.
3. Action execution must reference recommendation id.
4. Undo record must reference execution id.

## 7. API and Command Surface (Tauri)
1. register_workspace_folder
2. run_ingestion
3. run_retrieval_query
4. run_extraction
5. generate_action_recommendations
6. get_action_preview_queue
7. evaluate_action_risk
8. approve_actions
9. execute_actions
10. undo_action_batch
11. get_action_audit_log
12. run_benchmark_suite
13. get_run_metrics

## 8. Benchmark Harness Design
1. Fixed corpus manifest and version id.
2. Gold Q and A benchmark file.
3. Field label benchmark file.
4. Run output folder with summary and raw traces.

Required outputs per run:
1. ingestion success rate
2. retrieval relevance top-5
3. extraction precision macro and per field
4. harmful action rate
5. query latency p50 and p95

## 9. Error Handling Design
1. Standardized error categories:
- parser_failure
- ocr_failure
- embedding_failure
- db_failure
- action_safety_block
- rollback_failure

2. Every error includes:
- category
- root-cause guess
- retryable flag
- evidence

## 10. Observability Design
1. Structured JSON logs in local log directory.
2. Run-level correlation id.
3. Metrics snapshots persisted to benchmark_metrics table.
4. UI diagnostics panel reads run summaries.

## 11. Security and Safety Controls
1. File actions default to preview mode.
2. Prevent overwrite unless explicit safe strategy chosen.
3. Enforce path allowlist from registered workspace folders.
4. Block operations outside allowlist.

## 12. Technical Risks and Mitigations
1. Inconsistent parser output across file types.
- Mitigation: adapter layer with normalized intermediate schema.

2. OCR noise lowers extraction precision.
- Mitigation: confidence-aware extraction and low-confidence review flags.

3. Long-running worker deadlock.
- Mitigation: timeout policy and supervised worker restart.

## 13. Definition of Technical Readiness
Ready for implementation when:
1. command surface is finalized.
2. table schema is approved.
3. worker protocol is versioned.
4. benchmark harness contract is accepted.
