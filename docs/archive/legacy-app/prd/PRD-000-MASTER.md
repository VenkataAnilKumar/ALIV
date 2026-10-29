# PRD-000 Master Product Requirements

## 1. Product Name
Autonomous Local Intelligence Vault (ALIV)

## 2. Problem Statement
Professionals with sensitive documents need intelligent retrieval, extraction, and file organization without moving data to cloud AI services.

## 3. Product Objective
Build a local-first document intelligence system that can:
1. Ingest and index mixed local documents.
2. Retrieve relevant evidence for natural-language questions.
3. Extract structured facts with citations.
4. Execute safe file operations in preview-first mode.

## 4. Scope (Current Phase)
In scope:
1. Local corpus ingestion and indexing.
2. Retrieval and extraction benchmarks.
3. Dry-run file action recommendations with rollback support.
4. Performance and failure telemetry.
5. Persistent conversation memory across sessions (PRD-004).
6. Document summary cache for long-context handling (PRD-004).
7. User-pinned memory facts with citations (PRD-004).

Out of scope:
1. Multi-user collaboration.
2. Cloud sync or managed cloud inference.
3. Billing and production tenancy.
4. Fine-tuning or model weight updates from memory.

## 5. Personas
1. Solo operator (founder/developer) validating product quality.
2. Future end users in legal and finance workflows.

## 6. Architecture Constraints
1. Rust and Tauri backend shell.
2. React and TypeScript UI.
3. SQLite with sqlite-vec (ANN search via vec0 virtual tables — not brute-force scan).
4. Python subprocess bridge for OCR/parsing and ML pipelines.
5. Local LLM via Ollama (`llama3.2:3b` default) — no cloud inference.
6. Embeddings via sentence-transformers `all-MiniLM-L6-v2` (384 dims, batch size 32).
7. Minimum hardware: 8 GB RAM, 4-core CPU, 10 GB disk free.

See `docs/ALIV_PRODUCT_REQUIREMENTS.md` Section 2.1 for full model and hardware spec.

## 7. Non-Functional Requirements
1. Local-only processing by default.
2. Deterministic benchmark runs (fixed corpus, fixed model versions).
3. Audit trail for actions and failures.
4. Stable operation on reference development machine.

## 8. Validation Thresholds (Must Pass)
1. Ingestion success rate >= 98%.
2. Retrieval relevance at top-5 >= 85%.
3. Extraction precision on critical fields >= 95%.
4. Harmful file action rate < 1%.
5. Query latency p95 < 8 seconds.

## 8.1 Metric Definitions (Normative)
1. Ingestion success rate = indexed files / discovered eligible files.
2. Retrieval relevance at top-5 = fraction of benchmark queries where at least one gold evidence chunk appears in top-5.
3. Extraction precision = correct extracted fields / total extracted fields for benchmark-labeled fields.
4. Harmful file action = any rename or move that places a file into an incorrect folder, assigns wrong entity/date in filename, or causes data loss risk.
5. Query latency p95 = 95th percentile of end-to-end query response time over 50 benchmark queries on reference machine.

## 8.2 Validation Protocol
1. Use fixed corpus and fixed model versions during each validation cycle.
2. Run two consecutive full benchmark passes before declaring phase pass.
3. Record all run results with run id, model versions, corpus version, and timestamp.
4. Any regression in safety metrics blocks phase exit even if other metrics pass.

## 9. Safety Requirements
1. Preview-only mode for file actions until pass thresholds are met.
2. One-click undo for every executed action.
3. Collision-safe naming and deterministic fallback suffixing.
4. Citation-required extraction outputs.

## 10. Data and Benchmark Pack
1. 200-file local test corpus (mixed formats and quality).
2. 50 benchmark questions with gold outputs.
3. Gold labels for expiry date, vendor name, and penalty clause.

## 11. Milestones
1. M1: Ingestion and indexing stable.
2. M2: Retrieval and extraction meet quality threshold.
3. M3: File actions meet safety threshold.
4. M4: End-to-end benchmark pass and weekly validation report.

## 12. Exit Criteria for Current Phase
Exit when all five validation thresholds pass in two consecutive benchmark runs and no critical safety defects remain open.
