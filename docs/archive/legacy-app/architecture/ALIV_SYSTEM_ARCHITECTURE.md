# ALIV System Architecture

## 1. Architecture Vision
ALIV is a local-first, safety-first desktop intelligence platform designed for sensitive document workflows. The system is optimized for:
1. Data locality and privacy.
2. Deterministic product validation.
3. Reliable automation with rollback.
4. Modular evolution from solo build to production-grade platform.

## 2. Architectural Style
Primary style:
1. Modular monolith at application level for early stage velocity.
2. Hexagonal architecture inside each module for testability and substitution.
3. Event-driven internal workflow for ingestion, indexing, and action execution.

Rationale:
1. Faster development and debugging for solo founder.
2. Strong boundaries to reduce rewrite risk later.
3. Straight path to service decomposition when scale requires it.

## 3. System Context
Main runtime components:
1. Desktop control center UI (React and TypeScript via Tauri).
2. Local orchestration core (Rust and Tauri commands).
3. Parsing and ML worker layer (Python subprocess bridge).
4. Local persistence layer (SQLite and sqlite-vec).
5. Local file system watcher and action engine.

Data stays on local machine by default.

## 4. Logical Architecture
### 4.1 Core Modules
1. Workspace Manager
- Folder registration, scan policies, ignore rules.

2. Ingestion Pipeline
- Discovery, parsing, OCR fallback, chunking, metadata extraction.
- Post-ingestion trigger: summarization worker generates document/section summaries.

3. Indexing Engine
- Metadata indexing in SQLite and embedding persistence in sqlite-vec.

4. Retrieval Engine
- Hybrid query planner and ranker.
- Integrates with Memory Engine to inject conversation history and memory facts.

5. Extraction Engine
- Structured field extraction with confidence and citation binding.

6. Memory Engine (PRD-004)
- Conversation persistence: multi-turn history across sessions (conversations, conversation_turns).
- Document summary cache: pre-computed Ollama summaries at document and section granularity.
- User memory facts: pinned facts with citations and expiry scope.
- Context assembler: token-budget-aware prompt builder (MemoryService::assemble_context).
- Prompt formatter: assembles [MEMORY] / [CONVERSATION HISTORY] / [DOCUMENT CONTEXT] / [QUESTION] blocks.

7. Action Planner
- Filename and folder recommendations from extracted metadata.

8. Action Executor
- Preview queue, approvals, execution, rollback, audit logging.

9. Telemetry and Evaluation
- Benchmark runs, metric calculations, failure taxonomy, run reports.

### 4.2 Cross-Cutting Modules
1. Security and Policy Guardrails.
2. Config and version management.
3. Error handling and retries.
4. Observability and diagnostics.

## 5. Data Architecture
Primary stores:
1. SQLite relational tables for files, chunks, metadata, runs, actions, and audit events.
2. sqlite-vec table for embedding vectors linked to chunk ids.

Key entities:
1. FileRecord.
2. ChunkRecord.
3. EmbeddingRecord.
4. QueryRun.
5. ExtractionResult.
6. ActionRecommendation.
7. ActionExecution.
8. UndoRecord.

Design rules:
1. Use immutable run snapshots for benchmark reproducibility.
2. Keep explicit foreign keys between file, chunk, and extraction output.
3. Preserve source path history for rollback and diagnostics.

## 6. Runtime Workflow
### 6.1 Ingest and Index
1. Watcher emits file discovered event.
2. Ingestion pipeline resolves parser strategy.
3. Extracted text is chunked and normalized.
4. Metadata and chunks are written to SQLite.
5. Embeddings are generated and written to sqlite-vec.
6. Completion event updates run telemetry.

### 6.2 Retrieve and Extract
1. User query enters retrieval engine.
2. Engine builds metadata filter plus vector query plan.
3. Candidate chunks are ranked and selected.
4. Extraction engine returns structured output with citations and confidence.
5. Query run metrics are persisted.

### 6.3 Safe File Actions
1. Planner creates rename and move recommendations.
2. Recommendations enter preview queue.
3. Approved actions execute with collision-safe strategy.
4. Every action produces audit and undo record.

## 7. Safety and Trust Architecture
1. Preview-first operation for all file actions.
2. High-risk actions require explicit confirmation.
3. One-click undo supported for each execution.
4. Action engine blocks operations that risk overwrite or data loss.
5. Citation-required output path for extraction results.

## 8. Performance Architecture
Targets from PRDs:
1. Query p95 under 8 seconds.
2. Ingestion success at least 98 percent.

Performance strategies:
1. Batch writes for chunk and embedding persistence.
2. Incremental indexing with file hash checks.
3. Async task queues for heavy parsing and embedding generation.
4. Adaptive chunk sizing for large files.

## 9. Reliability Architecture
1. Retry policies around parser and embedding workers.
2. Poison event isolation for repeatedly failing files.
3. Structured failure taxonomy with root-cause tags.
4. Resume-safe ingestion runs with checkpoints.

## 10. Security Architecture
1. Local-only data processing by default.
2. Optional at-rest database encryption layer.
3. Least-privilege filesystem access scope.
4. Signed and versioned configuration profile.
5. Full local audit trail of high-impact actions.

## 11. Evolution Roadmap
Phase 1:
1. Local modular monolith with benchmark harness.

Phase 2:
1. Split parser and embedding workers as isolated process services.
2. Add background scheduler and policy packs.

Phase 3:
1. Multi-workspace tenancy and policy-based role controls.
2. Optional secure remote sync for non-sensitive metadata only.

## 12. Architecture Decision Summary
1. Use modular monolith first, not microservices.
2. Use hybrid retrieval, not vector-only search.
3. Keep file actions preview-first until safety thresholds pass.
4. Treat benchmarks as product gate, not optional QA.
