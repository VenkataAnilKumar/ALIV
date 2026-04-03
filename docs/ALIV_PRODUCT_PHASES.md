# ALIV Product Phases

## Purpose
Define a practical, solo-founder execution path from foundational build to production-ready release.

## Planning Assumptions
1. Build follows product-only validation first.
2. Scope expansion is allowed only after each phase passes exit criteria.
3. Quality and safety gates are mandatory, not optional.

## Phase 0: Foundation Setup
### Objective
Establish the minimal technical foundation for repeatable development and validation.

### Scope
1. Repository structure alignment.
2. Baseline schemas and migration framework.
3. Local runtime configuration strategy.
4. Benchmark artifact skeleton.

### Deliverables
1. Working project skeleton for Rust or Tauri, React, Python bridge, and DB migration folders.
2. Initial SQLite schema for files, chunks, embeddings, runs, and actions.
3. Config profile format with environment-specific overrides.

### Exit Criteria
1. App boots successfully in local dev mode.
2. DB migrations execute and rollback safely.
3. Benchmark folders and templates are committed.

### Estimated Duration
3 to 5 days.

### Gate Result: COMPLETE
- App boots in local dev mode.
- DB migrations execute correctly (0001–0008).
- Benchmark folder structure committed.

## Phase 1: Ingestion and Indexing (PRD-001)
### Objective
Ingest, parse, chunk, and index local files reliably.

### Scope
1. File discovery and change detection.
2. Parsing and OCR fallback.
3. Metadata extraction and chunk persistence.
4. Embedding generation and sqlite-vec indexing.

### Deliverables
1. Ingestion pipeline with resumable runs.
2. Incremental re-index behavior.
3. Failure taxonomy and ingestion metrics.

### Exit Criteria
1. Ingestion success rate >= 98%.
2. Incremental re-run no-op >= 95% for unchanged files.
3. No fatal crash on benchmark corpus.

### Estimated Duration
1 to 2 weeks.

### Gate Result (2026-04-02): PASS — COMPLETE
- Full run: 200/200 parsed, success_rate=1.0 (threshold 0.98 ✓)
- Incremental run: noop_ratio=1.0 (threshold 0.95 ✓)
- Two consecutive passing runs completed.
- Phase 1 → COMPLETE. Begin Phase 2 (PRD-002, PRD-004).

## Phase 2: Retrieval, Extraction, and Memory (PRD-002, PRD-004)
### Objective
Answer benchmark queries with citations, high extraction precision, and persistent conversation memory.

### Scope
1. Hybrid retrieval (metadata plus vector).
2. Evidence ranking and citation binding.
3. Structured extraction for critical fields.
4. **Conversation memory**: persist multi-turn query history across sessions.
5. **Long context assembly**: token-budget-aware context building (Workstream F1, F2).
6. **Document summary cache**: post-ingestion summarization via Ollama (Workstream F3).

### Deliverables
1. Query workbench backend contract with optional `conversation_id`.
2. Extraction templates and confidence scoring.
3. Benchmark scoring report for retrieval and extraction.
4. Conversation persistence (SQLite) — history survives restart.
5. Context assembler wired into query path with token budget enforcement.
6. Document-level summaries generated for all indexed files.

### Exit Criteria
1. Retrieval relevance top-5 >= 85%.
2. Extraction precision >= 95% on critical fields.
3. Non-empty answers include valid citations.
4. Conversation history loads < 100ms from SQLite after restart.
5. Assembled prompt never exceeds model context window.
6. Document summaries generated for >= 95% of successfully indexed files.

### Estimated Duration
1 to 2 weeks.

### Gate Result (2026-04-02): PASS — COMPLETE
- RRF hybrid retrieval implemented (55% lexical + 45% semantic).
- Extraction templates for 5 fields: expiry_date, vendor_name, penalty_clause, amount, client_name.
- Citation binding verified — all answers include source + page reference.
- Memory system fully implemented (PRD-004):
  - `db/migrations/0007_memory_context.sql` — conversations, conversation_turns, document_summaries, memory_facts tables.
  - `src-tauri/src/core/memory.rs` — MemoryService, TokenBudget (8192→6912 usable), context assembler.
  - `src-tauri/src/commands/memory.rs` — 8 Tauri commands wired.
  - `python/workers/summarization_worker.py` — Ollama HTTP summarization (document + section granularity).
- Token budget allocation: facts 10% / history 20% / chunks 60% / summaries 10%.

## Phase 3: Safe File Actions (PRD-003)
### Objective
Provide preview-first file action automation with rollback and auditability.

### Scope
1. Recommendation planner for rename and move.
2. Preview queue and risk labeling.
3. Execution and undo flows.
4. Collision-safe naming strategy.

### Deliverables
1. Action preview API and UI surface.
2. Undo-capable execution engine.
3. Action audit log and safety reports.

### Exit Criteria
1. Harmful action rate < 1% in dry-run validation.
2. Undo success rate = 100% on test set.
3. Collision handling prevents overwrite in all tested cases.

### Estimated Duration
1 to 2 weeks.

### Gate Result (2026-04-03): PASS — COMPLETE
- `infra/fs_guard.rs` — real path validation: traversal detection, workspace boundary enforcement, symlink rejection, collision-safe suffix strategy (_001–_999), filename normalization ({DocType}_{Vendor}_{YYYY-MM-DD}.ext).
- `core/actions.rs` — ActionService: evaluate_risk(), build_undo_record_json(), execute_rename() with collision-safe destination.
- `commands/actions.rs` — all 7 handlers fully wired:
  - generate_action_recommendations → DB query + risk evaluation + preview queue
  - get_action_preview_queue → returns current queue count
  - evaluate_action_risk → re-evaluates risk per recommendation ID
  - approve_actions → sets approval gate in AppState
  - execute_actions → enforces approval gate, renames files, writes undo records, partial failure safe
  - undo_action_batch → reverses all executions in a batch via inverse_operation_json
  - get_action_audit_log → returns full audit trail from action_executions + action_recommendations
- `state.rs` — drain_preview_queue(), set_approved_preview_queue(), is_action_approval_active(), clear_action_approval() added.
- DB methods added: save_recommendation, list_recommendations, save_execution, save_undo_record, get_undo_records_for_batch, list_audit_log, get_recommendation, update_execution_status, list_files_needing_rename.
- Safety: execute_actions hard-errors if approve_actions not called first. Undo record failure triggers immediate rename reversal.

## Phase 4: End-to-End Hardening
### Objective
Improve reliability, observability, and consistency for sustained usage.

### Scope
1. Error recovery and retry policy tuning.
2. Worker timeout and supervision controls.
3. Structured diagnostics and benchmark trend tracking.

### Deliverables
1. Full benchmark suite automation.
2. Run comparison dashboard or report output.
3. Defect backlog prioritized by impact.

### Exit Criteria
1. Two consecutive full benchmark runs pass all master thresholds.
2. No open critical safety defects.
3. Stable p95 latency within target on reference machine.

### Estimated Duration
1 week.

### Gate Result (2026-04-03): PASS — COMPLETE
Four defects identified and all resolved:

| ID | Title | Severity | Resolution |
|---|---|---|---|
| D001 | embedding_count always 0 in ingestion benchmark | critical | _embed_chunk_batch() wired into run_benchmark(), actual count recorded |
| D002 | sqlite-vec migration 0008 skipped in Python benchmarks | high | _load_sqlite_vec() loads Python wheel; migration 0008 runs when available; _populate_vec_index() syncs vectors into ANN index; sqlite-vec added to requirements.txt |
| D003 | Python worker no retry on transient timeout | high | run_worker() wraps run_worker_once() with 3-attempt retry loop, 500ms/1000ms exponential backoff on timeout |
| D004 | FilesystemGuard only validates non-empty string | critical | Replaced with real validate_and_canonicalize() (resolved in Phase 3) |

- Defect backlog migrated from markdown to structured JSON (docs/PHASE4_DEFECT_BACKLOG.json).
- All 4 defects closed. open_critical_count = 0.
- Two consecutive full-suite benchmark runs passed: ingestion ✓, retrieval ✓, extraction ✓, no open critical defects ✓.
- Phase 4 exit criteria met. Evidence: benchmarks/results/full_suite_latest.json.

## Phase 5: UX and Operator Experience
### Objective
Polish control-center usability for high-frequency workflows.

### Scope
1. Improve task flow for ingestion, query, extraction, and actions.
2. Add accessibility and keyboard-first improvements.
3. Clarify risk states and recovery guidance.

### Deliverables
1. Improved navigation and flow consistency.
2. Accessibility checks and fixes.
3. Updated UX copy and state messaging.

### Exit Criteria
1. Core tasks completed with minimal friction in internal test runs.
2. Safety and confidence states are clearly understandable.
3. No regressions in benchmark performance.

### Estimated Duration
1 week.

## Phase 6: Release Candidate Preparation
### Objective
Prepare ALIV for controlled external usage.

### Scope
1. Installation and update strategy.
2. Logging and support diagnostics package.
3. Release checklist and rollback plan.

### Deliverables
1. Release candidate build artifacts.
2. Setup and troubleshooting documentation.
3. Versioned release notes and known limitations.

### Exit Criteria
1. Clean install and upgrade tests pass.
2. Critical paths verified after packaged build.
3. Go or no-go release review completed.

### Estimated Duration
1 week.

## Cross-Phase Governance
### Canonical Quality Gates
Threshold authority is defined in docs/prd/PRD-000-MASTER.md.

### Change Control Rules
1. Any threshold change must update PRD-000 first.
2. Any safety-impacting change requires benchmark re-run.
3. Phase completion must include written summary and pass or fail decision.

## Recommended Execution Cadence (Solo Founder)
1. Monday: phase planning and backlog selection.
2. Tuesday to Thursday: implementation and subsystem tests.
3. Friday: benchmark run and defect triage.
4. Saturday: fixes for top failures.
5. Sunday: phase report and next-phase gate decision.

## Current Phase Focus
1. Active phase: Phase 5 (UX and Operator Experience)
2. Active start date: 2026-04-03
3. Last gate review date: 2026-04-03
4. Phase 4 exit evidence: benchmarks/results/full_suite_latest.json

## Historical Notes
Phase 1 checklist above was superseded by implemented benchmark automation and two consecutive passing full-suite runs recorded during Phase 4 hardening.

## Phase Status Tracker
| Phase | Status | Start Date | End Date | Gate Result | Notes |
| --- | --- | --- | --- | --- | --- |
| Phase 0 | Completed | 2026-04-02 | 2026-04-02 | Pass | Foundation setup completed and baseline validation scripts in place |
| Phase 1 | Completed | 2026-04-02 | 2026-04-02 | Pass | Ingestion and indexing validated with benchmark gates |
| Phase 2 | Completed | 2026-04-02 | 2026-04-02 | Pass | RRF hybrid retrieval, extraction templates, citation binding, full memory system (PRD-004) |
| Phase 3 | Completed | 2026-04-02 | 2026-04-03 | Pass | All 7 action handlers wired, real FilesystemGuard, approval gate, undo reversal, audit log |
| Phase 4 | Completed | 2026-04-03 | 2026-04-03 | Pass | All 4 defects closed (D001–D004), two consecutive benchmark suite passes, open_critical_count=0 |
| Phase 5 | In Progress | 2026-04-03 |  | Pending | UX and operator experience improvements |
| Phase 6 | Not Started |  |  |  | |
