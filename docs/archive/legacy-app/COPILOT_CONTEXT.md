# ALIV — Copilot Session Context Prompt

> Paste this entire file at the start of every Copilot/AI session to restore full project context.
> Last updated: 2026-04-03

---

## What Is ALIV

**Autonomous Local Intelligence Vault** — a local-first desktop document intelligence app for legal and finance teams. It ingests local PDFs, Word docs, and emails, indexes them using local AI models (no cloud), and lets users query, extract structured facts, and autonomously organize files — all offline and privacy-safe.

Built with: **Rust + Tauri 2** (backend), **React + TypeScript + Vite** (frontend), **SQLite WAL + sqlite-vec** (database), **Python subprocess bridge** (parsing/OCR/ML workers), **Ollama** (local LLM inference at localhost:11434).

---

## Repository Structure

```
ALIV/
├── src-tauri/src/
│   ├── commands/       — Tauri command handlers (ingestion, retrieval, extraction, actions, memory, benchmark, telemetry, workspace)
│   ├── core/           — Business logic (ingestion, retrieval, actions, memory, telemetry, workspace, index)
│   ├── infra/          — Infrastructure (db.rs, python_bridge.rs, fs_guard.rs)
│   ├── models/         — Shared types (contracts.rs, errors.rs)
│   └── state.rs        — AppState
├── src/                — React frontend (not yet implemented)
├── python/workers/     — Python parsing/OCR/embedding/summarization workers
├── db/migrations/      — SQLite migrations (0001–0005)
├── benchmarks/         — Benchmark runner + validation scripts
└── docs/               — All product, architecture, design, PRD docs
```

---

## Current Phase Status

### Completed Phases
1. Phase 1 complete: ingestion and indexing validated.
2. Phase 2 complete: retrieval and extraction thresholds met.
3. Phase 3 complete: preview-first action command surface and rollback scaffolding are in place.
4. Phase 4 complete: end-to-end hardening passed with two consecutive full-suite runs.

### Active Phase
Phase 5 — UX and operator experience.

### Canonical Status and Evidence
1. `docs/ALIV_PRODUCT_PHASES.md` — source of truth for phase tracker.
2. `docs/PHASE4_HARDENING_REPORT.md` — phase 4 summary and metrics.
3. `benchmarks/results/full_suite_latest.json` — latest full-suite run evidence.

---

---

## Key Architecture Rules

### IPC Contract (Rust ↔ Python)
All workers use JSON-lines over stdin/stdout:
```
Request:  {"request_id": "...", "worker_type": "parse|embed|ocr|extract|summarize", "payload": {...}, "timeout_ms": 30000}
Response: {"request_id": "...", "status": "ok|error", "result": {...}, "error_code": null, "error_detail": null}
```
Worker protocol is defined in `python/workers/protocol.py` (Pydantic models: `WorkerRequest`, `WorkerResponse`).
`python_bridge.rs::run_worker()` spawns the process, writes request JSON to stdin, reads response from stdout.

### Database Patterns
- All DB access through `src-tauri/src/infra/db.rs` via `Database` struct
- Use `rusqlite` with WAL mode: `PRAGMA journal_mode=WAL`
- Chunk IDs are deterministic: `{file_hash}_{page}_{chunk_index}`
- File IDs: UUID v4 strings
- All timestamps: ISO-8601 UTC strings
- sqlite-vec vectors stored as JSON arrays (`[0.1, 0.2, ...]`) in `embeddings` table and as binary blobs in `embeddings_vec` virtual table

### Command Pattern
Every Tauri command:
1. Validates inputs
2. Uses `State<'_, Database>` and `State<'_, AppState>`
3. Returns `Result<T, String>` (errors are `.to_string()` mapped)
4. Records command in `state.record_command("command_name")`

### Error Handling
- Python worker errors: check `response.status == "error"`, return typed `AlIVError`
- DB errors: propagate with `.map_err(|e| e.to_string())`
- File not found / unsupported type: log and continue (never crash the pipeline)

---

## Local AI Stack

| Component | Config |
|---|---|
| LLM | Ollama `llama3.2:3b` at `http://localhost:11434` |
| Fallback LLM | `mistral:7b-instruct-q4_K_M` |
| Embeddings | `all-MiniLM-L6-v2` (384 dims, batch 32) via sentence-transformers |
| OCR | Tesseract 5.x, min confidence 60, language `eng` |
| Summarization | Ollama HTTP POST `/api/generate`, temperature 0.1, max 300 tokens |

---

## Memory System (PRD-004 — implemented in Phase 2)

**Files already created:**
- `src-tauri/src/core/memory.rs` — `MemoryService`, `TokenBudget`, `ContextBlock`, `AssembledContext`
- `src-tauri/src/commands/memory.rs` — 8 commands: `start_conversation`, `list_conversations`, `get_conversation`, `save_memory_fact`, `list_memory_facts`, `delete_memory_fact`, `get_document_summary`, `build_long_context`
- `python/workers/summarization_worker.py` — Ollama HTTP summarization
- `db/migrations/0005_memory_context.sql` — `conversations`, `conversation_turns`, `document_summaries`, `memory_facts` tables

**Still needs (Phase 2 implementation):**
- `infra/db.rs` memory methods: `create_conversation`, `save_conversation_turn`, `list_conversations`, `get_conversation_turns`, `save_memory_fact`, `list_memory_facts`, `delete_memory_fact`, `save_document_summary`, `get_document_summary`, `invalidate_document_summaries`
- Wire `build_long_context` into `run_retrieval_query` command
- Post-ingestion hook to trigger `summarization_worker.py` per file

**Token budget:**
```
8192 model window - 1024 output reserve - 256 system = 6912 usable
Allocated: instructions 4% | memory facts 10% | history 20% | chunks 60% | summaries 10%
```

---

## Quality Gates (Must Pass to Exit Any Phase)

| Metric | Threshold |
|---|---|
| Ingestion success rate | >= 98% |
| Retrieval top-5 relevance | >= 85% |
| Extraction precision (critical fields) | >= 95% |
| Harmful file action rate | < 1% |
| Query latency p95 | < 8 seconds |

Two consecutive benchmark runs must pass before phase exit. Source: `docs/prd/PRD-000-MASTER.md`.

---

## Safety Rules (Non-Negotiable)
1. File rename/move stays preview-only until harmful_action_rate < 1% validated.
2. Every non-empty extraction answer must include at least one citation.
3. No cloud calls — all AI inference is local (Ollama + sentence-transformers + Tesseract).
4. Memory never leaves the device — all stored in local SQLite only.
5. Every executed file action must produce an undo record.

---

## Key Docs to Reference
- `docs/prd/PRD-000-MASTER.md` — canonical thresholds + validation protocol
- `docs/prd/PRD-001-INGESTION-INDEXING.md` — ingestion requirements
- `docs/prd/PRD-002-RETRIEVAL-EXTRACTION.md` — retrieval + extraction requirements
- `docs/prd/PRD-003-SAFE-FILE-ACTIONS.md` — file action safety requirements
- `docs/prd/PRD-004-MEMORY-LONG-CONTEXT.md` — memory + long context requirements
- `docs/ALIV_FILE_LEVEL_IMPLEMENTATION_CHECKLIST.md` — file-by-file done/pending status
- `docs/ALIV_IMPLEMENTATION_BACKLOG.md` — Workstreams A–F with acceptance checks
- `docs/architecture/ALIV_TECHNICAL_DESIGN.md` — module design + worker contracts
- `docs/design/ALIV_PRODUCT_DESIGN.md` — all 9 UI screens with components + states

---

## What To Work On Now (Ordered by Priority)

### Immediate (Phase 1 fixes — do before building corpus)
1. Fix `python_bridge.rs`: batch all chunk embeddings into one Python call per ingestion run
2. Fix `db.rs::search_embeddings()`: replace O(N) cosine scan with sqlite-vec `vec_distance_cosine` query
3. Add timeout to `run_worker()` in `python_bridge.rs`
4. Move orchestration from `db.rs` into `core/ingestion.rs`

### After corpus is built (Phase 1 validation)
5. Build 200-file test corpus (benchmarks/corpus/)
6. Define 50 gold benchmark questions (benchmarks/labels/questions_gold.v1.json)
7. Run `benchmarks/validate_ingestion_thresholds.py` — achieve >= 98% ingestion success
8. Two consecutive benchmark passes → Phase 1 gate closes

### Phase 2 implementation
9. Implement memory DB methods in `infra/db.rs` (see Memory section above)
10. Implement hybrid retrieval in `core/retrieval.rs`: SQL metadata filter + sqlite-vec ANN merge
11. Implement citation binding in `python/workers/extractors/citation_binding.py`
12. Implement extraction templates in `python/workers/extractors/templates.py`
13. Wire `build_long_context` into retrieval query path
14. Add post-ingestion summarization trigger
15. Build React frontend screens (see `docs/design/ALIV_PRODUCT_DESIGN.md` for 9 screen specs)
