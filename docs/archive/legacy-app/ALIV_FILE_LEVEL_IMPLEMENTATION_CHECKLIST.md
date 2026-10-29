# ALIV File-Level Implementation Checklist

## Goal
Translate ALIV backlog into concrete files and directories to create in build order.

## How to Use
1. Implement sections in sequence.
2. Mark each file as Created, Stubbed, Implemented, or Verified.
3. Run benchmark checks after each major section.

## Status Key
- [ ] Not started
- [~] In progress
- [x] Completed

## 0. Root Structure and Tooling (P0)

### 0.1 Directories
- [x] src-tauri/
- [x] src/
- [x] python/
- [x] db/
- [x] benchmarks/
- [x] logs/

### 0.2 Root files
- [x] README.md
- [x] .gitignore
- [x] .editorconfig
- [x] docs/IMPLEMENTATION_NOTES.md

## 1. Rust and Tauri Core (P0)

### 1.1 Tauri app shell
- [x] src-tauri/Cargo.toml
- [x] src-tauri/src/main.rs
- [x] src-tauri/src/lib.rs
- [x] src-tauri/tauri.conf.json

### 1.2 Command surface and routing
- [x] src-tauri/src/commands/mod.rs
- [x] src-tauri/src/commands/workspace.rs
- [x] src-tauri/src/commands/ingestion.rs
- [x] src-tauri/src/commands/retrieval.rs
- [x] src-tauri/src/commands/extraction.rs
- [x] src-tauri/src/commands/actions.rs
- [x] src-tauri/src/commands/benchmark.rs
- [x] src-tauri/src/commands/telemetry.rs

Required command handlers to expose:
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

### 1.3 Core modules
- [x] src-tauri/src/core/mod.rs
- [x] src-tauri/src/core/workspace.rs
- [x] src-tauri/src/core/ingestion.rs
- [x] src-tauri/src/core/index.rs
- [x] src-tauri/src/core/retrieval.rs
- [x] src-tauri/src/core/actions.rs
- [x] src-tauri/src/core/telemetry.rs

### 1.4 Infra modules
- [x] src-tauri/src/infra/mod.rs
- [x] src-tauri/src/infra/db.rs
- [x] src-tauri/src/infra/python_bridge.rs
- [x] src-tauri/src/infra/fs_guard.rs

### 1.5 Shared models and errors
- [x] src-tauri/src/models/mod.rs
- [x] src-tauri/src/models/contracts.rs
- [x] src-tauri/src/models/errors.rs

## 2. Python Worker Layer (P0)

### 2.1 Worker runtime
- [x] python/pyproject.toml
- [x] python/requirements.txt
- [x] python/workers/__init__.py
- [x] python/workers/runner.py
- [x] python/workers/protocol.py
- [x] python/workers/errors.py

### 2.2 Parse worker
- [x] python/workers/parse_worker.py
- [x] python/workers/parsers/__init__.py
- [x] python/workers/parsers/pdf_parser.py
- [x] python/workers/parsers/docx_parser.py
- [x] python/workers/parsers/email_parser.py
- [x] python/workers/parsers/image_parser.py

### 2.3 OCR worker
- [x] python/workers/ocr_worker.py
- [x] python/workers/ocr/tesseract_adapter.py

### 2.4 Extraction worker
- [x] python/workers/extraction_worker.py
- [x] python/workers/extractors/templates.py
- [x] python/workers/extractors/citation_binding.py

### 2.5 Embedding adapter bridge
- [x] python/workers/embedding_worker.py
- [x] python/workers/embeddings/tei_adapter.py
- [x] python/workers/embeddings/model_registry.py

## 3. Database and Migrations (P0)

### 3.1 Migration framework
- [x] db/README.md
- [x] db/migrations/0001_init.sql
- [x] db/migrations/0002_embeddings_vec.sql
- [x] db/migrations/0003_actions_audit.sql
- [x] db/migrations/0004_benchmark_runs.sql

### 3.2 Schema docs
- [x] db/schema/current_schema.md
- [x] db/schema/erd.md

### 3.3 Seed and diagnostics
- [x] db/scripts/check_wal.sql
- [x] db/scripts/health_checks.sql

Expected initial tables:
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

## 4. Frontend UI Layer (P0)

### 4.1 App shell
- [ ] src/main.tsx
- [ ] src/App.tsx
- [ ] src/styles/global.css

### 4.2 API client and contracts
- [ ] src/api/tauriClient.ts
- [ ] src/api/contracts.ts
- [ ] src/api/errorMapper.ts

### 4.3 Feature views
- [ ] src/features/workspace/WorkspaceSetup.tsx
- [ ] src/features/ingestion/IngestionRunPanel.tsx
- [ ] src/features/retrieval/QueryWorkbench.tsx
- [ ] src/features/extraction/ExtractionResultsPanel.tsx
- [ ] src/features/actions/ActionPreviewQueue.tsx
- [ ] src/features/actions/ActionAuditPanel.tsx
- [ ] src/features/benchmark/BenchmarkDashboard.tsx
- [ ] src/features/diagnostics/DiagnosticsPanel.tsx

### 4.4 Shared UI state
- [ ] src/state/appStore.ts
- [ ] src/state/runStore.ts

## 5. Benchmark Assets and Harness (P0)

### 5.1 Corpus and labels
- [ ] benchmarks/corpus/README.md
- [ ] benchmarks/manifests/corpus_manifest.v1.json
- [ ] benchmarks/labels/questions_gold.v1.json
- [ ] benchmarks/labels/field_labels.v1.json

### 5.2 Runner and metrics
- [ ] benchmarks/runner/run_benchmarks.ps1
- [ ] benchmarks/runner/run_benchmarks.py
- [ ] benchmarks/runner/metrics.py
- [ ] benchmarks/runner/report_writer.py
- [x] benchmarks/validate_ingestion_thresholds.py
- [x] benchmarks/generate_sample_ingestion_runs.py

### 5.3 Outputs
- [ ] benchmarks/runs/.gitkeep
- [ ] benchmarks/reports/.gitkeep

Required metrics per run:
1. ingestion_success_rate
2. retrieval_top5_relevance
3. extraction_precision_macro
4. harmful_action_rate
5. query_latency_p50
6. query_latency_p95

## 5.5 Memory and Long Context (Phase 2)

### 5.5.1 PRD
- [x] docs/prd/PRD-004-MEMORY-LONG-CONTEXT.md

### 5.5.2 Database migration
- [x] db/migrations/0005_memory_context.sql
  - conversations, conversation_turns, document_summaries, memory_facts tables

### 5.5.3 Rust core
- [x] src-tauri/src/core/memory.rs
  - MemoryService, ContextBlock, AssembledContext, TokenBudget, BudgetAllocation
  - estimate_tokens(), derive_title(), assemble_context(), format_prompt()

### 5.5.4 Rust commands
- [x] src-tauri/src/commands/memory.rs
  - start_conversation, list_conversations, get_conversation
  - save_memory_fact, list_memory_facts, delete_memory_fact
  - get_document_summary, build_long_context
- [x] src-tauri/src/commands/mod.rs — memory module registered
- [x] src-tauri/src/core/mod.rs — memory module registered

### 5.5.5 Contracts
- [x] src-tauri/src/models/contracts.rs — memory types added
  (StartConversationRequest, ConversationStub, ConversationTurnView,
   SaveMemoryFactRequest, MemoryFactView, BuildLongContextRequest, LongContextResponse)

### 5.5.6 Python worker
- [x] python/workers/summarization_worker.py
  - Ollama HTTP API, document/section granularities, temperature=0.1

### 5.5.7 Database methods (infra/db.rs — to implement in Phase 2)
- [ ] create_conversation(id, workspace_path, title)
- [ ] list_conversations(workspace_path, limit) → Vec<ConversationStub>
- [ ] get_conversation_turns(conversation_id, limit) → Vec<ConversationTurnView>
- [ ] save_conversation_turn(conversation_id, role, content, citations, token_count)
- [ ] update_conversation_title(conversation_id, title)
- [ ] save_memory_fact(id, workspace_path, content, citation_json, scope, expires_at)
- [ ] list_memory_facts(workspace_path) → Vec<MemoryFactView> (excludes expired)
- [ ] delete_memory_fact(fact_id)
- [ ] save_document_summary(file_id, file_hash, granularity, section_index, content, token_count, model_version)
- [ ] get_document_summary(file_id, granularity) → Option<String>
- [ ] invalidate_document_summaries(file_id) — called on re-ingestion when hash changes

### 5.5.8 Tests
- [ ] src-tauri/src/core/memory.rs — unit tests included in file
- [ ] python/tests/test_summarization_worker.py

## 6. Safety and Action Controls (P0)

### 6.1 Policy and risk rules
- [ ] src-tauri/src/policy/mod.rs
- [ ] src-tauri/src/policy/action_risk_rules.rs
- [ ] src-tauri/src/policy/path_allowlist.rs
- [ ] src-tauri/src/policy/safety_blockers.rs

### 6.2 Rollback and audit
- [ ] src-tauri/src/core/undo.rs
- [ ] src-tauri/src/core/audit.rs

Harmful action classes to enforce:
1. Wrong destination folder.
2. Wrong normalized filename values.
3. Data-loss or overwrite risk.
4. Irreversible action without undo path.

## 7. Tests (P0)

### 7.1 Rust tests
- [ ] src-tauri/tests/ingestion_tests.rs
- [ ] src-tauri/tests/retrieval_tests.rs
- [ ] src-tauri/tests/actions_safety_tests.rs
- [ ] src-tauri/tests/undo_tests.rs

### 7.2 Python tests
- [x] python/tests/test_parse_worker.py
- [ ] python/tests/test_ocr_worker.py
- [ ] python/tests/test_extraction_worker.py
- [ ] python/tests/test_embedding_worker.py
- [x] python/tests/test_parsers.py
- [x] python/tests/test_validate_ingestion_thresholds.py

### 7.3 End-to-end tests
- [ ] tests/e2e/test_benchmark_gate.py
- [ ] tests/e2e/test_preview_first_actions.py

## 8. Local Ops and Validation (P0)

### 8.1 Scripts
- [ ] scripts/bootstrap.ps1
- [ ] scripts/run_app.ps1
- [ ] scripts/run_ingestion.ps1
- [ ] scripts/run_benchmarks.ps1
- [ ] scripts/export_report.ps1

### 8.2 CI or local gate config
- [ ] .github/workflows/benchmark-gate.yml (optional now)
- [ ] scripts/check_phase_exit.ps1

check_phase_exit should fail if any gate fails:
1. ingestion_success_rate < 0.98
2. retrieval_top5_relevance < 0.85
3. extraction_precision_macro < 0.95
4. harmful_action_rate >= 0.01
5. query_latency_p95 >= 8.0

## 9. Suggested Build Order (Apply In Sequence)
1. 0 -> 1.1 -> 1.2 -> 1.4
2. 3.1 -> 3.2
3. 2.1 -> 2.2 -> 2.3
4. 1.3 ingestion/index + 2.5 + 3 tables
5. 5.1 and 5.2 first-pass benchmark runner
6. 1.3 retrieval + 2.4 extraction
7. 6 safety modules + action command handlers
8. 4 frontend feature panels
9. 7 tests and 8 gate checks

## 10. Phase-Exit Checklist
- [ ] Two consecutive benchmark runs recorded.
- [ ] All five PRD-000 thresholds pass.
- [ ] No critical safety defects open.
- [ ] Audit logs complete for executed action test set.
- [ ] Undo success is 100% on test set.
