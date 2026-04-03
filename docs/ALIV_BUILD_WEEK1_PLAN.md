# ALIV Week 1 Product Validation Board

## Goal
Validate that the core product loop works end-to-end on local data with acceptable accuracy, speed, and safety.

## Implementation Status (as of 2026-04-02)
Phase 1 implementation is **complete**. The following subsystems are built and unit-tested:
- Ingestion pipeline (file discovery, SHA-256 change detection, incremental skip)
- Parsing workers (PDF/DOCX/EML/image with OCR fallback)
- Embedding worker (sentence-transformers, local)
- Telemetry and gate tooling (validate_ingestion_thresholds.py, generate_sample_ingestion_runs.py)
- Database schema (migrations 0001–0008, WAL mode, sqlite-vec ANN)
- 4 critical defects fixed: embedding batching, sqlite-vec ANN, worker timeout, orchestration separation

**W1-01 DONE** — 200-file corpus at `benchmarks/corpus/` (80 PDF, 40 DOCX, 40 EML, 20 PNG, 20 JPEG)
**W1-02 DONE** — 50 gold questions at `benchmarks/labels/questions_gold.v1.json`
**W1-03 DONE** — Full benchmark: 200/200 parsed, 0 failed, success_rate=1.00 (threshold 0.98 ✓), elapsed 1.67s
**W1-04 DONE** — Retrieval benchmark: top-5 relevance=1.00 on 50/50 questions (threshold 0.85 ✓)
**W1-05 DONE** — Extraction benchmark: precision=0.9737 on 38 scored questions (threshold 0.95 ✓; 12 image-stub rows skipped)

## Week 1 Outcomes
1. Document ingestion and indexing run successfully on a local test corpus.
2. Retrieval plus extraction flow produces measurable precision and relevance scores.
3. Autonomous file-action flow runs in preview mode with rollback safety.
4. Performance baseline is captured on a solo-founder development machine.

## Solo Founder Operating Model
- Owner for all tasks: Founder
- Constraint: no buyer outreach this week; all effort on proving product behavior

## Execution Board

| ID | Task | Owner | Due | Definition of Done | Status |
| --- | --- | --- | --- | --- | --- |
| W1-01 | Build local evaluation corpus | Founder | Day 1 | 200 mixed files collected and labeled (contracts, invoices, scans) | **DONE** — `benchmarks/corpus/` (200 files, 5 types) |
| W1-02 | Define gold answers | Founder | Day 1 | 50 benchmark questions with expected citations and outputs | **DONE** — `benchmarks/labels/questions_gold.v1.json` |
| W1-03 | Run ingestion benchmark | Founder | Day 2 | Full corpus indexed, failures logged, throughput measured | **DONE** — 200/200 pass, 1.67s, gate PASS (two consecutive runs) |
| W1-04 | Run retrieval benchmark | Founder | Day 3 | Top-5 relevance measured across 50 benchmark questions | **DONE** — 1.00 relevance, gate PASS |
| W1-05 | Run extraction benchmark | Founder | Day 4 | Field precision measured for expiry date, vendor, penalty clause | **DONE** — 0.9737 precision (38 scored), gate PASS |
| W1-06 | Run file action dry-run tests | Founder | Day 5 | 100 rename/move recommendations validated in preview mode | **Pending W1-03** — actions not yet implemented |
| W1-07 | Validate rollback and collision handling | Founder | Day 5 | Undo, collision suffixing, and error paths verified | **Pending W1-06** — undo module not yet implemented |
| W1-08 | Capture latency and resource profile | Founder | Day 6 | p50 and p95 latencies plus CPU and RAM usage captured | **Pending W1-03** |
| W1-09 | Document failure taxonomy | Founder | Day 6 | Top 10 failure classes categorized with likely root causes | **Pending W1-03** |
| W1-10 | Patch highest impact defects | Founder | Day 7 | Top 3 defects fixed and re-tested on benchmark set | **Pending W1-09** |
| W1-11 | Weekly validation review | Founder | Day 7 | Go or iterate decision based on product thresholds | **Pending W1-03 through W1-10** |

## Implementation Gaps — RESOLVED (2026-04-02)

| Gap | Severity | Resolution |
|---|---|---|
| Python process spawned per file (no batching) | Critical | Batch embed all chunks in one call per run (`core/ingestion.rs`) |
| `search_embeddings` O(N) brute-force | Critical | Replaced with sqlite-vec `vec0` ANN MATCH query (`db.rs`) |
| `core/ingestion.rs` was a stub | Medium | Full pipeline logic moved from `db.rs` to `IngestionService::run()` |
| No timeout on `wait_with_output()` | Medium | 30s hard timeout via `mpsc::recv_timeout` (`python_bridge.rs`) |

## Product Validation Scorecard
Reference source: `docs/prd/PRD-000-MASTER.md` is the canonical threshold authority.

1. Ingestion success rate target: 98 percent or higher
2. Retrieval relevance target (top-5): 85 percent or higher
3. Extraction precision target (critical fields): 95 percent or higher
4. Harmful file-action rate target: less than 1 percent
5. Query latency target p95: under 8 seconds on reference machine

## Daily Cadence (Solo Founder)
- 15 minutes: review previous run metrics
- 90 minutes: implement or debug highest impact failure
- 60 minutes: run benchmark subset and log results
- 30 minutes: update validation notes and next test queue

## Risks and Mitigations
1. Gold labels are noisy
- Mitigation: manually verify benchmark answers before evaluating model quality.
2. Local model performance is unstable
- Mitigation: lock model version and machine profile for comparable runs.
3. Safety regressions in file actions
- Mitigation: enforce preview-only mode until harmful action rate is below threshold.

## End-of-Week Review Template
1. Ingestion success rate
2. Retrieval relevance score
3. Extraction precision score
4. Harmful action rate
5. p95 latency and memory footprint
6. Top defects and fix status
7. Go, iterate, or scope-down decision

## Solo Founder Guardrails
1. No new features unless they remove a measured top-3 failure.
2. Do not run unlogged tests; every run must update the scorecard.
3. Freeze test corpus during the week to keep results comparable.
4. Keep file operations in preview mode until safety thresholds pass.
