# ALIV Implementation Backlog

## Scope
This backlog converts PRD requirements into execution-ready work items for Phase 0 and early Phase 1. Every item maps to PRD-001, PRD-002, PRD-003, and PRD-000 exit gates.

## Priority Legend
- P0: Required to pass current phase gates.
- P1: Strongly recommended after P0 baseline.
- P2: Nice-to-have after baseline is stable.

## Gate Metrics (Source of Truth)
Use metrics and protocol from docs/prd/PRD-000-MASTER.md:
- Ingestion success >= 98%
- Retrieval top-5 relevance >= 85%
- Extraction precision >= 95%
- Harmful action rate < 1%
- Query latency p95 < 8s
- Two consecutive benchmark passes required

## Workstream A: Foundation and Contracts (P0)

### A1. Repository and Runtime Skeleton
- PRD mapping: PRD-000 (architecture constraints)
- Tasks:
  - Create Rust and Tauri command surface skeleton.
  - Create Python worker runner with JSON-lines IPC contracts.
  - Define shared schema package for command payloads and worker events.
- Deliverables:
  - Stable command names and request/response contracts.
  - Versioned worker protocol document.
- Done when:
  - UI can call mock commands and receive typed responses.
  - Worker handshake succeeds and reports version.

### A2. SQLite and Migration Baseline
- PRD mapping: PRD-001, PRD-003
- Tasks:
  - Initialize SQLite in WAL mode.
  - Create migrations for documents, chunks, embeddings metadata, actions, audit logs, benchmark runs.
  - Add DB health and migration status checks.
- Deliverables:
  - Migration scripts and schema docs.
- Done when:
  - Fresh setup and upgrade path both succeed.
  - WAL mode verified in runtime diagnostics.

### A3. Telemetry Envelope and Run Metadata
- PRD mapping: PRD-000 validation protocol, PRD-001/002/003 telemetry
- Tasks:
  - Standardize run_id, corpus_version, model_versions, timestamp, machine_profile fields.
  - Add event sink for ingest/query/action metrics.
- Deliverables:
  - Telemetry schema and local append-only run logs.
- Done when:
  - Every benchmark run emits complete metadata fields.

## Workstream B: Ingestion and Indexing (PRD-001, P0)

### B1. Corpus Registration and Discovery
- PRD mapping: PRD-001 requirements 1, 2
- Tasks:
  - Add folder registration command and persistence.
  - Implement file discovery with extension filters and hash-based change detection.
  - Implement skip-unchanged logic.
- Acceptance checks:
  - Re-run on unchanged corpus skips at least 95% of files.

### B2. Parsing Pipeline with OCR Fallback
- PRD mapping: PRD-001 requirements 3, 4
- Tasks:
  - Integrate PyMuPDF extraction for native PDFs.
  - Add OCR branch for scanned/low-quality pages via Tesseract worker.
  - Support DOCX, EML, and image-based scanned docs in adapter architecture.
- Acceptance checks:
  - Corrupt and password-protected files fail gracefully with typed reasons.
  - Pipeline does not crash on unsupported files.

### B3. Chunking, Metadata, and Deterministic IDs
- PRD mapping: PRD-001 requirements 4, 5
- Tasks:
  - Define deterministic chunk ID format based on file hash, page/offset, chunk index.
  - Store metadata: path, type, modified time, content hash, inferred doc type.
- Acceptance checks:
  - Re-ingest of unchanged file produces identical chunk IDs.

### B4. Embeddings and sqlite-vec Persistence
- PRD mapping: PRD-001 requirements 6, 8
- Tasks:
  - Implement embeddings adapter interface.
  - Write vectors into sqlite-vec with referential links to chunk table.
  - Implement retry and partial-failure resume.
- Acceptance checks:
  - 200-file corpus indexing completes with ingestion success >= 98%.

### B5. Ingestion Benchmark Harness
- PRD mapping: PRD-001 acceptance, PRD-000 protocol
- Tasks:
  - Implement deterministic benchmark runner for ingestion.
  - Record discovered/indexed/skipped/failed and latency stats.
- Acceptance checks:
  - Two consecutive benchmark passes with required threshold.

## Workstream C: Retrieval and Extraction (PRD-002, P0)

### C1. Hybrid Retrieval Planner
- PRD mapping: PRD-002 requirements 1, 2
- Tasks:
  - Build metadata filter retrieval from SQLite.
  - Build semantic top-k retrieval from sqlite-vec.
  - Merge and rank candidates with deterministic tie-breaking in eval mode.
- Acceptance checks:
  - Top-5 relevance benchmark >= 85%.

### C2. Citation Binding and Evidence Contract
- PRD mapping: PRD-002 requirements 3 and acceptance criterion 3
- Tasks:
  - Define citation object schema (document id, chunk id, span, confidence).
  - Enforce citation-required outputs for non-empty answers.
- Acceptance checks:
  - No non-empty answer is returned without at least one valid citation.

### C3. Structured Extraction Templates
- PRD mapping: PRD-002 requirements 4, 5
- Tasks:
  - Implement extraction templates for expiry date, vendor name, penalty clause.
  - Add JSON and CSV export shaping.
  - Add field-level confidence and review flags.
- Acceptance checks:
  - Extraction precision >= 95% macro-average across required fields.

### C4. Low-Confidence and Contradiction Guardrails
- PRD mapping: PRD-002 section 6
- Tasks:
  - Define threshold behavior for low-confidence outputs.
  - Add contradictory evidence detector and response annotations.
- Acceptance checks:
  - Confidence below threshold never labeled as final answer.

### C5. Query Performance and Stability
- PRD mapping: PRD-002 NFR
- Tasks:
  - Add query profiling for p50 and p95.
  - Optimize top bottlenecks in retrieval/rerank/extraction path.
- Acceptance checks:
  - Query latency p95 < 8s over 50 benchmark queries.

## Workstream D: Safe File Actions (PRD-003, P0)

### D1. Recommendation Engine for Rename and Move
- PRD mapping: PRD-003 requirements 1, 2
- Tasks:
  - Implement filename normalization pattern.
  - Implement rule-based destination suggestion.
- Acceptance checks:
  - Recommendations include source evidence and confidence.

### D2. Preview-First Action Queue
- PRD mapping: PRD-003 requirements 3, 4
- Tasks:
  - Build before/after diff preview for each action.
  - Enable approve single and approve batch modes.
- Acceptance checks:
  - Dry-run default behavior enforced for all action paths.

### D3. Collision Handling and Idempotent Execution
- PRD mapping: PRD-003 requirements 5, NFR 2
- Tasks:
  - Implement deterministic suffixing and overwrite prevention.
  - Add idempotency keys for retries.
- Acceptance checks:
  - Zero unintended overwrites in collision tests.

### D4. Undo and Rollback
- PRD mapping: PRD-003 requirements 6, safety 3
- Tasks:
  - Record inverse operation metadata for every execution.
  - Implement undo command for rename and move.
- Acceptance checks:
  - Undo success rate 100% on test set.

### D5. Risk Scoring, Blocking, and Audit Trail
- PRD mapping: PRD-003 sections 6.1 and 6.2
- Tasks:
  - Implement harmful-action taxonomy classifier.
  - Block live mode on data-loss risk and safety gate failures.
  - Persist audit records (timestamp, reason, evidence, outcome).
- Acceptance checks:
  - Harmful action rate < 1% on approved dry-run recommendations.
  - Live mode stays disabled until two consecutive safety passes.

## Workstream F: Memory and Long Context (PRD-004, P1 — Phase 2)

### F1. Conversation Persistence
- PRD mapping: PRD-004 requirements 1, 2, 3
- Tasks:
  - Add `conversations` and `conversation_turns` tables via migration 0005.
  - Implement `create_conversation`, `list_conversations`, `get_conversation_turns`, `save_conversation_turn` in `infra/db.rs`.
  - Expose `start_conversation`, `list_conversations`, `get_conversation` Tauri commands.
  - Restore last active conversation on workspace load.
- Acceptance checks:
  - Conversation history survives app restart.
  - Last 10 turns load within 100ms from SQLite.

### F2. Context Assembly and Token Budget
- PRD mapping: PRD-004 requirements 4, 5
- Tasks:
  - Wire `MemoryService::assemble_context()` into the retrieval query path.
  - Pass `conversation_id` as optional parameter to `run_retrieval_query` command.
  - Save assistant answer as a turn after every query.
  - Enforce token budget — never send oversized prompt to Ollama.
- Acceptance checks:
  - Assembled prompt never exceeds `model_context_window` tokens.
  - Context assembly completes in < 200ms (measured via telemetry).

### F3. Document Summary Cache
- PRD mapping: PRD-004 requirements 5, 6
- Tasks:
  - Add `document_summaries` table via migration 0005.
  - Add post-ingestion hook to trigger `summarization_worker.py` per file.
  - Implement `save_document_summary`, `get_document_summary`, `invalidate_document_summaries` in `infra/db.rs`.
  - Invalidate summaries when file hash changes on re-ingestion.
- Acceptance checks:
  - 100% of successfully indexed files have a document-level summary within 30s post-ingestion.
  - Summary regenerated after file content changes.

### F4. User Memory Facts
- PRD mapping: PRD-004 requirements 7, 8, 9
- Tasks:
  - Add `memory_facts` table via migration 0005.
  - Implement fact CRUD in `infra/db.rs`.
  - Add semantic relevance retrieval for facts at query time (keyword match initially; embedding match after B4 embeddings are stable).
  - Enforce scope expiry — exclude expired facts from query context.
- Acceptance checks:
  - Pinned facts recalled in 80%+ of relevant queries (cosine sim > 0.7 test set).
  - Session-scoped facts not recalled after restart.

### F5. Memory UI (Phase 5)
- PRD mapping: PRD-004 requirement 10
- Tasks:
  - Add conversation thread panel to Query Workbench screen.
  - Add "Remember this" button to answer and extraction result rows.
  - Add Memory Facts manager panel (list, delete, scope labels).
- Acceptance checks:
  - User can view full conversation history in Query Workbench.
  - User can pin and delete memory facts from UI.

## Workstream E: End-to-End Validation and Reports (P0)

### E1. Unified Benchmark Runner
- PRD mapping: PRD-000 protocol and exit criteria
- Tasks:
  - Run ingestion, retrieval, extraction, and action safety suites in one command.
  - Emit machine-readable and human-readable reports.
- Acceptance checks:
  - Reports include all gate metrics and pass/fail for each run.

### E2. Regression Guardrails
- PRD mapping: PRD-000 validation protocol item 4
- Tasks:
  - Add threshold assertions in CI/local validation task.
  - Block phase exit if any safety metric regresses.
- Acceptance checks:
  - Safety regression reliably fails the run.

## Suggested Build Sequence (Execution Order)
1. A1 -> A2 -> A3
2. B1 -> B2 -> B3 -> B4 -> B5
3. C1 -> C2 -> C3 -> C4 -> C5
4. D1 -> D2 -> D3 -> D4 -> D5
5. E1 -> E2

## Week-Level Starter Plan

### Week 1
- Target: complete A and B1-B3.
- Checkpoint:
  - Deterministic chunk IDs validated.
  - Discovery and incremental skip working.

### Week 2
- Target: finish B4-B5 and C1-C2.
- Checkpoint:
  - First retrieval benchmark run available.

### Week 3
- Target: C3-C5 plus D1-D2.
- Checkpoint:
  - Field extraction precision and query latency near target.

### Week 4
- Target: D3-D5 and E1-E2.
- Checkpoint:
  - Full gate run complete with two consecutive pass attempt.

## Backlog Risk Register
- R1: OCR quality variance causes extraction drift.
  - Mitigation: confidence gating, OCR quality score, fallback path.
- R2: Embedding model changes break reproducibility.
  - Mitigation: strict model version pinning in run metadata.
- R3: File action safety false positives/negatives.
  - Mitigation: keep dry-run default and hard block on data-loss risk.
- R4: Performance misses on reference machine.
  - Mitigation: profile-guided optimization and caching of immutable artifacts.

## Definition of Done for Current Phase
Current phase is done only when:
- All five PRD-000 threshold metrics pass.
- Two consecutive benchmark runs pass.
- No critical safety defects remain open.
